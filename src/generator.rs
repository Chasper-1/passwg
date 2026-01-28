use crate::words::WORDLIST;
use crate::i18n::I18n;
use crate::writer::OutputFormat;
use std::time::Instant;
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};

pub const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&'()*+,-./:;<=>?@[]^_`{|}~";
pub const CHARSET_LEN: usize = 92; 
pub const CHARSET_FAST: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
const CHARSET_LIMIT: u32 = (256 / CHARSET_LEN as u32) * CHARSET_LEN as u32;

pub fn generate_chunk(
    start_id: u64,
    size: u64,
    length: usize,
    fast_mode: bool,
    word_mode: bool,
    format: OutputFormat,
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(size as usize * (length + 45));
    
    let mut seed = [0u8; 32];
    let _ = getrandom::fill(&mut seed);
    let mut rng = ChaCha8Rng::from_seed(seed);

    unsafe {
        let ptr: *mut u8 = buf.as_mut_ptr();
        let mut offset = 0;

        for i in 0..size {
            let current_id = start_id + i;

            match format {
                OutputFormat::Csv => {
                    offset += fast_write_u64_ptr(ptr.add(offset), current_id);
                    *ptr.add(offset) = b',';
                    offset += 1;
                }
                OutputFormat::Json => {
                    let prefix: &[u8] = if current_id == 1 { b"  \"" } else { b",\n  \"" };
                    std::ptr::copy_nonoverlapping(prefix.as_ptr(), ptr.add(offset), prefix.len());
                    offset += prefix.len();
                }
                _ => {}
            }

            if word_mode {
                for _ in 0..length {
                    let idx = (rng.next_u32() as usize) % WORDLIST.len();
                    let word = WORDLIST[idx];
                    std::ptr::copy_nonoverlapping(word.as_ptr(), ptr.add(offset), word.len());
                    offset += word.len();
                    *ptr.add(offset) = b'-'; // Упростили логику тире для скорости
                    offset += 1;
                }
                offset -= 1; // Убираем последнее тире
            } else {
                let target_charset = if fast_mode { CHARSET_FAST } else { CHARSET };
                
                // КЛЮЧЕВАЯ ОПТИМИЗАЦИЯ: читаем сразу 8 байт
                let mut random_val = rng.next_u64();
                let mut available_bytes = 8;

                for _ in 0..length {
                    if available_bytes == 0 {
                        random_val = rng.next_u64();
                        available_bytes = 8;
                    }

                    if fast_mode {
                        *ptr.add(offset) = target_charset[(random_val & 63) as usize];
                        random_val >>= 6;
                        // В fast mode 64 бита хватает на 10 символов
                        if (64 - (available_bytes * 6)) > 58 { available_bytes = 0; } 
                    } else {
                        let mut r = (random_val & 0xFF) as u32;
                        if r >= CHARSET_LIMIT {
                            loop {
                                r = rng.next_u32() & 0xFF;
                                if r < CHARSET_LIMIT { break; }
                            }
                        }
                        *ptr.add(offset) = target_charset[r as usize % 92];
                        random_val >>= 8;
                        available_bytes -= 1;
                    }
                    offset += 1;
                }
            }

            if format == OutputFormat::Json {
                *ptr.add(offset) = b'\"';
                offset += 1;
            } else {
                *ptr.add(offset) = b'\n';
                offset += 1;
            }
        }
        buf.set_len(offset);
    }
    buf
}

#[inline(always)]
unsafe fn fast_write_u64_ptr(ptr: *mut u8, mut n: u64) -> usize {
    if n == 0 { unsafe { *ptr = b'0'; } return 1; }
    static TABLE: &[u8; 200] = b"0001020304050607080910111213141516171819\
                                 2021222324252627282930313233343536373839\
                                 4041424344454647484950515253545556575859\
                                 6061626364656667686970717273747576777879\
                                 8081828384858687888990919293949596979899";
    let mut temp = [0u8; 20];
    let mut i = 20;
    while n >= 100 {
        let tri = ((n % 100) * 2) as usize;
        n /= 100;
        i -= 2;
        unsafe {
            temp[i] = *TABLE.get_unchecked(tri);
            temp[i + 1] = *TABLE.get_unchecked(tri + 1);
        }
    }
    if n < 10 {
        i -= 1;
        temp[i] = b'0' + n as u8;
    } else {
        let tri = (n * 2) as usize;
        i -= 2;
        unsafe {
            temp[i] = *TABLE.get_unchecked(tri);
            temp[i + 1] = *TABLE.get_unchecked(tri + 1);
        }
    }
    let len = 20 - i;
    unsafe { std::ptr::copy_nonoverlapping(temp.as_ptr().add(i), ptr, len); }
    len
}

pub fn copy_to_clipboard(pwd: &str) {
    use std::process::{Command, Stdio};
    use std::io::Write;
    let _ = Command::new("wl-copy").stdin(Stdio::piped()).arg("-n").spawn()
        .and_then(|mut c| c.stdin.take().unwrap().write_all(pwd.as_bytes()));
}

pub fn print_report(start: Instant, count: u64, _length: usize, l: &I18n) {
    let dur = start.elapsed().as_secs_f64();
    if dur > 0.0 {
        let speed = count as f64 / dur;
        println!("\n--- {} ---", l.stat_title);
        println!("{}: {:.4} s", l.stat_time, dur);
        println!("{}: {:.2} p/s", l.stat_speed, speed);
        println!("{}: {:.2} Mp/s", l.stat_perf, speed / 1_000_000.0);
    }
}