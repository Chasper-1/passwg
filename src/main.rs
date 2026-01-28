use getrandom::fill;
use rayon::prelude::*;
use std::io::{self, Write};
use std::time::Instant;
use arboard::Clipboard; // Не забудь про arboard = "3.4" в Cargo.toml

const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
const CHARSET_LEN: usize = CHARSET.len();
const LIMIT: usize = (256 / CHARSET_LEN) * CHARSET_LEN;
const CHARSET_FAST: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
const CHUNK_SIZE: u64 = 5000;

fn print_help() {
    println!("PASSWG - Ультимативный генератор паролей");
    println!("\nИспользование:");
    println!("  passwg [длина] [количество] [флаги]");
    println!("\nАргументы:");
    println!("  длина         Длина пароля (по умолчанию 16)");
    println!("  количество    Сколько паролей создать (по умолчанию 1)");
    println!("\nФлаги:");
    println!("  -c, --copy    Скопировать пароль в буфер обмена (только для 1 шт.)");
    println!("  -s, --stats   Показать скорость и статистику (в stderr)");
    println!("  -f, --fast    Режим максимальной скорости (64 символа, без Bias)");
    println!("  -h, --help    Показать это окно");
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let (length, count, show_stats, fast_mode, copy_mode) = parse_args(&args);
    if count == 0 { return Ok(()); }

    let start_time = if show_stats { Some(Instant::now()) } else { None };

    // Параллельная генерация
    let num_chunks = count / CHUNK_SIZE;
    let remainder = count % CHUNK_SIZE;

    (0..=num_chunks).into_par_iter().for_each(|chunk_idx| {
        let current_chunk_size = if chunk_idx < num_chunks { CHUNK_SIZE } else { remainder };
        if current_chunk_size == 0 { return; }

        let mut local_output = Vec::with_capacity(current_chunk_size as usize * (length + 1));
        let mut rnd_buf = [0u8; 16384];
        let mut rnd_pos = rnd_buf.len();

        for _ in 0..current_chunk_size {
            for _ in 0..length {
                loop {
                    if rnd_pos >= rnd_buf.len() {
                        let _ = fill(&mut rnd_buf);
                        rnd_pos = 0;
                    }
                    let val = rnd_buf[rnd_pos];
                    rnd_pos += 1;

                    if fast_mode {
                        let idx = (val & 63) as usize;
                        unsafe { local_output.push(*CHARSET_FAST.get_unchecked(idx)); }
                        break;
                    } else {
                        let val_usize = val as usize;
                        if val_usize < LIMIT {
                            let idx = val_usize % CHARSET_LEN;
                            unsafe { local_output.push(*CHARSET.get_unchecked(idx)); }
                            break;
                        }
                    }
                }
            }
            local_output.push(b'\n');
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let _ = handle.write_all(&local_output);
    });

    // Логика копирования (только если count == 1)
    if copy_mode && count == 1 {
        let mut pwd = String::with_capacity(length);
        let mut rb = vec![0u8; length * 4]; // Запас для Rejection Sampling
        let _ = fill(&mut rb);
        let mut p = 0;
        for _ in 0..length {
            loop {
                if p >= rb.len() {
                    let _ = fill(&mut rb);
                    p = 0;
                }
                let v = rb[p] as usize;
                p += 1;
                if fast_mode {
                    pwd.push(CHARSET_FAST[v & 63] as char);
                    break;
                } else if v < LIMIT {
                    pwd.push(CHARSET[v % CHARSET_LEN] as char);
                    break;
                }
            }
        }
        if let Ok(mut clipboard) = Clipboard::new() {
            if clipboard.set_text(pwd).is_ok() {
                eprintln!("(Пароль скопирован в буфер обмена)");
            }
        }
    }

    if let Some(start) = start_time {
        print_report(start, count, length);
    }

    Ok(())
}

fn parse_args(args: &[String]) -> (usize, u64, bool, bool, bool) {
    let mut length = 16;
    let mut count = 1;
    let mut show_stats = false;
    let mut fast_mode = false;
    let mut copy_mode = false;
    let mut nums = Vec::new();

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-s" | "--stats" => show_stats = true,
            "-f" | "--fast" => fast_mode = true,
            "-c" | "--copy" => copy_mode = true,
            _ => if let Ok(n) = arg.parse::<u64>() { nums.push(n); }
        }
    }

    if let Some(&l) = nums.get(0) { length = l as usize; }
    if let Some(&c) = nums.get(1) { count = c; }

    (if length == 0 { 1 } else { length }, count, show_stats, fast_mode, copy_mode)
}

fn print_report(start: Instant, count: u64, length: usize) {
    let duration = start.elapsed();
    let seconds = duration.as_secs_f64();
    if seconds == 0.0 { return; }
    let total_bytes = count * (length as u64 + 1);
    let mib_per_sec = (total_bytes as f64 / (1024.0 * 1024.0)) / seconds;
    
    eprintln!("\n--- СТАТИСТИКА ГЕНЕРАЦИИ ---");
    eprintln!("Время выполнения:    {:.3} сек", seconds);
    eprintln!("Скорость потока:     {:.2} MiB/s", mib_per_sec);
    eprintln!("Производительность:  {:.0} паролей/сек", count as f64 / seconds);
    eprintln!("----------------------------");
}