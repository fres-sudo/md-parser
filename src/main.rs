use md_parser::Parser;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

const OUTPUT_DIR: &str = "output";

/// Read the input markdown file
///
/// # Errors
///
/// Returns an error if the file cannot be read
fn read_input_file(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    fs::read_to_string(file_path)
        .map_err(|e| format!("Error reading file '{}': {}", file_path, e).into())
}

/// Ensure the output directory exists
///
/// # Errors
///
/// Returns an error if the directory cannot be created
fn ensure_output_dir() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(OUTPUT_DIR)
        .map_err(|e| format!("Error creating output dir '{}': {}", OUTPUT_DIR, e).into())
}

/// Write the AST in debug format to a file in `output/`
///
/// # Errors
///
/// Returns an error if file writing fails
fn write_ast_debug(ast: &[md_parser::Node]) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(OUTPUT_DIR).join("ast.txt");
    let mut f = fs::File::create(&path)
        .map_err(|e| format!("Error creating '{}': {}", path.display(), e))?;
    writeln!(f, "Parsed AST (Debug Format):")?;
    writeln!(f, "==========================\n")?;
    for (i, node) in ast.iter().enumerate() {
        writeln!(f, "  {}: {:?}\n", i, node)?;
    }
    Ok(())
}

/// Write the AST in JSON format to a file in `output/`
///
/// # Errors
///
/// Returns an error if JSON serialization or file writing fails
fn write_ast_json(parser: &mut Parser) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(OUTPUT_DIR).join("ast.json");
    let json = parser.to_json()?;
    fs::write(&path, json).map_err(|e| {
        let msg = format!("Error writing '{}': {}", path.display(), e);
        Box::<dyn std::error::Error>::from(msg)
    })?;
    Ok(())
}

/// Generate HTML output file in `output/`
///
/// # Errors
///
/// Returns an error if HTML generation or file writing fails
fn write_html_output(parser: &mut Parser) -> Result<(), Box<dyn std::error::Error>> {
    parser.to_html_file("output.html")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.md>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];
    let markdown = read_input_file(file_path)?;

    let mut parser = Parser::new(markdown)?;
    let ast = parser.parse()?;

    // Check for warnings and display them
    let warnings = parser.warnings();
    if !warnings.is_empty() {
        for warning in warnings {
            eprintln!("Warning: {}", warning);
        }
    }

    ensure_output_dir()?;
    write_ast_debug(&ast)?;
    write_ast_json(&mut parser)?;
    write_html_output(&mut parser)?;

    println!(
        "Wrote {}/ast.txt, {}/ast.json, {}/output.html",
        OUTPUT_DIR, OUTPUT_DIR, OUTPUT_DIR
    );

    Ok(())
}
