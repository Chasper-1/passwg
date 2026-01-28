use getrandom::fill;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::process::{Command, Stdio};
use std::time::Instant;
use std::sync::mpsc;

const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&'()*+,-./:;<=>?@[]^_`{|}~";
const CHARSET_LEN: usize = CHARSET.len();
const LIMIT: usize = (256 / CHARSET_LEN) * CHARSET_LEN;
const CHARSET_FAST: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
const CHUNK_SIZE: u64 = 10000; // Увеличили размер пачки

#[derive(PartialEq, Clone, Copy)]
enum OutputFormat { Plain, Json, Csv }

// Быстрая конвертация числа без аллокаций
fn fast_write_u64(buf: &mut Vec<u8>, mut n: u64) {
    if n == 0 { buf.push(b'0'); return; }
    let mut temp = [0u8; 20];
    let mut i = 20;
    while n > 0 {
        i -= 1;
        temp[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    buf.extend_from_slice(&temp[i..]);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help(); return Ok(());
    }

    let (length, count, show_stats, fast_mode, copy_mode, out_file, format) = parse_args(&args);
    if count == 0 { return Ok(()); }

    let start_time = if show_stats { Some(Instant::now()) } else { None };
    // Увеличили очередь канала до 256, чтобы ядра не ждали
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(256);

    let writer_thread = std::thread::spawn(move || -> io::Result<()> {
        // УСТАНОВИЛИ БУФЕР 32 МБ ДЛЯ ЩАДЯЩЕГО РЕЖИМА SSD
        let mut out: Box<dyn Write> = if let Some(path) = out_file {
            Box::new(BufWriter::with_capacity(32 * 1024 * 1024, File::create(path)?))
        } else {
            Box::new(BufWriter::with_capacity(1024 * 1024, io::stdout()))
        };

        if format == OutputFormat::Csv { writeln!(out, "id,password")?; }
        else if format == OutputFormat::Json { write!(out, "[\n")?; }

        for received in rx { out.write_all(&received)?; }

        if format == OutputFormat::Json { write!(out, "\n]")?; }
        out.flush()?;
        Ok(())
    });

    let num_chunks = count / CHUNK_SIZE;
    let remainder = count % CHUNK_SIZE;

    (0..=num_chunks).into_par_iter().for_each_with(tx, |tx, chunk_idx| {
        let current_chunk_size = if chunk_idx < num_chunks { CHUNK_SIZE } else { remainder };
        if current_chunk_size == 0 { return; }

        let mut local_buf = Vec::with_capacity(current_chunk_size as usize * (length + 60));
        let mut rnd_buf = [0u8; 16384];
        let mut rnd_pos = rnd_buf.len();

        for i in 0..current_chunk_size {
            let global_id = chunk_idx * CHUNK_SIZE + i + 1;
            match format {
                OutputFormat::Json => {
                    local_buf.extend_from_slice(b"{\"id\":");
                    fast_write_u64(&mut local_buf, global_id);
                    local_buf.extend_from_slice(b",\"pass\":\"");
                },
                OutputFormat::Csv => {
                    fast_write_u64(&mut local_buf, global_id);
                    local_buf.extend_from_slice(b",\"");
                },
                _ => {}
            }
            for _ in 0..length {
                loop {
                    if rnd_pos >= rnd_buf.len() { let _ = fill(&mut rnd_buf); rnd_pos = 0; }
                    let val = rnd_buf[rnd_pos];
                    rnd_pos += 1;
                    if fast_mode {
                        local_buf.push(CHARSET_FAST[(val & 63) as usize]);
                        break;
                    } else if (val as usize) < LIMIT {
                        local_buf.push(CHARSET[(val as usize) % CHARSET_LEN]);
                        break;
                    }
                }
            }
            match format {
                OutputFormat::Json => {
                    local_buf.extend_from_slice(b"\"}");
                    if global_id < count { local_buf.extend_from_slice(b",\n"); }
                },
                OutputFormat::Csv => local_buf.extend_from_slice(b"\"\n"),
                _ => local_buf.push(b'\n'),
            }
        }
        let _ = tx.send(local_buf);
    });

    let _ = writer_thread.join();
    if copy_mode && count == 1 { copy_to_clipboard(length, fast_mode); }
    if let Some(start) = start_time { print_report(start, count, length); }
    Ok(())
}

fn parse_args(args: &[String]) -> (usize, u64, bool, bool, bool, Option<String>, OutputFormat) {
    let mut length = 16; let mut count = 1; let mut show_stats = false;
    let mut fast_mode = false; let mut copy_mode = false;
    let mut out_file = None; let mut format = OutputFormat::Plain;
    let mut nums = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--stats" => show_stats = true,
            "-f" | "--fast" => fast_mode = true,
            "-c" | "--copy" => copy_mode = true,
            "--json" => format = OutputFormat::Json,
            "--csv" => format = OutputFormat::Csv,
            "-o" => { if i + 1 < args.len() { out_file = Some(args[i+1].clone()); i += 1; } },
            _ => if let Ok(n) = args[i].parse::<u64>() { nums.push(n); }
        }
        i += 1;
    }
    if let Some(&l) = nums.get(0) { length = l as usize; }
    if let Some(&c) = nums.get(1) { count = c; }
    (if length == 0 { 1 } else { length }, count, show_stats, fast_mode, copy_mode, out_file, format)
}

fn copy_to_clipboard(length: usize, fast_mode: bool) {
    let mut pwd = String::new();
    let mut rb = vec![0u8; length * 4]; let _ = fill(&mut rb);
    let mut p = 0;
    for _ in 0..length {
        loop {
            if p >= rb.len() { let _ = fill(&mut rb); p = 0; }
            let v = rb[p] as usize; p += 1;
            if fast_mode { pwd.push(CHARSET_FAST[v & 63] as char); break; }
            else if v < LIMIT { pwd.push(CHARSET[v % CHARSET_LEN] as char); break; }
        }
    }
    let _ = Command::new("wl-copy").stdin(Stdio::piped()).spawn().map(|mut c| {
        if let Some(mut si) = c.stdin.take() { let _ = si.write_all(pwd.as_bytes()); }
        let _ = c.wait();
    });
}

fn print_report(start: Instant, count: u64, length: usize) {
    let dur = start.elapsed().as_secs_f64();
    if dur > 0.0 {
        let bytes = count * (length as u64 + 1);
        let mib_s = (bytes as f64 / 1048576.0) / dur;
        let p_s = count as f64 / dur;

        fn format_number(n: u64) -> String {
            let s = n.to_string();
            s.as_bytes().rchunks(3).rev()
                .map(|chunk| std::str::from_utf8(chunk).unwrap())
                .collect::<Vec<_>>().join(" ")
        }

        eprintln!("\n--- СТАТИСТИКА ---");
        eprintln!("Время выполнения:    {:.3} сек", dur);
        eprintln!("Скорость потока:     {:.2} MiB/s", mib_s);
        eprintln!("Производительность:  {} пар/сек", format_number(p_s as u64));
        eprintln!("------------------");
    }
}

fn print_help() {
    println!("PASSWG\nАргументы: [длина] [количество]\nФлаги: -c (copy), -s (stats), -f (fast), -o [file], --json, --csv");
}