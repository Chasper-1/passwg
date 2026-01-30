use crate::i18n::I18n;
use crate::writer::OutputFormat;

pub struct Config {
    pub length: usize,
    pub count: u64,
    pub rounds: u8,
    pub show_stats: bool,
    pub fast_mode: bool,
    pub copy_mode: bool,
    pub word_mode: bool,
    pub out_file: Option<String>,
    pub format: OutputFormat,
}

pub fn parse_args(args: &[String]) -> Config {
    let mut length = 16;
    let mut count = 1;
    let mut rounds = 8;
    let mut show_stats = false;
    let mut fast_mode = false;
    let mut copy_mode = false;
    let mut word_mode = false;
    let mut out_file = None;
    let mut format = OutputFormat::Plain;
    let mut nums = Vec::new();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--stats" => show_stats = true,
            "-f" | "--fast" => fast_mode = true,
            "-c" | "--copy" => copy_mode = true,
            "-w" | "--words" => {
                word_mode = true;
                if length == 16 {
                    length = 4;
                } // Дефолт для фраз — 4 слова
            }
            "--json" => format = OutputFormat::Json,
            "--csv" => format = OutputFormat::Csv,
            "-o" => {
                if i + 1 < args.len() {
                    out_file = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Ошибка: флаг -o требует указания имени файла");
                    eprintln!("Пример: passwg -o passwords.txt");
                    std::process::exit(1);
                }
            }
            "-h" | "--help" => {
                // help уже обработан в main, но на всякий случай
                std::process::exit(0);
            }
            "-r" | "--rounds" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u8>() {
                        Ok(r) if r == 8 || r == 12 || r == 20 => {
                            rounds = r;
                            i += 1;
                        }
                        _ => {
                            eprintln!(
                                "Ошибка: неверное количество раундов. Допустимо только: 8, 12, 20"
                            );
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Ошибка: флаг -r требует указания числа (8, 12, 20)");
                    std::process::exit(1);
                }
            }
            arg if arg.starts_with('-') => {
                // Неизвестный флаг
                eprintln!("Ошибка: неизвестный флаг '{}'", arg);
                eprintln!("Используйте -h для просмотра доступных флагов");
                std::process::exit(1);
            }
            _ => {
                if let Ok(n) = args[i].parse::<u64>() {
                    nums.push(n);
                } else {
                    eprintln!("Ошибка: неверный аргумент '{}'", args[i]);
                    eprintln!("Аргументы должны быть числами или флагами");
                    eprintln!("Пример: passwg 20 5 -s");
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }

    // Проверяем конфликт флагов
    if copy_mode && out_file.is_some() {
        eprintln!("Предупреждение: флаг -c (копирование) игнорируется при использовании -o (файл)");
        copy_mode = false;
    }

    if fast_mode && word_mode {
        eprintln!(
            "Предупреждение: флаг -f (быстрый режим) игнорируется при использовании -w (слова)"
        );
    }

    if let Some(&l) = nums.get(0) {
        if l == 0 {
            eprintln!("Ошибка: длина не может быть 0");
            std::process::exit(1);
        }
        length = l as usize;
    }

    if let Some(&c) = nums.get(1) {
        if c == 0 {
            eprintln!("Ошибка: количество не может быть 0");
            std::process::exit(1);
        }
        count = c;
    }

    // Если включен режим слов, проверяем длину
    if word_mode && length > 20 {
        eprintln!(
            "Предупреждение: количество слов слишком большое ({})",
            length
        );
        eprintln!("Рекомендуется не более 10 слов для удобства");
    }

    Config {
        length: if length == 0 { 1 } else { length },
        count,
        rounds,
        show_stats,
        fast_mode,
        copy_mode,
        word_mode,
        out_file,
        format,
    }
}

pub fn print_help(l: &I18n, app_name: &str, version: &str) {
    println!("{} v{}\n", app_name, version);
    println!("{}", l.help_usage);
    println!("\n{}", l.help_args);
    println!("{}", l.help_len);
    println!("{}", l.help_count);
    println!("\n{}", l.help_flags);
    println!("{}", l.help_out);
    println!("{}", l.help_json);
    println!("{}", l.help_csv);
    println!(
        "  -w, --words    {}",
        if l.help_usage.contains("Использование") {
            "Режим фраз (длина = количество слов)"
        } else {
            "Passphrase mode (length = number of words)"
        }
    );
    println!("{}", l.help_stats);
    println!("{}", l.help_fast);
    println!("{}", l.help_copy);
    println!("{}", l.help_rounds);
    println!("{}", l.help_h);
}
