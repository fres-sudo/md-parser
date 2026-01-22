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
