mod i18n;
mod generator;
mod writer;
mod args;

use rayon::prelude::*;
use std::io;
use std::time::Instant;
use std::sync::mpsc;
use crate::writer::OutputFormat;

const APP_NAME: &str = "PASSWG";
const VERSION: &str = "1.2.0";
const CHUNK_SIZE: u64 = 10000;

fn main() -> io::Result<()> {
    let locale = i18n::get_locale();
    let raw_args: Vec<String> = std::env::args().collect();
    
    if raw_args.contains(&"-h".to_string()) || raw_args.contains(&"--help".to_string()) {
        args::print_help(locale, APP_NAME, VERSION);
        return Ok(());
    }

    let config = args::parse_args(&raw_args);
    if config.count == 0 { return Ok(()); }

    let start_time = if config.show_stats { Some(Instant::now()) } else { None };
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(256);

    // Поток записи
    let writer_thread = std::thread::spawn(move || {
        writer::start_writer(rx, config.out_file, config.format)
    });

    let num_chunks = config.count / CHUNK_SIZE;
    let remainder = config.count % CHUNK_SIZE;

    // Параллельная генерация
    (0..=num_chunks).into_par_iter().for_each_with(tx, |tx, chunk_idx| {
        let current_chunk_size = if chunk_idx < num_chunks { CHUNK_SIZE } else { remainder };
        if current_chunk_size == 0 { return; }

        let mut local_buf = Vec::with_capacity(current_chunk_size as usize * (config.length + 60));
        let mut rnd_buf = [0u8; 16384];
        let mut rnd_pos = rnd_buf.len();

        for i in 0..current_chunk_size {
            let global_id = chunk_idx * CHUNK_SIZE + i + 1;
            
            // Запись метаданных (ID)
            match config.format {
                OutputFormat::Json => {
                    local_buf.extend_from_slice(b"{\"id\":");
                    generator::fast_write_u64(&mut local_buf, global_id);
                    local_buf.extend_from_slice(b",\"pass\":\"");
                },
                OutputFormat::Csv => {
                    generator::fast_write_u64(&mut local_buf, global_id);
                    local_buf.extend_from_slice(b",\"");
                },
                _ => {}
            }

            // Генерация самого пароля
            for _ in 0..config.length {
                loop {
                    if rnd_pos >= rnd_buf.len() { 
                        let _ = getrandom::fill(&mut rnd_buf); 
                        rnd_pos = 0; 
                    }
                    let val = rnd_buf[rnd_pos];
                    rnd_pos += 1;
                    if config.fast_mode {
                        local_buf.push(generator::CHARSET_FAST[(val & 63) as usize]);
                        break;
                    } else if (val as usize) < generator::LIMIT {
                        local_buf.push(generator::CHARSET[(val as usize) % generator::CHARSET_LEN]);
                        break;
                    }
                }
            }

            // Закрытие строки/объекта
            match config.format {
                OutputFormat::Json => {
                    local_buf.extend_from_slice(b"\"}");
                    if global_id < config.count { local_buf.extend_from_slice(b",\n"); }
                },
                OutputFormat::Csv => local_buf.extend_from_slice(b"\"\n"),
                _ => local_buf.push(b'\n'),
            }
        }
        let _ = tx.send(local_buf);
    });

    let _ = writer_thread.join().unwrap();
    
    if config.copy_mode && config.count == 1 { 
        generator::copy_to_clipboard(config.length, config.fast_mode); 
    }
    
    if let Some(start) = start_time { 
        generator::print_report(start, config.count, config.length, locale); 
    }
    
    Ok(())
}