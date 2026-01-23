# Prompts Used During Development

This document contains a record of all prompts used during the AI-assisted development of this Markdown parser project. This serves as documentation of the development process and AI usage.

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

### Prompt 24 - Cursor (auto mode)

Can you plan and build the possibility to parse and render images? (\!\[alt\]\(url\))

### Prompt 25 - Cursor (auto mode)

Can you add a CI/CD Github Actions pipeline that run linters and tests upon pushing to the master branch?

**Verification**:

- Ensure the pipeline is working and the steps are correct.

### Prompt 26 - Cursor (auto mode)

Currently multiple regex searches in find_earliest_match() could be optimized

Consider:

- Use RegexSet for multiple pattern matching
- Cache match results when possible
- Consider using aho-corasick for multiple string searches

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains the files.
- Ensuring new tests passes.

### Prompt 27 - Cursor (auto mode)

Can you add a possibility to parse and render code blocks with different languages?

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains properly rendered code blocks with different languages.
- Ensuring new tests passes.

### Prompt 28 - Cursor (auto mode)

Currently the page is not beign stlyed at all, all the rules are not beign used, can u fix this?

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains properly styled pages.
- Ensuring new tests passes.

### Prompt 29 - Cursor (auto mode)

Can you add parsing and rendering logic for Blockquotes?

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains properly rendered blockquotes.
- Ensuring new tests passes.

### Prompt 30 - Cursor (auto mode)

I am getting this error in the CI.

...CI output...

Can you fix it?

**Verification**:

- Ensuring the CI is passing.

### Prompt 31 - Perplexity AI

Why am I getting this error?

...CI output...

in this CI pipeline?

...CI content...

Please fix it.

**Verification**:

- Ensuring the CI is passing.
- Ensuring the output/ folder contains properly rendered blockquotes.
- Ensuring new tests passes.

### Prompt 32 - Cursor (auto mode)

I am getting this error in the CI. Can you help?

...CI output...

**Verification**:

- Ensuring the CI is passing.
- Ensuring the output/ folder contains properly rendered blockquotes.
- Ensuring new tests passes.

### Prompt 33 - Cursor (auto mode)

Currenlty there are some tests that do not pass and some other that gives warnings, can you help fixes?

...tests output...

**Verification**:

- Ensuring all the tests passes.

### Prompt 34 - Cursor (auto mode)

Can you add support for more complex nested inline elements like a bold text nexted with an italic one? Currently those are not supported in the parser and in the render engine. Can you add this feature and its relative tests?

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains properly rendered nested inline elements.
- Ensuring new tests passes.

### Prompt 35 - Cursor (auto mode)

Can you add parsing and rendering support for ordered lists as (1. 2. 3., etc...). Add all the necessary tests.

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains properly rendered ordered lists.
- Ensuring new tests passes.

### Prompt 36 - Cursor (auto mode)

Can you implement horizontal rules (--- or ***) with parsing and render logic as well as tests.

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains properly rendered horizontal rules.
- Ensuring new tests passes.

### Prompt 37 - Cursor (auto mode)

Can you implement inline code (backticks) parsing and rendering logic as well as tests.

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains properly rendered inline code.
- Ensuring new tests passes.

### Prompt 38 - Cursor (auto mode)


Can you implement reference-style links parsing and rendering logic as well as tests.

**Verification**:

- Ensuring all the tests passes.
- Ensuring the output/ folder contains properly rendered reference-style links.
- Ensuring new tests passes.
