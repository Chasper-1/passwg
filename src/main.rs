use getrandom::fill;
use rayon::prelude::*;
use std::io::{self, Write};

const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
const CHARSET_LEN: usize = CHARSET.len();
const LIMIT: usize = (256 / CHARSET_LEN) * CHARSET_LEN;

// Размер чанка (сколько паролей одно ядро генерирует за раз)
const CHUNK_SIZE: u64 = 1000; 

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let (length, count) = parse_args(&args);

    // Рассчитываем количество полных чанков и остаток
    let num_chunks = count / CHUNK_SIZE;
    let remainder = count % CHUNK_SIZE;

    // Создаем итератор по чанкам и запускаем его параллельно
    (0..=num_chunks).into_par_iter().for_each(|chunk_idx| {
        let current_chunk_size = if chunk_idx < num_chunks {
            CHUNK_SIZE
        } else {
            remainder
        };

        if current_chunk_size == 0 { return; }

        // Локальный буфер для одного ядра, чтобы не блокировать stdout постоянно
        // Резервируем память: (длина пароля + \n) * количество паролей в чанке
        let mut local_output = Vec::with_capacity(current_chunk_size as usize * (length + 1));
        
        // Локальный буфер случайных чисел для ядра (L1 кэш)
        let mut rnd_buf = [0u8; 16384];
        let mut rnd_pos = rnd_buf.len();

        for _ in 0..current_chunk_size {
            for _ in 0..length {
                loop {
                    if rnd_pos >= rnd_buf.len() {
                        let _ = fill(&mut rnd_buf); // В параллели игнорируем мелкие ошибки для скорости
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

        // Блокируем stdout только для вывода готовой пачки данных
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let _ = handle.write_all(&local_output);
    });

    Ok(())
}

fn parse_args(args: &[String]) -> (usize, u64) {
    let length = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(16);
    let count = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    (if length == 0 { 1 } else { length }, count)
}