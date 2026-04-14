# Format to PSV Tool

A Rust-based tool for converting various file formats (XLSX, CSV, JSON) to PSV (pipe-separated values) format, designed for seamless integration with CrabJar and Nushell pipelines.

## Philosophy

This tool embodies the **CrabJar philosophy**: memory-safe, deterministic, and focused on structured data. PSV (pipe-separated values) is the universal language of structured data in the Rust ecosystem - it's easier to parse than CSV, avoids quoting quirks, and works perfectly with Nushell's pipeline model.

### Why PSV?

- **Consistent structure**: One format for all data sources
- **Simpler parsing**: No escaping rules like CSV's quotes
- **Nushell-friendly**: Native support for pipe-delimited data
- **Memory-safe**: Pure Rust implementation
- **Deterministic**: Same input → same output

## Installation

```bash
cargo install --path tools/format-to-psv
```

Or build it yourself:

```bash
cd tools/format-to-psv
cargo build --release
```

## Usage

### Basic Usage

```bash
format-to-psv <input_file> [output_file]
```

### Examples

**Convert Excel to PSV and save to file:**
```bash
format-to-psv data.xlsx output.psv
```

**Convert CSV to PSV and output to stdout:**
```bash
format-to-psv data.csv
```

**Convert JSON to PSV:**
```bash
format-to-psv data.json output.psv
```

## Supported Formats

### XLSX / XLS
- Reads Excel spreadsheets
- Uses the first sheet in the workbook
- Preserves all data including formulas
- Memory-safe parsing via calamine

### CSV
- Reads CSV files with any delimiter
- Handles quoted fields correctly
- Preserves all data
- Uses the csv crate for robust parsing

### JSON
- Expects JSON root to be an array of objects
- Automatically extracts headers from the first object
- Processes all objects in the array
- Type-safe parsing via serde_json

## Output Format

The tool outputs data in PSV (pipe-separated values) format:

```
column1|column2|column3
value1|value2|value3
value4|value5|value6
```

### Pipe Escaping

Pipes inside values are escaped with backslashes:

```
name|email|address
John Doe|john@example|123 Main St
```

Becomes:

```
name|email|address
John Doe|john\|example|123 Main St
```

### Nushell Integration

The PSV format works seamlessly with Nushell:

```nushell
open output.psv | get 'column name' | each {|v| if $v == "" {0.0} else { $v | into float } } | math sum
```

Or for more complex operations:

```nushell
open output.psv | 
  where 'total cost' > 100 | 
  select 'product name' 'total cost' | 
  sort-by 'total cost'
```

## Project Structure

```
format-to-psv/
├── Cargo.toml          # Project dependencies and metadata
├── README.md           # This file
└── src/
    ├── main.rs         # Entry point and CLI logic
    ├── csv_unit.rs     # CSV to PSV conversion module
    ├── json_unit.rs    # JSON to PSV conversion module
    └── xlsx_unit.rs    # XLSX to PSV conversion module
```

## Module Details

### main.rs
Entry point that handles command-line arguments and dispatches to appropriate conversion module based on file extension.

### csv_unit.rs
Handles CSV file parsing and conversion to PSV format using the `csv` crate with proper error handling.

### json_unit.rs
Handles JSON file parsing and conversion to PSV format, expecting an array of objects.

### xlsx_unit.rs
Handles Excel file parsing and conversion to PSV format using the `calamine` crate.

## Dependencies

- `calamine` 0.23 - Excel file reading
- `csv` 1.3 - CSV file reading
- `serde_json` 1.0 - JSON file reading
- `anyhow` 1.0 - Error handling

## Development

To add support for additional formats:

1. Create a new module (e.g., `tsv_unit.rs`)
2. Implement the conversion function with the same signature
3. Add file extension detection in `main.rs`
4. Export the function for reuse

### Adding New Format Support

The modular design makes it easy to extend:

```rust
// Example: TSV support
pub fn tsv_to_psv(path: &str, mut output: impl Write) -> Result<()> {
    // Implementation
}
```

## Testing

Run the test suite:

```bash
cargo test --manifest-path tools/format-to-psv/Cargo.toml
```

## Integration with CrabJar

This tool is part of the CrabJar ecosystem, designed to work with:

- **Nushell pipelines**: PSV is the native format for structured data
- **Memory safety**: Pure Rust implementation
- **Deterministic behavior**: Same input always produces same output
- **Sandboxing**: Can be run in capability-constrained environments

### Capability Sandbox Integration

The tool follows CrabJar's capability sandbox principles:

- **No shell escape**: All operations are within Rust
- **Deterministic tool surface**: Predictable input/output
- **Resource limits**: Can be run with ulimits or in containers
- **Transparent**: Pure Rust, no external dependencies beyond crates

## License

This project is part of the CrabJar ecosystem. See the main project license for details.

## Contributing

This tool is designed to be modular and extensible. Feel free to add new format support or enhance existing functionality. Follow these principles:

1. Keep it memory-safe
2. Maintain deterministic behavior
3. Preserve PSV format consistency
4. Add tests for new features
5. Document any breaking changes

## Future Enhancements

Potential improvements for the next iteration:

- **Auto-detect numeric columns** → coerce to floats for downstream math
- **Multiple sheet support** → allow selecting specific sheets
- **Streaming large files** → avoid loading everything into memory
- **Custom delimiters** → support other separators if needed
- **Type inference** → automatically detect column types
- **Schema validation** → ensure data integrity

## The "AI in a Jar" Approach

This tool exemplifies the "AI in a Jar" philosophy:

- **Focused**: Does one thing well (format conversion)
- **Transparent**: Pure Rust, no black boxes
- **Safe**: Memory-safe and deterministic
- **Portable**: Works in any Rust environment
- **Extensible**: Modular design for easy extension

By converting all formats to PSV, we create a **universal table abstraction** that works across the entire CrabJar ecosystem, from local agents to remote inference.

## Quick Start

```bash
# Install
cargo install --path tools/format-to-psv

# Convert a file
format-to-psv data.xlsx output.psv

# Use in Nushell
open output.psv | get 'column' | math sum
```

## Troubleshooting

### "Unsupported file type"
Make sure your file has a valid extension (.xlsx, .xls, .csv, or .json).

### "JSON root must be an array of objects"
The JSON file must have an array at the root level, like `[{"col1": "val1", "col2": "val2"}]`.

### Memory issues with large files
The current implementation loads files into memory. For very large files, consider streaming or file-backed approaches.

## References

- [Rust-based Autonomous Agent](../Rust-based%20Autonomous%20Agent.md) - High-level architecture and philosophy
- [CrabJar Project](../README.md) - Main project documentation
- [Nushell Documentation](https://www.nushell.sh/) - For PSV integration examples

## Summary

The format-to-psv tool provides a **universal table abstraction** for the CrabJar ecosystem. By converting XLSX, CSV, and JSON to a consistent PSV format, we enable:

1. **Unified data access**: One format for all data sources
2. **Simple pipelines**: Easy to work with in Nushell
3. **Memory safety**: Pure Rust implementation
4. **Extensibility**: Easy to add new format support
5. **Deterministic behavior**: Predictable and reliable

This tool embodies the CrabJar principles of **simplicity, safety, and universality** in data handling.