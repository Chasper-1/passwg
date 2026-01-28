use crate::writer::OutputFormat;
use crate::i18n::I18n;

pub struct Config {
    pub length: usize,
    pub count: u64,
    pub show_stats: bool,
    pub fast_mode: bool,
    pub copy_mode: bool,
    pub out_file: Option<String>,
    pub format: OutputFormat,
}

pub fn parse_args(args: &[String]) -> Config {
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
    
    Config {
        length: if length == 0 { 1 } else { length },
        count, show_stats, fast_mode, copy_mode, out_file, format
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
    println!("{}", l.help_stats);
    println!("{}", l.help_fast);
    println!("{}", l.help_copy);
    println!("{}", l.help_h);
}