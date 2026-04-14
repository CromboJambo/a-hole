use anyhow::Result;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Write};

/// Convert JSON files to PSV format
/// Expects JSON root to be an array of objects
pub fn json_to_psv(path: &str, mut output: impl Write) -> Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data: Value = serde_json::from_reader(reader)?;

    // Validate that JSON root is an array of objects
    if let Value::Array(arr) = data {
        if arr.is_empty() {
            anyhow::bail!("JSON array is empty");
        }

        // Extract headers from the first object
        if let Value::Object(first_obj) = &arr[0] {
            let headers: Vec<String> = first_obj.keys().cloned().collect();
            let header_line: Vec<String> = headers.iter().map(|h| escape_pipe(h)).collect();
            writeln!(output, "{}", header_line.join("|"))?;
        }

        // Process each object in the array
        for (idx, obj) in arr.iter().enumerate() {
            if let Value::Object(map) = obj {
                let line: Vec<String> = map.values().map(|v| escape_pipe(&v.to_string())).collect();
                writeln!(output, "{}", line.join("|"))?;
            } else {
                anyhow::bail!("JSON array element at index {} is not an object", idx);
            }
        }
    } else {
        anyhow::bail!("JSON root must be an array of objects");
    }

    Ok(())
}

/// Escape any pipes in the value for PSV format
pub fn escape_pipe(value: &str) -> String {
    value.replace("|", "\\|")
}
