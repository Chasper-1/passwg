mod args;
mod generator;
mod i18n;
mod words;
mod writer;

use crate::writer::OutputFormat;
use rayon::prelude::*;
use std::io;
use std::sync::mpsc;
use std::time::Instant;

const APP_NAME: &str = "PASSWG";
const VERSION: &str = "1.3.2";  // Увеличили версию
const CHUNK_SIZE: u64 = 10000;

fn main() -> io::Result<()> {
    let locale = i18n::get_locale();
    let raw_args: Vec<String> = std::env::args().collect();

    if raw_args.len() > 1 {
        let first_arg = &raw_args[1];
        if first_arg == "-h" || first_arg == "--help" {
            args::print_help(locale, APP_NAME, VERSION);
            return Ok(());
        }
    }

    let config = args::parse_args(&raw_args);
    if config.count == 0 { 
        return Ok(());
    }

    let start_time = if config.show_stats { Some(Instant::now()) } else { None };
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(256);

    let writer_thread = std::thread::spawn(move || writer::start_writer(rx, config.out_file, config.format));

    let num_chunks = (config.count + CHUNK_SIZE - 1) / CHUNK_SIZE;
    let mut first_pwd_for_copy: Option<String> = None;

    for i in 0..num_chunks {
        let start_id = i * CHUNK_SIZE + 1;
        let current_chunk_size = if i == num_chunks - 1 {
            config.count - (num_chunks - 1) * CHUNK_SIZE
        } else {
            CHUNK_SIZE
        };

        let chunk_data: Vec<Vec<u8>> = (0..current_chunk_size)
            .into_par_iter()
            .map(|_| {
                let mut buf = Vec::with_capacity(config.length + 1);
                let mut rnd_data = [0u8; 64];
                let mut rnd_pos = 0;
                let _ = getrandom::fill(&mut rnd_data);

                if config.word_mode {
                    for k in 0..config.length {
                        let word = generator::get_random_word(&mut rnd_data, &mut rnd_pos);
                        buf.extend_from_slice(word.as_bytes());
                        if k < config.length - 1 { buf.push(b'-'); }
                    }
                } else {
                    for _ in 0..config.length {
                        loop {
                            let mut b = [0u8; 1];
                            let _ = getrandom::fill(&mut b);
                            if config.fast_mode {
                                buf.push(generator::CHARSET_FAST[(b[0] & 63) as usize]);
                                break;
                            } else if (b[0] as usize) < generator::LIMIT {
                                buf.push(generator::CHARSET[(b[0] as usize) % generator::CHARSET_LEN]);
                                break;
                            }
                        }
                    }
                }
                buf
            })
            .collect();

        // Сохраняем первый пароль для копирования
        if config.copy_mode && first_pwd_for_copy.is_none() && !chunk_data.is_empty() {
            let first_pwd_bytes = &chunk_data[0];
            first_pwd_for_copy = Some(String::from_utf8_lossy(first_pwd_bytes).to_string());
        }

        for (idx, data) in chunk_data.into_iter().enumerate() {
            let mut formatted = Vec::new();
            let current_id = start_id + idx as u64;
            
            match config.format {
                OutputFormat::Csv => {
                    generator::fast_write_u64(&mut formatted, current_id);
                    formatted.push(b',');
                    formatted.extend_from_slice(&data);
                    formatted.push(b'\n');
                },
                OutputFormat::Json => {
                    if current_id == 1 {
                        formatted.extend_from_slice(b"  \"");
                    } else {
                        formatted.extend_from_slice(b",\n  \"");
                    }
                    formatted.extend_from_slice(&data);
                    formatted.push(b'\"');
                },
                OutputFormat::Plain => {
                    formatted.extend_from_slice(&data);
                    formatted.push(b'\n');
                }
            }
            
            tx.send(formatted).unwrap();
        }
    }

    drop(tx);
    let _ = writer_thread.join();

    // Копируем в буфер обмена ПОСЛЕ завершения вывода
    if let Some(pwd) = first_pwd_for_copy {
        generator::copy_to_clipboard(&pwd);
    }
    
    if let Some(start) = start_time {
        generator::print_report(start, config.count, config.length, locale);
    }
    
    Ok(())
}