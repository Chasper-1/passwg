use crate::words::WORDLIST;
use crate::i18n::I18n;
use getrandom::fill;
use std::time::Instant;

pub const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&'()*+,-./:;<=>?@[]^_`{|}~";
pub const CHARSET_LEN: usize = CHARSET.len();
pub const LIMIT: usize = (256 / CHARSET_LEN) * CHARSET_LEN;
pub const CHARSET_FAST: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";

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

pub fn get_random_word(rnd_buf: &mut [u8], rnd_pos: &mut usize) -> &'static str {
    if *rnd_pos + 1 >= rnd_buf.len() {
        let _ = fill(rnd_buf);
        *rnd_pos = 0;
    }
    let idx = ((rnd_buf[*rnd_pos] as usize) << 8 | (rnd_buf[*rnd_pos + 1] as usize)) % WORDLIST.len();
    *rnd_pos += 2;
    WORDLIST[idx]
}

pub fn copy_to_clipboard(pwd: &str) {
    use std::process::{Command, Stdio};
    use std::io::Write;

    let mut child = match Command::new("wl-copy")
        .stdin(Stdio::piped())
        .arg("-n")
        .spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Ошибка запуска wl-copy: {}", e);
            return;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(pwd.as_bytes()) {
            eprintln!("Ошибка записи в wl-copy: {}", e);
            return;
        }
    }
    
    match child.wait() {
        Ok(status) => {
            if status.success() {
                println!("✓ Скопировано в буфер обмена");
            } else {
                eprintln!("✗ wl-copy завершился с ошибкой");
            }
        }
        Err(e) => {
            eprintln!("✗ Ошибка ожидания wl-copy: {}", e);
        }
    }
}

pub fn print_report(start: Instant, count: u64, length: usize, l: &I18n) {
    let dur = start.elapsed().as_secs_f64();
    if dur > 0.0 {
        let bytes = count * (length as u64 + 1);
        let mib_s = (bytes as f64 / 1048576.0) / dur;
        let p_s = count as f64 / dur;
        println!("\n--- {} ---", l.stat_title);
        println!("{}: {:.4}s", l.stat_time, dur);
        println!("{}: {:.2} MiB/s", l.stat_speed, mib_s);
        println!("{}: {:.0} p/s", l.stat_perf, p_s);
    }
}