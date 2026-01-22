# Markdown Parser with Mermaid Diagram Support

## Goal

Use AI to help design and implement a Markdown parser that supports standard Markdown features plus **Mermaid code blocks**, producing a structured representation that could be rendered to HTML (or another target) with special handling for Mermaid diagrams.

## Tasks

1. **Design & Implement**
Use AI to assist in designing and implementing a Markdown parser (language of your choice) that can recognize headings, lists, links, emphasis, code blocks, and specifically detect and tag ````mermaid` code blocks as separate diagram nodes in the output AST/structure.
2. **Test & Demonstrate**
Create a suite of test Markdown documents (including edge cases) and a small demo program that parses them and outputs a readable representation (e.g. JSON AST or HTML), showing correct handling of normal Markdown and Mermaid blocks.
3. **AI Usage & Verification Report**
Collect and submit the prompts you used with AI and write a short report explaining how you evaluated, corrected, and verified the AI-generated code (tests, manual inspection, comparison with reference Markdown behaviour).

## Prompts Used

### Prompt 1 - Google Gemini

  I need to develop this project for a uni course.

  ...assignment specifications...

  I firstly need you to create a very detailed prompt for an LLM in order to guide it in the development of a TDD markdown parser with special support for mermaid in rust. Make sure to guide it to follow the rust best practices focusing on KISS principle and mantainability.

### Prompt 2 - Google Gemini

 Can you also create the rules.mdc file for the project?

### Prompt 3 - Cursor (auto mode - it selects the best model for the task automatically)

 Role: You are a Senior Rust Engineer and Technical Lead. We are building a lightweight, maintainable Markdown parser from scratch for a university project.

Goal: Create a library that parses Markdown text into a structured Abstract Syntax Tree (AST). The parser must support standard Markdown features and have special handling for Mermaid diagrams.

Constraints & Principles:

    Language: Rust (2021 Edition).

    Architecture: Input String → Parser State Machine → Vector of AST Nodes.

    Methodology: TDD (Test-Driven Development). You must provide the test cases before the implementation logic for each feature.

    KISS Principle: Do not use heavy parsing libraries like nom or pest unless necessary. Use the standard library and regex crate if needed. Keep the logic readable.

    Maintainability: Use Enums for the AST. Use robust pattern matching.

Specific Requirements:

    The AST: Define a Node enum. It must clearly distinguish between a standard CodeBlock and a MermaidDiagram.

    Supported Syntax:

        Headings (H1-H6)

        Paragraphs

        Unordered Lists ( - or *)

        Inline elements: Bold (**), Italic (*), and Links [text](url).

        Fenced Code Blocks (```)

        Crucial: If a fenced code block has the language identifier mermaid, it must be parsed into a specific Node::MermaidDiagram, not a generic Node::CodeBlock.

Workflow: Please guide me through this development in 3 Phases. Stop after each phase to let me implement and run the code.

Phase 1: Foundation & AST Design

    Define the Node Enum and the basic Parser struct.

    Implement a simple parse function that takes a string and returns Vec<Node>.

    TDD: Write a test for a simple plain text paragraph.

    Implement the parsing logic to pass that test.

Phase 2: Block Structure (The Mermaid Logic)

    TDD: Write tests for:

        A standard Rust code block.

        A Mermaid code block (ensure the output AST variant is different).

        Headings.

    Implement the logic to detect fenced blocks. Check the "info string" (the language tag). If it equals mermaid, produce the specific Mermaid AST node.

Phase 3: Inline Elements & Polish

    TDD: Write tests for bold, italic, and links.

    Implement the inline parsing logic (this can be a second pass or integrated, whichever is simpler/cleaner).

    Add a method to serialize the AST to JSON (or a Debug print) to demonstrate the structure.

Immediate Request: Please start with Phase 1. Show me the Cargo.toml dependencies, the AST design, the first test case, and the initial implementation.

### Prompt 4 - Cursor (auto mode)

