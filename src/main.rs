use md_parser::Parser;
use std::env;
use std::fs;

const OUTPUT_DIR: &str = "output";

fn read_input_file(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    fs::read_to_string(file_path)
        .map_err(|e| format!("Error reading file '{}': {}", file_path, e).into())
}

fn ensure_output_dir() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(OUTPUT_DIR)
        .map_err(|e| format!("Error creating output dir '{}': {}", OUTPUT_DIR, e).into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.md>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];
    let markdown = read_input_file(file_path)?;

    let parser = Parser::new(markdown)?;
    let ast = parser.parse()?;

    println!("Parsed AST (Phase 3 - Debug Format):");
    println!("====================================\n");
    for (i, node) in ast.iter().enumerate() {
        println!("  {}: {:?}\n", i, node);
    }

    println!("\n\nParsed AST (JSON Format):");
    println!("==========================\n");
    let json = parser.to_json()?;
    println!("{}", json);

    println!("\n\nGenerating HTML file...");
    ensure_output_dir()?;
    parser.to_html_file("output.html")?;
    println!("✓ HTML file generated successfully: {}/output.html", OUTPUT_DIR);

    Ok(())
}
