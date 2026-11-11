# ReflowPDF connector (MCP)

A remote MCP server that turns HTML and CSS, or one of 50+ ready templates, into paged PDFs. Each document also gets an edit link to the ReflowPDF editor. You can change the document there by hand, and the assistant reads your version before it makes the next change.

```
https://app.reflowpdf.com/mcp
```

Transport: Streamable HTTP. No account needed; signing in is optional (OAuth).

## Setup

**Claude** (web, desktop, mobile): Customize → Connectors → Add custom connector, paste the URL above.

**Claude Code:**

```bash
claude mcp add --transport http reflowpdf https://app.reflowpdf.com/mcp
```

or as a plugin, which also installs a skill that tells Claude how to use the tools:

```
/plugin marketplace add ReflowPDF/reflowpdf
/plugin install reflowpdf@reflowpdf
```

**ChatGPT:** chatgpt.com/plugins → Add → custom MCP server, paste the URL. Custom servers work on web and desktop.

**Cursor** (`~/.cursor/mcp.json`):

```json
{ "mcpServers": { "reflowpdf": { "url": "https://app.reflowpdf.com/mcp" } } }
```

**VS Code** (`.vscode/mcp.json`):

```json
{ "servers": { "reflowpdf": { "type": "http", "url": "https://app.reflowpdf.com/mcp" } } }
```

Other clients that support remote MCP over Streamable HTTP work with the same URL.

## Example requests

- Make a PDF invoice for Brightloom B.V.: 8 standing desks at €489 and 8 task chairs at €329, 21% VAT, due in 14 days.
- Turn my meeting notes into an A4 report with a table of contents and page numbers in the footer.
- I changed the headline in the editor. Now remove the second table on page 2.
- Add my company logo to the top of the invoice.
- Open this PDF in the editor so I can change it: https://reflowpdf.com/samples/invoice-pdfua.pdf

## Tools

| Tool | Does |
|---|---|
| `render_pdf` | Prints an HTML document as paged PDF. Returns a PDF link (24 h) and an edit link |
| `render_template` | Fills a gallery template, or your own, with JSON data and prints it |
| `list_templates` | Finds templates by word or category |
| `get_template` | Returns a template's source and sample data |
| `open_in_editor` | Opens HTML or a template in the editor, returns the edit link |
| `get_document` | Reads a document as it is now, including edits made in the editor |
| `update_document` | Writes a new version; refused if the document changed since you read it |
| `upload_image` | Stores an image and returns a link to use in a document |
| `ask_for_images` | Gives the person an upload link for a logo or photos |
| `import_pdf` | Returns a link that opens an existing PDF in the editor's import |
| `read_guide` | The engine's guide: HTML, pages, design, charts, codes, forms, templates |
| `connect_account` | Signs the person in to ReflowPDF |

## Accounts and data

Without an account every tool works and each page carries a small ReflowPDF mark. A free account removes the mark within the plan's pages and keeps documents in your library; paid plans add pages ([pricing](https://reflowpdf.com/pricing)).

PDF links last 24 hours. Documents are kept 30 days after their last change, images 30 days after the last change of a document that uses them, PDFs sent for import 24 hours. Full details: [privacy policy](https://reflowpdf.com/privacy), [terms](https://reflowpdf.com/terms).

## Files in this folder

| File | For |
|---|---|
| `plugin.json`, `mcp.json`, `skills/` | [Agent Plugins](https://agent-plugins.org) package (Cursor, Codex and compatible clients) |
| `.claude-plugin/plugin.json`, `.mcp.json` | Claude Code plugin |
| `server.json` | [MCP Registry](https://registry.modelcontextprotocol.io) entry, `com.reflowpdf/reflowpdf` |
| `assets/` | Icons |

Questions or a document that doesn't lay out right: hello@reflowpdf.com or an issue in this repository.
