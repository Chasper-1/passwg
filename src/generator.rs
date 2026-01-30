#![allow(dead_code)]
use crate::i18n::I18n;
use crate::words::WORDLIST;
use crate::writer::OutputFormat;
// Импортируем все варианты ChaCha
use rand_chacha::{ChaCha8Rng, ChaCha12Rng, ChaCha20Rng};
use rand_core::{RngCore, SeedableRng};
use std::time::Instant;

pub const CHARSET: &[u8] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&'()*+,-./:;<=>?@[]^_`{|}~";
pub const CHARSET_LEN: usize = 92;
pub const CHARSET_FAST: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";

const CHARSET_LIMIT: u32 = (u32::MAX / CHARSET_LEN as u32) * CHARSET_LEN as u32;

/// Публичная точка входа. Выбирает алгоритм на основе rounds и вызывает generic-функцию.
pub fn generate_chunk(
    start_id: u64,
    size: u64,
    length: usize,
    fast_mode: bool,
    word_mode: bool,
    format: OutputFormat,
    rounds: u8, // Новый параметр
) -> Vec<u8> {
    let mut seed = [0u8; 32];
    // Используем системную энтропию для инициализации
    let _ = getrandom::fill(&mut seed);

    match rounds {
        12 => generate_internal(
            ChaCha12Rng::from_seed(seed),
            start_id,
            size,
            length,
            fast_mode,
            word_mode,
            format,
        ),
        20 => generate_internal(
            ChaCha20Rng::from_seed(seed),
            start_id,
            size,
            length,
            fast_mode,
            word_mode,
            format,
        ),
        _ => generate_internal(
            ChaCha8Rng::from_seed(seed),
            start_id,
            size,
            length,
            fast_mode,
            word_mode,
            format,
        ),
    }
}

