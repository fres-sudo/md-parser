use md_parser::{Config, Parser};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

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
fn ensure_output_dir(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("Error creating output dir '{}': {}", output_dir, e).into())
}

/// Write the AST in debug format to a file
///
/// # Errors
///
/// Returns an error if file writing fails
fn write_ast_debug(
    ast: &[md_parser::Node],
    output_dir: &str,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(output_dir).join(filename);
    let mut f = fs::File::create(&path)
        .map_err(|e| format!("Error creating '{}': {}", path.display(), e))?;
    writeln!(f, "Parsed AST (Debug Format):")?;
    writeln!(f, "==========================\n")?;
    for (i, node) in ast.iter().enumerate() {
        writeln!(f, "  {}: {:?}\n", i, node)?;
    }
    Ok(())
}

/// Write the AST in JSON format to a file
///
/// # Errors
///
/// Returns an error if JSON serialization or file writing fails
fn write_ast_json(
    parser: &mut Parser,
    output_dir: &str,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(output_dir).join(filename);
    let json = parser.to_json()?;
    fs::write(&path, json).map_err(|e| {
        let msg = format!("Error writing '{}': {}", path.display(), e);
        Box::<dyn std::error::Error>::from(msg)
    })?;
    Ok(())
}

/// Generate HTML output file
///
/// # Errors
///
/// Returns an error if HTML generation or file writing fails
fn write_html_output(
    parser: &mut Parser,
    filename: &str,
    renderer_config: &md_parser::RendererConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    parser.to_html_file_with_config(filename, renderer_config)?;
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

    // Load configuration
    let config =
        Config::load_config().map_err(|e| format!("Failed to load configuration: {}", e))?;

    // Create parser with config
    let mut parser = Parser::with_config(markdown, config.parser.clone())?;
    let ast = parser.parse()?;

    // Check for warnings and display them
    let warnings = parser.warnings();
    if !warnings.is_empty() {
        for warning in warnings {
            eprintln!("Warning: {}", warning);
        }
    }

    // Ensure output directory exists
    ensure_output_dir(&config.output.directory)?;

    // Write outputs based on configuration
    let mut outputs = Vec::new();

    if config.output.enable_ast_debug {
        write_ast_debug(
            &ast,
            &config.output.directory,
            &config.output.ast_debug_filename,
        )?;
        outputs.push(format!(
            "{}/{}",
            config.output.directory, config.output.ast_debug_filename
        ));
    }

    if config.output.enable_ast_json {
        write_ast_json(
            &mut parser,
            &config.output.directory,
            &config.output.ast_json_filename,
        )?;
        outputs.push(format!(
            "{}/{}",
            config.output.directory, config.output.ast_json_filename
        ));
    }

    if config.output.enable_html {
        write_html_output(&mut parser, &config.output.html_filename, &config.renderer)?;
        outputs.push(format!(
            "{}/{}",
            config.output.directory, config.output.html_filename
        ));
    }

    if !outputs.is_empty() {
        println!("Wrote: {}", outputs.join(", "));
    } else {
        println!("No outputs enabled in configuration");
    }

    Ok(())
}
