use anyhow::Result;
use calamine::{open_workbook_auto, Reader};
use std::io::Write;

/// Convert XLSX/Excel files to PSV format
pub fn xlsx_to_psv(path: &str, mut output: impl Write) -> Result<()> {
    let mut workbook = open_workbook_auto(path)?;
    let sheet_names = workbook.sheet_names().to_owned();

    if sheet_names.is_empty() {
        anyhow::bail!("No sheets found in workbook");
    }

    // Use the first sheet for conversion
    let sheet_name = &sheet_names[0];
    let range = workbook.worksheet_range(sheet_name)?;

    // Process header row
    if let Some(header_row) = range.rows().next() {
        let headers: Vec<String> = header_row
            .iter()
            .map(|c| escape_pipe(&c.to_string()))
            .collect();
        writeln!(output, "{}", headers.join("|"))?;
    }

    // Process data rows
    for row in range.rows().skip(1) {
        let line: Vec<String> = row.iter().map(|c| escape_pipe(&c.to_string())).collect();
        writeln!(output, "{}", line.join("|"))?;
    }

    Ok(())
}

/// Escape any pipes in the value for PSV format
pub fn escape_pipe(value: &str) -> String {
    value.replace("|", "\\|")
}
