use std::fs::File;
use std::io::{self, BufWriter, Write};

#[derive(PartialEq, Clone, Copy)]
pub enum OutputFormat { Plain, Json, Csv }

/// Создает буферизированный поток вывода.
/// 1MB для stdout и 32MB для файла, чтобы реже дергать диск.
pub fn get_writer(out_file: &Option<String>) -> io::Result<Box<dyn Write + Send>> {
    if let Some(path) = out_file {
        let file = File::create(path)?;
        Ok(Box::new(BufWriter::with_capacity(32 * 1024 * 1024, file)))
    } else {
        Ok(Box::new(BufWriter::with_capacity(1024 * 1024, io::stdout())))
    }
}