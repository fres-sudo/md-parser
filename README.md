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

 ## Prompts Used

 ### Prompt 1 - Google Gemini

  I need to develop this project for a uni course.

  ...assignment specifications...

  I firstly need you to create a very detailed prompt for an LLM in order to guide it in the development of a TDD markdown parser with special support for mermaid in rust. Make sure to guide it to follow the rust best practices focusing on KISS principle and mantainability.
