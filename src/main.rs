use getrandom::fill;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Instant;

const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
const CHARSET_LEN: usize = CHARSET.len();
const LIMIT: usize = (256 / CHARSET_LEN) * CHARSET_LEN;
const CHARSET_FAST: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
const CHUNK_SIZE: u64 = 5000;

#[derive(PartialEq, Clone, Copy)]
enum OutputFormat {
    Plain,
    Json,
    Csv,
}

fn print_help() {
    println!("PASSWG - Ультимативный генератор паролей");
    println!("\nИспользование: passwg [длина] [количество] [флаги]");
    println!("\nАргументы:");
    println!("  длина         Длина пароля (по умолчанию 16)");
    println!("  количество    Сколько паролей создать (по умолчанию 1)");
    println!("\nФлаги:");
    println!("  -c, --copy    Скопировать пароль через wl-copy (только для 1 шт.)");
    println!("  -s, --stats   Показать скорость и статистику (в stderr)");
    println!("  -f, --fast    Режим максимальной скорости (64 символа, без Bias)");
    println!("  -o [file]     Запись в файл вместо stdout");
    println!("  --json        Вывод в формате JSON Lines");
    println!("  --csv         Вывод в формате CSV (id, password)");
    println!("  -h, --help    Показать это окно");
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let (length, count, show_stats, fast_mode, copy_mode, out_file, format) = parse_args(&args);
    if count == 0 {
        return Ok(());
    }

    let start_time = if show_stats {
        Some(Instant::now())
    } else {
        None
    };

    // Канал для передачи готовых буферов в поток записи
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(128);

    // Поток-писатель (Consumer)
    let writer_thread = std::thread::spawn(move || -> io::Result<()> {
        let mut out: Box<dyn Write> = if let Some(path) = out_file {
            Box::new(BufWriter::with_capacity(512 * 1024, File::create(path)?))
        } else {
            Box::new(BufWriter::with_capacity(512 * 1024, io::stdout()))
        };

        for received in rx {
            out.write_all(&received)?;
        }
        out.flush()?;
        Ok(())
    });

    // Параллельная генерация (Producers)
    let num_chunks = count / CHUNK_SIZE;
    let remainder = count % CHUNK_SIZE;

    (0..=num_chunks)
        .into_par_iter()
        .for_each_with(tx, |tx, chunk_idx| {
            let current_chunk_size = if chunk_idx < num_chunks {
                CHUNK_SIZE
            } else {
                remainder
            };
            if current_chunk_size == 0 {
                return;
            }

            let mut local_buf = Vec::with_capacity(current_chunk_size as usize * (length + 40));
            let mut rnd_buf = [0u8; 16384];
            let mut rnd_pos = rnd_buf.len();

            for i in 0..current_chunk_size {
                let global_id = chunk_idx * CHUNK_SIZE + i + 1;

                match format {
                    OutputFormat::Json => local_buf
                        .extend_from_slice(format!(r#"{{"id":{},"pass":""#, global_id).as_bytes()),
                    OutputFormat::Csv => {
                        local_buf.extend_from_slice(format!(r#"{},""#, global_id).as_bytes())
                    } // Начало CSV с кавычкой
                    _ => {}
                }

                for _ in 0..length {
                    loop {
                        if rnd_pos >= rnd_buf.len() {
                            let _ = fill(&mut rnd_buf);
                            rnd_pos = 0;
                        }
                        let val = rnd_buf[rnd_pos];
                        rnd_pos += 1;

                        let ch = if fast_mode {
                            CHARSET_FAST[(val & 63) as usize]
                        } else if (val as usize) < LIMIT {
                            CHARSET[(val as usize) % CHARSET_LEN]
                        } else {
                            continue;
                        };

                        match format {
                            OutputFormat::Json => {
                                if ch == b'"' || ch == b'\\' {
                                    local_buf.push(b'\\');
                                }
                                local_buf.push(ch);
                            }
                            OutputFormat::Csv => {
                                if ch == b'"' {
                                    local_buf.push(b'"');
                                } // В CSV кавычка удваивается
                                local_buf.push(ch);
                            }
                            _ => local_buf.push(ch),
                        }
                        break;
                    }
                }

                match format {
                    OutputFormat::Json => local_buf.extend_from_slice(b"\"}\n"),
                    OutputFormat::Csv => local_buf.extend_from_slice(b"\"\n"), // Конец CSV с кавычкой
                    _ => local_buf.push(b'\n'),
                }
            }
            let _ = tx.send(local_buf);
        });

    // Дожидаемся завершения записи
    writer_thread.join().unwrap()?;

    if copy_mode && count == 1 {
        copy_to_clipboard(length, fast_mode);
    }

    if let Some(start) = start_time {
        print_report(start, count, length);
    }

    Ok(())
}

fn copy_to_clipboard(length: usize, fast_mode: bool) {
    let mut pwd = String::new();
    let mut rb = vec![0u8; length * 4];
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
    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(pwd.as_bytes());
        }
        let _ = child.wait();
        eprintln!("(Пароль скопирован)");
    }
}

fn parse_args(args: &[String]) -> (usize, u64, bool, bool, bool, Option<String>, OutputFormat) {
    let mut length = 16;
    let mut count = 1;
    let mut show_stats = false;
    let mut fast_mode = false;
    let mut copy_mode = false;
    let mut out_file = None;
    let mut format = OutputFormat::Plain;
    let mut nums = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--stats" => show_stats = true,
            "-f" | "--fast" => fast_mode = true,
            "-c" | "--copy" => copy_mode = true,
            "--json" => format = OutputFormat::Json,
            "--csv" => format = OutputFormat::Csv,
            "-o" => {
                if i + 1 < args.len() {
                    out_file = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {
                if let Ok(n) = args[i].parse::<u64>() {
                    nums.push(n);
                }
            }
        }
        i += 1;
    }
    if let Some(&l) = nums.get(0) {
        length = l as usize;
    }
    if let Some(&c) = nums.get(1) {
        count = c;
    }
    (
        if length == 0 { 1 } else { length },
        count,
        show_stats,
        fast_mode,
        copy_mode,
        out_file,
        format,
    )
}

fn print_report(start: Instant, count: u64, length: usize) {
    let duration = start.elapsed();
    let seconds = duration.as_secs_f64();
    if seconds == 0.0 {
        return;
    }
    let total_bytes = count * (length as u64 + 1);
    eprintln!(
        "\n--- СТАТИСТИКА ---\nВремя: {:.3} сек\nСкорость: {:.2} MiB/s\n------------------",
        seconds,
        (total_bytes as f64 / 1048576.0) / seconds
    );
}
