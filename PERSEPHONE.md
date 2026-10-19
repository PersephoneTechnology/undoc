# Persephone fork maintenance

Upstream: https://github.com/iyulab/undoc

Official baseline: **0.27.0**, `8dd4cdd7d77399a191b0122cc154e4c7a5ea1e40`.
The baseline is an ancestor of the fork's current committed HEAD,
`cd67c615e56c8c1ce8aeabfc7d9d4cb38d5e974d`. Its two additional commits only
change README documentation. Source changes below are published on `codex/word-source-provenance`; the product
pins their full commit SHA.

## Build configuration

Product before migration: undoc `=0.5.4`, default features, default RenderOptions.
Metadata compatibility comparison: official and fork 0.27.0, identical default
features and default RenderOptions. No new product format is enabled by this work.

## Local changes

| ID | Reason | Modules | PR | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| UD-001 | Optional structural XML provenance without changing text semantics | model/source, model/paragraph, model/table | codex/word-source-provenance | source_provenance_test | active |
| UD-002 | Capture source paths during DOCX parsing, including filtered nodes and table descendants | docx/source, docx/parser | codex/word-source-provenance | source_provenance_test; official/fork model and Markdown comparison (5 fixtures) | active |
| DOC-001 | Explain fork purpose and maintenance | README, PERSEPHONE | Existing README commits; codex/word-source-provenance | Manual review | active |

## Reviewing the effective difference

Use Git's diff against the full official baseline SHA above. Do not maintain a
second patch inventory. Every effective source difference must map to an active
entry here. Preserve upstream history and the MIT license. Separate necessary
refactoring, feature commits, and upstream merges when commits are authorized.

## Upgrade record

2026-10-19: verified remote HEAD and baseline ancestry; no upstream upgrade and
no product-consumed provenance SHA yet. Upgrade/rollback rehearsal and source
compatibility validation are pending. Published history must not be rewritten.

## Validation evidence (2026-10-19)

- `cargo test -p undoc --test source_provenance_test --offline`: 5 passed.
- `cargo test -p undoc --lib docx::parser::tests --offline`: 59 passed.
- Built `examples/provenance_snapshot.rs` unchanged on the official baseline and
  fork, both with default features and default RenderOptions. Ran
  `tests/support/compare_provenance.py` on the decision fixture and product spike
  fixtures basic, mapping, structural, and large: all 5 models and Markdown outputs
  equal after removing only the added structural source fields.
- Real D-03 output is `word/document.xml` / document/body/p[7]. This is parser
  evidence, not a product navigation or GUI acceptance result.
- Upstream recognizes literal `w:` tags. Alternate prefix nodes retain upstream
  extraction behavior (ignored); they still count as expanded-name siblings for
  provenance of supported output. Broadening upstream prefix support is not part
  of this metadata change.
- No logging added; no upstream extraction or deduplication policy changed.

## XLSX source extension (2026-10-19)

Base: `43259db46ea18d50e8f252c14f5747a359dec167` (the product's Word provenance fork).
No parser upgrade. Publication authorized on 2026-10-19 to `codex/xlsx-source-provenance`.
The consuming product records the full published SHA in Cargo.toml and Cargo.lock.

| ID | Reason | Modules | PR | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| UD-003 | Retain worksheet identity and original cell/merge coordinates separately from display spans | model/spreadsheet_source, model/table, sheet, xlsx/parser, xlsx/cell; mechanical Cell literals in consumers | codex/xlsx-source-provenance (no upstream PR) | xlsx_source; source_conflicts_never_produce_coordinates; xlsx_merged_header_end_to_end_test | active |

`build_sheet_cell` was extracted from the existing oversized XLSX parser without changing behavior.
Missing addresses keep no source; conflicting/duplicate coordinates invalidate the sheet's sources.
All fields are optional and omitted from serialization when absent. XLS does not enable source tracking.

Validation: 2 source integration cases, 1 conflict case, 6 existing merged-header cases,
and 5 Word provenance cases passed. Pinned fork vs candidate snapshots (identical default
features and Markdown options) matched for sparse/merged/long-cell XLSX, the Word decision
fixture, and a PPTX table after removing only spreadsheet_source. Product tests additionally
cover normalization, repeated occurrences, UTF-8 and cross-segment offsets.

### Authorized publication record

- Keep the pure cell-construction extraction separate: `f93768c`.
- Follow with the optional XLSX source feature, focused tests and this maintenance record.
- Prior "no product-consumed provenance SHA" notes describe the earlier Word development
  stage; the current product baseline is the published `43259db46ea18d50e8f252c14f5747a359dec167`.
- No upstream merge, version upgrade, history rewrite or product commit is included.
- Product integration and native platform qualification remain separately tracked in
  `specs/changes/xlsx-search-source-navigation/spec.md` in financial-agent.
