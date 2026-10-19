# Persephone fork maintenance

## Current baseline and ownership

- Official repository: https://github.com/iyulab/undoc
- Official baseline: **0.27.0**, `8dd4cdd7d77399a191b0122cc154e4c7a5ea1e40`.
- Organization fork: https://github.com/PersephoneTechnology/undoc
- Current functional revision: `8b52d1339129b419d940dfff72969808b5ed45b3`, on `codex/xlsx-source-provenance`.
- Word-only functional revision: `43259db46ea18d50e8f252c14f5747a359dec167`.
- The consuming product's Cargo.toml and Cargo.lock are authoritative for its deployed dependency revision. A later documentation-only commit does not change the functional revision above.

The official baseline is an ancestor of the current functional revision. No upstream
upgrade has been incorporated since that baseline. This does not claim that 0.27.0
is the latest upstream release. Official baseline code belongs to upstream; the net
Git difference from that baseline is our maintained delta.

Git commits are the patch history; this document is the single change inventory.
Do not maintain a duplicate hand-edited patch series. Review the final difference
as well as individual commits: reverted work remains history, not active capability.

## Maintenance rules

1. Before each change, record its ID, purpose, affected modules, compatibility impact,
   predecessor, validation and upstream status here (and in the product spec when applicable).
2. Keep additions narrowly scoped. Preserve existing parsing/text behavior and module
   structure; do not move existing methods or refactor merely because a file is large.
   Any new behavior or structural exception requires explicit approval.
3. Keep feature, documentation, rollback and upstream-merge commits distinguishable.
   Record fixes under the same capability ID. Never rewrite published history.
4. Preserve upstream history and the MIT license. Do not mix an upstream upgrade with
   feature work; record the old/new official SHAs and conflict decisions separately.
5. An optional serialized field is not necessarily Rust source-compatible. Record
   public model and output differences honestly; test the affected consumers.
6. After upstream adopts equivalent behavior, verify semantics, remove the redundant
   local implementation, and mark its ID retired with the upstream release/commit.
   Preserve its history, tests and retirement decision.

## Active capabilities

| ID | Purpose and boundary | Implementation history | Validation | Upstream status |
| --- | --- | --- | --- | --- |
| UD-001 | Optional DOCX structural XML addresses on paragraphs/tables and public source types; no text-search or viewer behavior | Word commit W1 below | source_provenance_test; model/Markdown comparison | Not submitted; no issue or PR |
| UD-002 | Capture original DOCX paths before filtering/deduplication, including table descendants; does not expand namespace-prefix support | W1; uses UD-001 | source_provenance_test; DOCX parser tests | Not submitted; no issue or PR |
| UD-003 | Optional XLSX worksheet identity and original cell/merge ranges; only XLSX enables tracking, uncertain/conflicting sources remain unavailable | X1 → X2 → X3 below; net result restores original parser structure | xlsx_source; source_conflicts_never_produce_coordinates; xlsx_merged_header_end_to_end_test; Word regression | Not submitted; no issue or PR |
| DOC-001 | Fork purpose, traceability and compatibility evidence; no runtime behavior | D1/D2 and documentation updates | File coverage, ancestry and diff review | Fork maintenance only; exclude organization-specific prose from upstream feature PRs |

### File ownership in the effective delta

Shared files have multiple owners; inspect individual hunks rather than assigning a
whole file to one feature. Test constructor updates are compatibility support, not
new DOCX/XLS/Markdown behavior.

| Files relative to repository root | Owner / reason |
| --- | --- |
| `src/model/source.rs`, `src/model/paragraph.rs`, `src/lib.rs` | UD-001: XML source model, optional paragraph source and exports |
| `src/model/mod.rs`, `src/model/table.rs` | UD-001 + UD-003: model declarations/exports and optional table/cell source fields |
| `src/docx/mod.rs`, `src/docx/source.rs` | UD-002: DOCX source tracking |
| `src/docx/parser.rs` | UD-002: capture XML paths; UD-003: initialize the added optional cell field to None |
| `src/model/spreadsheet_source.rs`, `src/sheet.rs`, `src/xlsx/parser.rs` | UD-003: worksheet/cell provenance, guarded tracking and XLSX enablement |
| `src/xls/parser.rs` | UD-003: initialize the added optional field to None; XLS tracking remains disabled |
| `src/render/grid.rs`, `src/render/markdown.rs` | UD-003: test Cell constructors only; renderer behavior unchanged |
| `tests/source_provenance_test.rs`, `tests/fixtures/provenance-decisions.docx` | UD-001/002 regression evidence |
| `examples/provenance_snapshot.rs`, `tests/support/compare_provenance.py` | UD-001/002 compatibility tooling; also used when assessing UD-003 |
| `tests/xlsx_source.rs`, `tests/fixtures/xlsx-source.xlsx` | UD-003 regression evidence |
| `README.md`, `PERSEPHONE.md` | DOC-001 |

### Compatibility and layering

- Text extraction, filtering, existing display spans and Markdown behavior are to
  remain unchanged. Source metadata is additive and independently interpreted by
  the consuming product. The fork has no search offsets, file revisions or viewer API.
- Old serialized models can omit the new optional fields. XLSX serialization now
  includes source metadata when available; it is not byte-identical JSON output.
- Adding fields to public Rust structs requires external explicit struct literals
  to be adapted. This is a known source-compatibility impact, not a parser behavior change.
- UD-003 was developed on the Word fork. Its types do not require Word source data,
  but the recorded diff has shared-model context dependencies. Directly applying
  the XLSX-only net diff to official 0.27.0 fails at `src/model/mod.rs` and
  `src/model/table.rs`; do not present it as an independently replayable upstream patch.
