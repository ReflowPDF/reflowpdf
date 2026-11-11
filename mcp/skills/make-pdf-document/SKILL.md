---
name: make-pdf-document
description: Make a paged PDF document with ReflowPDF — an invoice, quote, report, contract, letter, CV, certificate or docs export — from a gallery template or from HTML and CSS, and let the person keep editing it in the ReflowPDF editor. Use when the person asks for a PDF or a printable document.
---

# Make a document with ReflowPDF

Requires the ReflowPDF MCP server (`https://app.reflowpdf.com/mcp`).

1. **Common document?** Invoice, quote, receipt, statement, report, contract, letter, CV, certificate, label, menu: find a template with `list_templates`, read its sample data with `get_template`, then print it with the person's values using `render_template`.
2. **Anything else:** read `read_guide` (topic `html`) once, then write the whole document as HTML and CSS and print it with `render_pdf`.
   - `@page` for paper size and margins; margin boxes (`@top-center`, `@bottom-right`, …) for running headers, footers and page numbers.
   - `target-counter(attr(href), page)` for table-of-contents page numbers, `float: footnote` for footnotes, `string-set` for running chapter titles.
   - `break-before`, `break-inside: avoid` to control pagination. JavaScript does not run.
3. **Check before you hand it over.** The answer includes pictures of pages: look at fonts, layout and images, and make sure nothing is cut off or spilled onto an extra page.
4. **Give both links:** the PDF link (valid 24 hours) and the edit link.
5. **Later changes:** the person may have edited the document in the editor. Read the current version with `get_document`, change it, and write it with `update_document`, passing the `base_revision` you read. If the write is refused, read again — never overwrite their edit.
6. **Images:** use `upload_image` for a file you have, or `ask_for_images` to give the person an upload link. An image at a public https URL can be used directly.
7. **Existing PDF to edit:** `import_pdf` returns a link that opens it in the editor's import.

Without signing in, every page carries a small ReflowPDF mark. If the person wants it removed or wants documents kept in their library, call `connect_account`.
