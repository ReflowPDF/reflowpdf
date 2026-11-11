# <picture><source media="(prefers-color-scheme: dark)" srcset="assets/mark-dark.svg"><img height="28" src="assets/mark-light.svg" alt=""></picture>&nbsp; ReflowPDF

ReflowPDF is a PDF editor that treats a PDF as a document: you click a line and type, and the pages after it are laid out again. It runs on our own layout engine, written in Rust, without a headless browser. The engine paginates (running headers and footers, page numbers, tables of contents, footnotes, tables that continue across pages) and writes tagged PDF/UA output.

- Editor: https://app.reflowpdf.com
- Website: https://reflowpdf.com
- Docs: https://reflowpdf.com/docs
- CSS conformance results: https://reflowpdf.com/conformance

The engine and the editor are closed source. This repository holds the parts that are public.

## Contents

| Folder | What is in it |
|---|---|
| [`mcp/`](mcp) | The ReflowPDF connector for AI assistants (MCP): setup for Claude, ChatGPT, Cursor and VS Code, plugin manifests, MCP Registry entry |

## Contact

hello@reflowpdf.com. Security reports: see [SECURITY.md](SECURITY.md).

ReflowPDF is made by INSEAW LLC, South Carolina, and is a member of the [PDF Association](https://pdfa.org).
