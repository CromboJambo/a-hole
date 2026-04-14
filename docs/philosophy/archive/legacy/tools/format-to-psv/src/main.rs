use std::env;
use std::io::{self, Write};

mod csv_unit;
mod json_unit;
mod xlsx_unit;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        anyhow::bail!("Usage: format-to-psv <input_file> [output_file]");
    }

    let input_path = &args[1];
    let output_path = if args.len() > 2 { Some(&args[2]) } else { None };

    let output = if let Some(path) = output_path {
        Box::new(FileWithHeader::new(path)?) as Box<dyn Write>
    } else {
        Box::new(io::stdout())
    };

    if input_path.ends_with(".xlsx") || input_path.ends_with(".xls") {
        xlsx_unit::xlsx_to_psv(input_path, output)?;
    } else if input_path.ends_with(".csv") {
        csv_unit::csv_to_psv(input_path, output)?;
    } else if input_path.ends_with(".json") {
        json_unit::json_to_psv(input_path, output)?;
    } else {
        anyhow::bail!("Unsupported file type. Please use .xlsx, .xls, .csv, or .json");
    }

    Ok(())
}

struct FileWithHeader(std::fs::File);

impl FileWithHeader {
    fn new(path: &str) -> io::Result<Self> {
        Ok(FileWithHeader(std::fs::File::create(path)?))
    }
}

impl Write for FileWithHeader {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
