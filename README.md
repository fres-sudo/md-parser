# Markdown Parser with Mermaid Diagram Support

A lightweight, maintainable Markdown parser written in Rust that supports standard Markdown features with special handling for Mermaid diagrams. The parser converts Markdown text into a structured Abstract Syntax Tree (AST) that can be rendered to HTML or other formats.

## Installation

### Prerequisites

- **Rust**: This project requires Rust 2021 Edition or later. If you don't have Rust installed, you can install it using [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building from Source

1. Clone the repository:

```bash
git clone <repository-url>
cd md-parser
```

2. Build the project:

```bash
cargo build --release
```

3. The binary will be located at `target/release/md-parser`

### Running Tests

Run the test suite to verify everything works correctly:

```bash
cargo test
```

## Usage

### Command-Line Usage

The parser can be used as a command-line tool to convert Markdown files to various output formats:

```bash
cargo run --release -- assets/input.md
```

Or if you've installed the binary:

```bash
./target/release/md-parser assets/input.md
```

The program will:

- Parse the input Markdown file
- Generate output files in the `output/` directory (configurable via `config.toml`)
- Display any warnings (e.g., unclosed code blocks)

**Output files** (configurable in `config.toml`):

- `output/ast.txt` - AST in debug format
- `output/ast.json` - AST in JSON format
- `output/output.html` - Rendered HTML document

### Library Usage

The parser can also be used as a library in your Rust projects:

```rust
use md_parser::{Parser, ParserConfig};

// Parse a simple markdown string
let markdown = "# Hello World\n\nThis is a paragraph.".to_string();
let mut parser = Parser::new(markdown)?;
let ast = parser.parse()?;

// Generate HTML
let html = parser.to_html()?;

// Or save to file
parser.to_html_file("output.html")?;

// With custom configuration
let config = ParserConfig::default();
let mut parser = Parser::with_config(markdown, config)?;
```

### Configuration

The parser uses a `config.toml` file in the project root for configuration. If the file doesn't exist, default values are used.

**Example `config.toml`:**

```toml
[parser]
max_heading_level = 6
code_fence_length = 3
code_fence_pattern = "```"
mermaid_language = "mermaid"

[parser.mermaid]
default_theme = "default"
default_font_size = "16px"
default_font_family = "trebuchet ms, verdana, arial"
validate_syntax = true
use_cli_validation = false

[renderer]
output_directory = "output"
html_header_path = "assets/html_header.html"
html_footer_path = "assets/html_footer.html"
html_body_start_path = "assets/html_body_start.html"
styles_css_path = "assets/styles.css"

[output]
directory = "output"
ast_debug_filename = "ast.txt"
ast_json_filename = "ast.json"
html_filename = "output.html"
enable_ast_debug = true
enable_ast_json = true
enable_html = true
```

## Features

### Supported Features

The parser supports the following Markdown features:

- **Headings** (levels 1-6) with validation
- **Paragraphs** with inline formatting support
- **Unordered lists** with nested sub-lists (using `-`, `*`, or `+`)
- **Task lists** (checked/unchecked items: `- [ ]` and `- [x]`)
- **Inline elements**:
  - **Bold** text (`**text**`)
  - **Italic** text (`*text*`)
  - **Strikethrough** text (`~~text~~`)
  - **Links** (`[text](url)`)
  - **Images** (`![alt](url)`)
- **Fenced code blocks** with language identifiers (```` ```language ````)
- **Mermaid diagrams** with special handling:
  - Syntax validation
  - Configuration support (theme, font size, etc.)
  - Frontmatter parsing (`%%{init: {...}}%%`)
  - Graceful error handling for invalid diagrams
- **Tables** with column alignment (left, center, right)
- **Blockquotes** with nesting support (`>`, `>>`, etc.)

### Not Supported

The following common Markdown features are **not currently supported**:

- **Ordered lists** (numbered lists: `1.`, `2.`, etc.)
- **Horizontal rules** (`---` or `***`)
- **Inline code** (backticks: `` `code` ``)
- **HTML tags** and entities
- **Definition lists**
- **Footnotes** and reference-style links
- **Autolinks** (automatic URL detection)
- **Hard line breaks** (two spaces + newline)
- **Escaped characters** (`\*` for literal asterisk)
- **Typographic replacements** (smart quotes, etc.)

## Known Limitations

1. **Ordered Lists**: Numbered lists are not supported. Only unordered lists with `-`, `*`, or `+` markers are parsed.

2. **Inline Code**: Inline code spans using backticks are not parsed. Only fenced code blocks are supported.

3. **HTML Support**: The parser does not parse or render HTML tags embedded in Markdown. All HTML is treated as plain text.

4. **Reference-Style Links**: Only inline-style links `[text](url)` are supported. Reference-style links `[text][ref]` with definitions are not supported.

5. **Nested Inline Elements**: While the parser supports nested inline elements (e.g., bold within italic), complex nesting scenarios may not always parse correctly.

6. **Code Block Fence Length**: The parser is configured to use 3 backticks for code fences. Different fence lengths are not supported.

7. **Mermaid Validation**: Mermaid syntax validation is basic and may not catch all syntax errors. Full validation would require the Mermaid CLI tool.

8. **Table Parsing**: Tables must have proper alignment rows. Malformed tables may not parse correctly.

9. **Blockquote Nesting**: While nested blockquotes are supported, very deep nesting (more than 3-4 levels) may not render correctly.

10. **Performance**: The parser uses a single-pass approach with regex matching. Very large documents (10,000+ lines) may experience slower parsing times.

## Performance Characteristics

The parser is designed for efficiency and maintainability:

### Optimizations

- **RegexSet for Inline Parsing**: Uses `RegexSet` for efficient multi-pattern matching when parsing inline elements (bold, italic, links, images, strikethrough). This allows checking multiple patterns in a single pass.

- **Single-Pass Parsing**: The parser uses a state machine approach to parse the document in a single pass, reducing memory allocations and improving performance.

- **Memory-Efficient AST**: The AST uses Rust enums for efficient memory representation. Each node type only stores the data it needs.

- **Lazy Regex Compilation**: Regex patterns are compiled once when the parser is created, not on every parse operation.

### Performance Expectations

- **Small documents** (< 100 lines): Parses in < 1ms
- **Medium documents** (100-1000 lines): Parses in 1-10ms
- **Large documents** (1000-5000 lines): Parses in 10-50ms
- **Very large documents** (> 5000 lines): Parses in 50-200ms

Performance may vary based on:

- Number of inline elements (more inline formatting = slower)
- Number of code blocks (code blocks are parsed separately)
- Complexity of nested structures (lists, blockquotes)

### Memory Usage

- **AST Size**: Approximately 2-5x the size of the input Markdown text
- **Parser State**: Minimal overhead (~1KB for regex patterns)
- **No Memory Leaks**: Uses Rust's ownership system to prevent memory leaks

## Documentation

For more information about the development process:

- **[Assignment Specification](docs/assignment.md)** - Original project requirements
- **[Development Prompts](docs/prompts.md)** - Record of all AI prompts used during development

## License

See [LICENSE](LICENSE) file for details.
