use std::fs;
use std::io::Write;
use std::path::PathBuf;

mod csv_unit;
mod json_unit;
mod main;
mod xlsx_unit;

fn create_test_file(path: &str, content: &str) {
    fs::write(path, content).expect("Failed to create test file");
}

fn read_test_file(path: &str) -> String {
    fs::read_to_string(path).expect("Failed to read test file")
}

#[test]
fn test_csv_to_psv_basic() {
    let test_csv = "name,age,city\nAlice,30,New York\nBob,25,Chicago";
    let test_file = "/tmp/test_format_to_psv.csv";
    let output_file = "/tmp/test_format_to_psv_output.psv";

    create_test_file(test_file, test_csv);

    let mut output = fs::File::create(output_file).expect("Failed to create output file");
    csv_unit::csv_to_psv(test_file, &mut output).expect("CSV to PSV conversion failed");

    let output_content = read_test_file(output_file);
    let expected = "name|age|city\nAlice|30|New York\nBob|25|Chicago";

    assert_eq!(output_content, expected);

    fs::remove_file(test_file).ok();
    fs::remove_file(output_file).ok();
}

#[test]
fn test_json_to_psv_basic() {
    let test_json = r#"[
        {"name": "Alice", "age": "30", "city": "New York"},
        {"name": "Bob", "age": "25", "city": "Chicago"}
    ]"#;
    let test_file = "/tmp/test_format_to_psv.json";
    let output_file = "/tmp/test_format_to_psv_output.psv";

    create_test_file(test_file, test_json);

    let mut output = fs::File::create(output_file).expect("Failed to create output file");
    json_unit::json_to_psv(test_file, &mut output).expect("JSON to PSV conversion failed");

    let output_content = read_test_file(output_file);
    let expected = "name|age|city\nAlice|30|New York\nBob|25|Chicago";

    assert_eq!(output_content, expected);

    fs::remove_file(test_file).ok();
    fs::remove_file(output_file).ok();
}

#[test]
fn test_pipe_escaping() {
    let test_csv = "name|email|address\nJohn Doe|john@example|123 Main St";
    let test_file = "/tmp/test_format_to_psv_escaped.csv";
    let output_file = "/tmp/test_format_to_psv_escaped_output.psv";

    create_test_file(test_file, test_csv);

    let mut output = fs::File::create(output_file).expect("Failed to create output file");
    csv_unit::csv_to_psv(test_file, &mut output).expect("CSV to PSV conversion failed");

    let output_content = read_test_file(output_file);
    let expected = "name|email|address\nJohn Doe|john\\|example|123 Main St";

    assert_eq!(output_content, expected);

    fs::remove_file(test_file).ok();
    fs::remove_file(output_file).ok();
}

#[test]
fn test_empty_csv() {
    let test_csv = "name,age,city";
    let test_file = "/tmp/test_format_to_psv_empty.csv";
    let output_file = "/tmp/test_format_to_psv_empty_output.psv";

    create_test_file(test_file, test_csv);

    let mut output = fs::File::create(output_file).expect("Failed to create output file");
    csv_unit::csv_to_psv(test_file, &mut output).expect("CSV to PSV conversion failed");

    let output_content = read_test_file(output_file);
    let expected = "name|age|city";

    assert_eq!(output_content, expected);

    fs::remove_file(test_file).ok();
    fs::remove_file(output_file).ok();
}

#[test]
fn test_empty_json() {
    let test_json = "[]";
    let test_file = "/tmp/test_format_to_psv_empty.json";
    let output_file = "/tmp/test_format_to_psv_empty_output.psv";

    create_test_file(test_file, test_json);

    let mut output = fs::File::create(output_file).expect("Failed to create output file");
    json_unit::json_to_psv(test_file, &mut output).expect("JSON to PSV conversion failed");

    let output_content = read_test_file(output_file);
    let expected = "";

    assert_eq!(output_content, expected);

    fs::remove_file(test_file).ok();
    fs::remove_file(output_file).ok();
}
