use getrandom::fill;
use std::io::{self, Write};

const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
const CHARSET_LEN: usize = CHARSET.len(); // 94
const LIMIT: usize = (256 / CHARSET_LEN) * CHARSET_LEN; // 188

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let (length, count) = parse_args(&args);

    let mut stdout = io::BufWriter::new(io::stdout());
    
    // Буфер 16КБ для баланса между системными вызовами и L1 кэшем
    let mut rnd_buf = [0u8; 16384];
    let mut rnd_pos = rnd_buf.len();

    for _ in 0..count {
        for _ in 0..length {
            loop {
                if rnd_pos >= rnd_buf.len() {
                    // Используем актуальный fill для версии 0.3+
                    fill(&mut rnd_buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                    rnd_pos = 0;
                }

                let val = rnd_buf[rnd_pos] as usize;
                rnd_pos += 1;

                if val < LIMIT {
                    let idx = val % CHARSET_LEN;
                    // unsafe убирает проверку границ (bounds check), ускоряя цикл
                    unsafe {
                        stdout.write_all(CHARSET.get_unchecked(idx..idx + 1))?;
                    }
                    break;
                }
            }
        }
        stdout.write_all(b"\n")?;
    }

    stdout.flush()?;
    Ok(())
}

fn parse_args(args: &[String]) -> (usize, u64) {
    let length = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(16);
    let count = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    
    (if length == 0 { 1 } else { length }, count)
}