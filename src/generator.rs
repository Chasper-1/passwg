use crate::words::WORDLIST;
use crate::i18n::I18n;
use crate::writer::OutputFormat;
use std::time::Instant;

pub const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&'()*+,-./:;<=>?@[]^_`{|}~";
pub const CHARSET_LEN: usize = CHARSET.len();
pub const CHARSET_FAST: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";

pub fn generate_chunk(
    start_id: u64,
    size: u64,
    length: usize,
    fast_mode: bool,
    word_mode: bool,
    format: OutputFormat,
) -> Vec<u8> {
    // Резервируем под L1 кэш (~32KB чанк при 1024 паролях)
    let mut buf = Vec::with_capacity(size as usize * (length + 12));
    
    // Сид один раз на поток
    let mut seed = [0u8; 8];
    let _ = getrandom::fill(&mut seed);
    let mut state = u64::from_le_bytes(seed);

    for i in 0..size {
        let current_id = start_id + i;

        match format {
            OutputFormat::Csv => {
                fast_write_u64(&mut buf, current_id);
                buf.push(b',');
            }
            OutputFormat::Json => {
                if current_id == 1 { buf.extend_from_slice(b"  \""); }
                else { buf.extend_from_slice(b",\n  \""); }
            }
            _ => {}
        }

        if word_mode {
            for k in 0..length {
                state = next_rand(&mut state);
                let word_idx = (state as usize) % WORDLIST.len();
                buf.extend_from_slice(WORDLIST[word_idx].as_bytes());
                if k < length - 1 { buf.push(b'-'); }
            }
        } else {
            for _ in 0..length {
                state = next_rand(&mut state);
                let b = (state >> 32) as u8;
                if fast_mode {
                    buf.push(CHARSET_FAST[(b & 63) as usize]);
                } else {
                    buf.push(CHARSET[(b as usize) % CHARSET_LEN]);
                }
            }
        }

        if format == OutputFormat::Json { buf.push(b'\"'); }
        else { buf.push(b'\n'); }
    }
    buf
}

#[inline(always)]
fn next_rand(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    *state
}

pub fn fast_write_u64(buf: &mut Vec<u8>, mut n: u64) {
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

pub fn copy_to_clipboard(pwd: &str) {
    use std::process::{Command, Stdio};
    use std::io::Write;
    let mut child = match Command::new("wl-copy").stdin(Stdio::piped()).arg("-n").spawn() {
        Ok(c) => c,
        Err(_) => return,
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(pwd.as_bytes());
    }
    let _ = child.wait();
}

pub fn print_report(start: Instant, count: u64, _length: usize, l: &I18n) {
    let dur = start.elapsed().as_secs_f64();
    if dur > 0.0 {
        let speed = count as f64 / dur;
        println!("\n--- {} ---", l.stat_title);
        println!("{}: {:.4} s", l.stat_time, dur);
        println!("{}: {:.2} p/s", l.stat_speed, speed);
        // Используем то самое поле stat_perf, чтобы не было ворнингов
        println!("{}: {:.2} MHz (approx)", l.stat_perf, speed / 1_000_000.0);
    }
}