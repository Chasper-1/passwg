use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::sync::mpsc::Receiver;

#[derive(PartialEq, Clone, Copy)]
pub enum OutputFormat { Plain, Json, Csv }

pub fn start_writer(rx: Receiver<Vec<u8>>, out_file: Option<String>, format: OutputFormat) -> io::Result<()> {
    let mut out: Box<dyn Write> = if let Some(path) = out_file {
        Box::new(BufWriter::with_capacity(32 * 1024 * 1024, File::create(path)?))
    } else {
        Box::new(BufWriter::with_capacity(1024 * 1024, io::stdout()))
    };

    match format {
        OutputFormat::Csv => writeln!(out, "id,password")?,
        OutputFormat::Json => write!(out, "[\n")?,
        _ => {}
    }

    for received in rx {
        out.write_all(&received)?;
    }

    if format == OutputFormat::Json {
        write!(out, "\n]")?;
    }
    out.flush()?;
    Ok(())
}