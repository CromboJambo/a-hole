use anyhow::Result;
use csv::ReaderBuilder;
use std::io::Write;

pub fn csv_to_psv(path: &str, mut output: impl Write) -> Result<()> {
    let mut rdr = ReaderBuilder::new().from_path(path)?;

    let headers = rdr.headers()?;
    let header_line: Vec<String> = headers
        .iter()
        .map(|h| escape_pipe(&h.to_string()))
        .collect();
    writeln!(output, "{}", header_line.join("|"))?;

    for result in rdr.records() {
        let record = result?;
        let line: Vec<String> = record.iter().map(|v| escape_pipe(&v.to_string())).collect();
        writeln!(output, "{}", line.join("|"))?;
    }

    Ok(())
}

pub fn escape_pipe(value: &str) -> String {
    value.replace("|", "\\|")
}
