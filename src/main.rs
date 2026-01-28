use getrandom::fill;
use rayon::prelude::*;
use std::io::{self, Write};
use std::time::Instant;

const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
const CHARSET_LEN: usize = CHARSET.len();
const LIMIT: usize = (256 / CHARSET_LEN) * CHARSET_LEN;
const CHUNK_SIZE: u64 = 5000;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let (length, count, show_stats) = parse_args(&args);

    if count == 0 { return Ok(()); }

    let start_time = if show_stats { Some(Instant::now()) } else { None };

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
                    let val = rnd_buf[rnd_pos] as usize;
                    rnd_pos += 1;

                    if val < LIMIT {
                        let idx = val % CHARSET_LEN;
                        unsafe {
                            local_output.push(*CHARSET.get_unchecked(idx));
                        }
                        break;
                    }
                }
            }
            local_output.push(b'\n');
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let _ = handle.write_all(&local_output);
    });

    if let Some(start) = start_time {
        let duration = start.elapsed();
        let seconds = duration.as_secs_f64();
        let total_bytes = count * (length as u64 + 1);
        let mib_per_sec = (total_bytes as f64 / (1024.0 * 1024.0)) / seconds;
        let passwords_per_sec = count as f64 / seconds;

        eprintln!("\n--- СТАТИСТИКА ГЕНЕРАЦИИ ---");
        eprintln!("Время выполнения:    {:.3} сек", seconds);
        eprintln!("Скорость потока:     {:.2} MiB/s", mib_per_sec);
        eprintln!("Производительность:  {:.0} паролей/сек", passwords_per_sec);
        eprintln!("Всего данных:        {:.2} GiB", total_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
        eprintln!("----------------------------");
    }

    Ok(())
}

fn parse_args(args: &[String]) -> (usize, u64, bool) {
    let mut length = 16;
    let mut count = 1;
    let mut show_stats = false;
    let mut nums = Vec::new();

    for arg in args.iter().skip(1) {
        if arg == "-s" || arg == "--stats" {
            show_stats = true;
        } else if let Ok(n) = arg.parse::<u64>() {
            nums.push(n);
        }
    }

    if let Some(&l) = nums.get(0) { length = l as usize; }
    if let Some(&c) = nums.get(1) { count = c; }

    (if length == 0 { 1 } else { length }, count, show_stats)
}