/// Внутренняя функция с логикой генерации.
/// <R: RngCore> означает, что она принимает любой генератор (8, 12 или 20 раундов),
/// и компилятор создаст для каждого отдельную оптимизированную версию кода.
fn generate_internal<R: RngCore>(
    mut rng: R,
    start_id: u64,
    size: u64,
    length: usize,
    fast_mode: bool,
    word_mode: bool,
    format: OutputFormat,
) -> Vec<u8> {
    // Резервируем память: длина пароля + макс. длина ID (20) + разделители
    let mut buf = Vec::with_capacity(size as usize * (length + 32));

    unsafe {
        let ptr: *mut u8 = buf.as_mut_ptr();
        let mut offset = 0;

        for i in 0..size {
            let current_id = start_id + i;

            // 1. ПРЕФИКСЫ ФОРМАТА
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

            // 2. ГЕНЕРАЦИЯ КОНТЕНТА
            if word_mode {
                for k in 0..length {
                    let random_u32 = rng.next_u32();
                    // Умножение вместо деления по модулю для скорости и равномерности
                    let idx = ((random_u32 as u64 * WORDLIST.len() as u64) >> 32) as usize;
                    let word = *WORDLIST.get_unchecked(idx);

                    std::ptr::copy_nonoverlapping(word.as_ptr(), ptr.add(offset), word.len());
                    offset += word.len();

                    if k < length - 1 {
                        *ptr.add(offset) = b'-';
                        offset += 1;
                    }
                }
            } else if fast_mode {
                let mut current_len = length;

                #[cfg(target_arch = "x86_64")]
                if is_x86_feature_detected!("avx2") && current_len >= 32 {
                    let chunks_32 = current_len / 32;
                    let mut rand_buf = [0u8; 32];
                    for _ in 0..chunks_32 {
                        rng.fill_bytes(&mut rand_buf);
                        // Убрал лишний unsafe внутри, так как мы уже в unsafe контексте
                        crate::avx2::Avx2Mapper::map_64_symbols(rand_buf.as_ptr(), ptr.add(offset));
                        offset += 32;
                        current_len -= 32;
                    }
                }

                if current_len >= 10 {
                    let chunks_10 = current_len / 10;
                    for _ in 0..chunks_10 {
                        let r = rng.next_u64();
                        *ptr.add(offset) = *CHARSET_FAST.get_unchecked((r & 63) as usize);
                        *ptr.add(offset + 1) =
                            *CHARSET_FAST.get_unchecked(((r >> 6) & 63) as usize);
                        *ptr.add(offset + 2) =
                            *CHARSET_FAST.get_unchecked(((r >> 12) & 63) as usize);
                        *ptr.add(offset + 3) =
                            *CHARSET_FAST.get_unchecked(((r >> 18) & 63) as usize);
                        *ptr.add(offset + 4) =
                            *CHARSET_FAST.get_unchecked(((r >> 24) & 63) as usize);
                        *ptr.add(offset + 5) =
                            *CHARSET_FAST.get_unchecked(((r >> 30) & 63) as usize);
                        *ptr.add(offset + 6) =
                            *CHARSET_FAST.get_unchecked(((r >> 36) & 63) as usize);
                        *ptr.add(offset + 7) =
                            *CHARSET_FAST.get_unchecked(((r >> 42) & 63) as usize);
                        *ptr.add(offset + 8) =
                            *CHARSET_FAST.get_unchecked(((r >> 48) & 63) as usize);
                        *ptr.add(offset + 9) =
                            *CHARSET_FAST.get_unchecked(((r >> 54) & 63) as usize);
                        offset += 10;
                        current_len -= 10;
                    }
                }

                if current_len > 0 {
                    let mut r = rng.next_u64();
                    for _ in 0..current_len {
                        *ptr.add(offset) = *CHARSET_FAST.get_unchecked((r & 63) as usize);
                        r >>= 6;
                        offset += 1;
                    }
                }
            } else {
                for _ in 0..length {
                    let mut r = rng.next_u32();
                    // Отсеивание (Rejection Sampling) для удаления Modulo Bias
                    if r >= CHARSET_LIMIT {
                        loop {
                            r = rng.next_u32();
                            if r < CHARSET_LIMIT {
                                break;
                            }
                        }
                    }
                    *ptr.add(offset) = *CHARSET.get_unchecked((r % CHARSET_LEN as u32) as usize);
                    offset += 1;
                }
            }

            // 3. ПОСТФИКСЫ
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

/// Супер-быстрая запись u64 через таблицу предзаписанных пар цифр
#[inline(always)]
unsafe fn fast_write_u64_ptr(ptr: *mut u8, mut n: u64) -> usize {
    static TABLE: &[u8] = b"0001020304050607080910111213141516171819\
                            2021222324252627282930313233343536373839\
                            4041424344454647484950515253545556575859\
                            6061626364656667686970717273747576777879\
                            8081828384858687888990919293949596979899";
    if n == 0 {
        unsafe {
            *ptr = b'0';
        }
        return 1;
    }
    let mut temp = [0u8; 20];
    let mut i = 20;
    while n >= 100 {
        let digit2 = ((n % 100) * 2) as usize;
        n /= 100;
        i -= 2;
        unsafe {
            temp[i] = *TABLE.get_unchecked(digit2);
            temp[i + 1] = *TABLE.get_unchecked(digit2 + 1);
        }
    }
    if n < 10 {
        i -= 1;
        temp[i] = b'0' + n as u8;
    } else {
        let digit2 = (n * 2) as usize;
        i -= 2;
        unsafe {
            temp[i] = *TABLE.get_unchecked(digit2);
            temp[i + 1] = *TABLE.get_unchecked(digit2 + 1);
        }
    }
    let len = 20 - i;
    unsafe {
        std::ptr::copy_nonoverlapping(temp.as_ptr().add(i), ptr, len);
    }
    len
}

pub fn copy_to_clipboard(pwd: &str) {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let _ = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .arg("-n")
        .spawn()
        .and_then(|mut c| c.stdin.take().unwrap().write_all(pwd.as_bytes()));
}

pub fn print_report(start: Instant, count: u64, _length: usize, l: &I18n) {
    let dur = start.elapsed().as_secs_f64();
    if dur > 0.0 {
        let speed = count as f64 / dur;
        eprintln!("\n--- {} ---", l.stat_title);
        eprintln!("{}: {:.4} s", l.stat_time, dur);
        eprintln!("{}: {:.2} p/s", l.stat_speed, speed);
        eprintln!("{}: {:.2} Mp/s", l.stat_perf, speed / 1_000_000.0);
    }
}