Go on with phase 2.

### Prompt 5 - Cursor (auto mode)

Go on with phase 3.

### Prompt 6 - Cursor (auto mode)

Can you create a function that takes the json formatted AST and output an html  file so i can visualize the output?

### Prompt 7 - Cursor (auto mode)

Can you split tests in a separate folder to ensure good code structure?

### Prompt 8 - Cursor (auto mode)

Can you move all the static assets that are currently beign hardcoded into the rust files to a separate folder/file for better mantainability code structure?

### Prompt 9 - Cursor (auto mode)

Can you ensure that the html output of the program is located in a specific output/ folder?

### Prompt 10 - Cursor (auto mode)

Can you make sure that the program takes a file .md as input instead of an hardcoded string inside the @src/main.rs file?

### Prompt 11 - Cursor (auto mode)

Can you create a cursor rule for rust code styling and best practices?

**Verification**:

- [RustWiki Style Guide](https://rustwiki.org/en/style-guide/)
- [RustLang Style Guide](https://doc.rust-lang.org/style-guide/)

### Prompt 12 - Cursor (auto mode)

Can you do a refactoring of the @src/lib.rs file and @src/main.rs file ensuring those files (and the ones created if needed) follow the style guidelines defined in @.cursor/rules/rust-style.mdc file.

**Verification**:

- Ensuring all the tests passes.

### Prompt 13 - Cursor (auto mode)

Instead of printing in the terminal can u output other format of the AST as files in the output/ folder?

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains the files.
- Ensuring new tests passes.

### Prompt 14 - Cursor (auto mode)

Can you split up the parsing logic from the rendering logic in @src/lib.rs file?

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains the files.
- Ensuring new tests passes.

### Prompt 15 - Cursor (auto mode)

Can you edit the AST and add logic for lists? Currently there's no parsing logic for unordered lists or nested lists with indentation. Add tests for that.

**Verification**:

- Ensuring all the tests passes.

### Prompt 16 - Cursor (auto mode)

Can you edit the parsing logic and handle also unclosed code blocks gracefully? Add warnings in the cli for unclosed code blocks and auto-close them at EOF.

**Verification**:

- Ensuring all the tests passes.

### Prompt 17 - Cursor (auto mode)

Can you improve the current error handling for the AST (in the @src/ast.rs file) in order to inclued error for:

- invalid heading levels (>6)
- malformed markdown structure
- position/line information in errors of the input md file.

**Verification**:

- Ensuring all the tests passes.

### Prompt 18 - Cursor (auto mode)

Can you improve the code quality and avoid hardcoded numbers and strings?
For example:

- hardcoded pattern for code blocks matching
- hardcoded numbers for fence length (3 for exmpale)
- String literals like "link", "bold" etc...used as match type -> use enum instead.

**Verification**:

- Ensuring all the tests passes.

### Prompt 19 - Cursor (auto mode)

Can you add a proper configuration file to this project?

**Verification**:

- Ensuring all the tests passes.
- Ensuring the configuration file is created in the root of the project.
- Ensuring the configuration file is named .env.
- Ensuring the configuration file is named .env.example.
- Ensuring the configuration file is named .env.example.

### Prompt 20 - Cursor (auto mode)

Can you add task lists (- [ ] and - [x]) to the parser?

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains the files.
- Ensuring new tests passes.

### Prompt 21 - Cursor (auto mode)

Can you add markdown tables in the parser?

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains the files.
- Ensuring new tests passes.

### Prompt 22 - Cursor (auto mode)

Can you organize tests better in differernt files divided by domain?

**Verification**:

- Ensuring all the tests passes.

### Prompt 23 - Cursor (auto mode)

Can u improve the current mermaid implementation by adding:

- Validate Mermaid syntax
- Support for Mermaid configuration
- Error handling for invalid Mermaid diagrams (gracefully)

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains the files.
- Ensuring new tests passes.
