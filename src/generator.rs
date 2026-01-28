use getrandom::fill;
use std::process::{Command, Stdio};
use std::io::Write;
use std::time::Instant;
use crate::i18n::I18n;

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

pub fn copy_to_clipboard(length: usize, fast_mode: bool) {
    let mut pwd = String::new();
    let mut rb = vec![0u8; length * 4];
    let _ = fill(&mut rb);
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

pub fn print_report(start: Instant, count: u64, length: usize, l: &I18n) {
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

        eprintln!("\n--- {} ---", l.stat_title);
        eprintln!("{:<20} {:.3} сек", l.stat_time, dur);
        eprintln!("{:<20} {:.2} MiB/s", l.stat_speed, mib_s);
        eprintln!("{:<20} {} пар/сек", l.stat_perf, format_number(p_s as u64));
        eprintln!("------------------");
    }
}