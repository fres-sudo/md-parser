use md_parser::Parser;

fn main() {
    let markdown = "This is a simple paragraph.\n\nThis is another paragraph.";
    let parser = Parser::new(markdown.to_string());
    let ast = parser.parse();

    println!("Parsed AST:");
    for (i, node) in ast.iter().enumerate() {
        println!("  {}: {:?}", i, node);
    }
}