- A few explicit None initializers alongside existing default constructors are
  redundant but harmless. They remain known review noise; this documentation pass
  intentionally does not edit source code. Drop them when preparing a focused upstream PR.

## Commit and correction ledger

| Key | Full commit | Meaning / current effect |
| --- | --- | --- |
| D1 | `60d2c150946c8b69f7168ebdc2ddb3106d585c5d` | Explain fork purpose and maintenance |
| D2 | `cd67c615e56c8c1ce8aeabfc7d9d4cb38d5e974d` | English documentation and 0.27.0 baseline; historical documentation checkpoint, not current HEAD |
| W1 | `43259db46ea18d50e8f252c14f5747a359dec167` | UD-001/002 implementation and tests; published Word-only baseline |
| X1 | `f93768c01564dee7d485535ccaceb1185ea17864` | Extract XLSX cell construction; structural effect reverted by X3, not an active feature |
| X2 | `22c94910d68342896b1b9415fbfbd9189e218116` | UD-003 implementation and tests |
| X3 | `8b52d1339129b419d940dfff72969808b5ed45b3` | Restore original parser method/file structure, visibility and model declaration order; retain UD-003 |

2026-10-19: X1/X2 were published with authorization; X3 followed the user's explicit
correction. `src/xlsx/cell.rs` no longer exists. Excluding its new optional-field
initialization, the restored method matches W1 byte-for-byte. Earlier statements
that the fork had no product-consumed SHA described the pre-publication stage only.

2026-10-19 clarification: keep the fork + Git-commit workflow. The requirement is
clear ownership, minimal changes and traceability, not zero touched upstream lines
or a second independent patch-file inventory. Structural rollback remains in force.

## Validation evidence and limits

Same baseline/fork configuration: default features and default RenderOptions.
The product previously used undoc 0.5.4; that migration is separate from these
metadata-only comparisons. No additional product format is enabled here.

- W1: `source_provenance_test` 5 passed; DOCX parser tests 59 passed.
- Official vs W1: models and Markdown match after removing only added XML source
  fields across decisions/basic/mapping/structural/large fixtures. D-03 resolves
  to document/body/p[7]; this is parser evidence, not GUI acceptance.
- Upstream's literal `w:` handling remains unchanged. Alternate-prefix nodes retain
  existing extraction behavior while expanded-name siblings inform source paths.
- UD-003: XLSX source integration 2, conflict unit 1, merged-header integration 6,
  and Word source integration 5 passed on X3. Sparse coordinates, original merges,
  duplicate/conflicting positions and source absence are covered.
- W1 vs XLSX candidate: sparse/merged/long-cell XLSX, Word decisions and a PPTX table
  have equal models/Markdown after removing only spreadsheet_source. Product tests
  additionally cover normalization, UTF-8, repeated hits, long-cell segmentation,
  comments and resource limits. Sample parity is not a proof for all documents.
- No logging, extraction policy, deduplication policy or renderer behavior added.
- Native viewer qualification and real financial samples remain product work.
  See financial-agent `specs/changes/xlsx-search-source-navigation/spec.md` for the
  source-navigation validation and the consuming revision.

## Review, replay and future integration

Use immutable SHAs when recording evidence; branches are navigation aids only.
For the functional version documented here:

```sh
BASE=8dd4cdd7d77399a191b0122cc154e4c7a5ea1e40
WORD=43259db46ea18d50e8f252c14f5747a359dec167
TIP=8b52d1339129b419d940dfff72969808b5ed45b3
git merge-base --is-ancestor "$BASE" "$TIP"
git log --reverse --oneline "$BASE..$TIP"
git diff --stat "$BASE" "$TIP"
git diff "$WORD" "$TIP" -- src tests
git diff --binary "$BASE" "$TIP" > /tmp/undoc-effective.patch
```

To validate the complete effective delta, export BASE to a disposable directory,
check/apply the generated patch there, compare resulting changed-file bytes with
TIP, then reverse-apply and compare with BASE. Never run the reverse experiment in
the working checkout. This is an on-demand Git artifact, not a maintained patch copy.

For each future official upgrade:

1. Record old/new upstream SHAs and inspect official changes against every active ID.
2. Integrate on an isolated branch; record conflicts and whether each capability is
   still necessary, partially upstreamed or fully equivalent. Do not silently retain
   duplicate official functionality or claim equivalence from names alone.
3. Run affected parser/source/consumer cases and same-configuration text comparisons.
   Record failures and limitations as well as passes.
4. Pin the verified fork SHA in the product, retaining the previous pin for rollback.
   Mark retired IDs with the upstream commit/release and local removal commit.

For an upstream contribution, use a separate branch from the chosen official SHA.
Prepare Word model/capture and XLSX source changes as explicitly scoped units;
adapt shared-model context and test constructor requirements, omit product-only
documentation, and explain public Rust model compatibility. Include minimal
fixtures, expected behavior and regression results. Record the actual issue/PR URL,
review status and eventual upstream commit here. Do not send the current branch's
entire Word/XLSX/refactor/revert history as an unexamined feature PR.

### Maintenance audit (2026-10-19)

The 22-file complete effective delta from official baseline
`8dd4cdd7d77399a191b0122cc154e4c7a5ea1e40` to functional revision
`8b52d1339129b419d940dfff72969808b5ed45b3` was checked/applied to a fresh
baseline archive. Every changed file matched the functional revision byte-for-byte.
Reverse application then restored the baseline bytes and removed added files.
All 22 paths are covered by the ownership inventory above. This confirms replay of
the combined maintained delta, not independent XLSX application or compatibility
with a newer upstream version. The XLSX-only shared-context limitation remains noted.
This audit changes documentation only; parser behavior and product pin are unchanged.
