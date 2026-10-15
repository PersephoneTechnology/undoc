# @iyulab/undoc (WASM)

Browser/Node.js WebAssembly package for [undoc](https://github.com/iyulab/undoc).

Extracts DOCX, XLSX and PPTX files, and their 97-2003 binary predecessors (DOC, XLS, PPT), to
Markdown, plain text, or JSON — no server required.

## Install

```bash
npm install @iyulab/undoc
```

## Usage (ESM)

> This package is built with `wasm-pack --target bundler`. It is an ES module and the
> WebAssembly binary is initialised for you, so there is no `init()` to await — import
> the functions and call them. Use it through a bundler (webpack, Vite, Rollup, esbuild).

```js
import { parse } from '@iyulab/undoc';

const bytes = new Uint8Array(await file.arrayBuffer());
const doc = parse(bytes);

console.log(doc.format());       // "docx" | "xlsx" | "pptx" | "doc" | "xls" | "ppt"
console.log(doc.toMarkdown());   // Markdown string
console.log(doc.toText());       // Plain text string
console.log(doc.toJson());       // JSON string
```

## API

### `supportedFormats(): string`

JSON array of the formats this build can parse (the published package parses all six):

```js
import { supportedFormats } from '@iyulab/undoc';

JSON.parse(supportedFormats());
// [{ extension: "docx", name: "Word Document" },
//  { extension: "xlsx", name: "Excel Workbook" },
//  { extension: "pptx", name: "PowerPoint Presentation" },
//  { extension: "doc", name: "Word 97-2003 Document" },
//  { extension: "xls", name: "Excel 97-2003 Workbook" },
//  { extension: "ppt", name: "PowerPoint 97-2003 Presentation" }]
```

Ask rather than hardcoding the extensions on your side — a local copy of the list goes stale
the moment this package learns a new format, and nothing tells you it has.

### `parse(data: Uint8Array): OfficeDocument`

Parse a document's bytes. Throws if the format is unrecognized, or if it is a format this build
leaves out (see below) — the error's `kind` then says `UnsupportedFormat` (4).

### `OfficeDocument`

| Method | Returns | Description |
|--------|---------|-------------|
| `fromBytes(data)` | `OfficeDocument` | Alias for module-level `parse()` |
| `format()` | `string` | `"docx"` \| `"xlsx"` \| `"pptx"` \| `"doc"` \| `"xls"` \| `"ppt"` |
| `toMarkdown()` | `string` | Full document as Markdown |
| `toText()` | `string` | Plain text extraction |
| `toJson()` | `string` | Structured JSON |
| `metadata()` | `string` | JSON of title/author/subject etc. |

## Building with fewer formats

Each format's parser is a cargo feature of `undoc-wasm` (`docx`, `xlsx`, `pptx`, `doc`, `xls`,
`ppt`; all on by default). An application that reads only some of them can build the package
itself without the rest — most of the bundle is the parsers, not the shared code:

```bash
git clone https://github.com/iyulab/undoc && cd undoc/undoc-wasm
wasm-pack build --release --target bundler -- --no-default-features --features docx,xlsx,pptx
```

| Features | `undoc_wasm.wasm` (release, before `wasm-opt`) |
|---|---|
| all six (the published package) | 1,929,505 bytes |
| `docx,xlsx,pptx` | 1,682,136 bytes (−12.8%) |
| `docx` | 1,433,407 bytes (−25.7%) |

`supportedFormats()` reports only the formats the build contains, so code that asks it rather than
hardcoding extensions keeps working unchanged.

## Playground

Live demo: https://iyulab.github.io/undoc/
