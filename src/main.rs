mod args;
mod generator;
mod i18n;
mod words;
mod writer;

use crate::writer::OutputFormat;
use rayon::prelude::*;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;

const APP_NAME: &str = "PASSWG";
const VERSION: &str = "2.2.0";

// Целевой размер данных в одном чанке — 32 КБ (чтобы влезло в L1d любого ядра)
const TARGET_L1_SIZE: usize = 16 * 1024;

fn main() -> std::io::Result<()> {
    let locale = i18n::get_locale();
    let raw_args: Vec<String> = std::env::args().collect();

    if raw_args.len() > 1 && (raw_args[1] == "-h" || raw_args[1] == "--help") {
        args::print_help(locale, APP_NAME, VERSION);
        return Ok(());
    }

    let config = args::parse_args(&raw_args);
    if config.count == 0 {
        return Ok(());
    }

    // АВТОКОРРЕКЦИЯ: Вычисляем размер чанка на лету
    // Примерный размер одного пароля: длина + ID (до 20) + разделители
    let bytes_per_pass = config.length + 20;
    let chunk_size = (TARGET_L1_SIZE / bytes_per_pass).clamp(32, 4096) as u64;

    let start_time = if config.show_stats {
        Some(Instant::now())
    } else {
        None
    };

    let out = writer::get_writer(&config.out_file)?;
    let out_arc = Arc::new(Mutex::new(out));

    {
        let mut out_lock = out_arc.lock().unwrap();
        match config.format {
            OutputFormat::Csv => writeln!(out_lock, "id,password")?,
            OutputFormat::Json => write!(out_lock, "[")?,
            _ => {}
        }
    }

    let num_chunks = (config.count + chunk_size - 1) / chunk_size;
    let first_password = Arc::new(Mutex::new(None));

    (0..num_chunks).into_par_iter().for_each(|chunk_idx| {
        let start_id = chunk_idx * chunk_size + 1;
        let size = if chunk_idx == num_chunks - 1 {
            config.count - chunk_idx * chunk_size
        } else {
            chunk_size
        };

        let data = generator::generate_chunk(
            start_id,
            size,
            config.length,
            config.fast_mode,
            config.word_mode,
            config.format,
            config.rounds
        );

        if config.copy_mode && start_id == 1 {
            let mut fp = first_password.lock().unwrap();
            if fp.is_none() {
                let s = String::from_utf8_lossy(&data);
                *fp = s.lines().next().map(|l| {
                    l.trim_matches(|c| c == ' ' || c == '"' || c == ',')
                        .to_string()
                });
            }
        }

        let mut out_lock = out_arc.lock().unwrap();
        let _ = out_lock.write_all(&data);
    });

    {
        let mut out_lock = out_arc.lock().unwrap();
        if config.format == OutputFormat::Json {
            write!(out_lock, "\n]")?;
        }
        let _ = out_lock.flush();
    }

    if let Some(pwd) = first_password.lock().unwrap().as_ref() {
        generator::copy_to_clipboard(pwd);
    }

    if let Some(start) = start_time {
        generator::print_report(start, config.count, config.length, locale);
    }

    Ok(())
}
