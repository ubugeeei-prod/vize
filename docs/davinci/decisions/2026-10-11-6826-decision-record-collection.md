# Decision record collection (#6826)

Issue: [#6826](https://github.com/ubugeeei-prod/vize/issues/6826).

## Problem

At source `ceb3677dfc29b9b6a9c5751771b8f01cb0969c52`, the level restructure
foundation occupies 983,785 bytes in 350 physical rows. Its largest row is
55,750 bytes. Independent decisions append to the same paragraphs, making
their PRs conflict and making the reasoning difficult to read.

The directory already contains 858 independent Markdown records, including
the history, order, and designs that declare themselves part of the canonical
record. The foundation links 796 distinct records, but there is no collection
entry point. Requiring another shared list entry for every record would retain
the same conflict point.

## Decision

Treat the full [decision collection](./README.md) as canonical. Keep the
foundation and every historical record at its existing path. Add one stable
collection link near the foundation's title, and clarify the paired recording
rule in `AGENTS.md`.

Future decisions are recorded in a dedicated, uniquely named file and a
concise comment on their existing issue in the same change. The GitHub
directory offers browsing, and recursive local file search enumerates the
complete collection. A new decision does not require appending to the shared
foundation or README.
Explicit supersession links define changes to earlier decisions; dates alone
do not establish authority or implementation completion.

## Preservation and validation

The foundation's only change appends the collection link to its existing
introductory note paragraph; its heading separator stays blank. Removing that
128-byte addition restores all 983,785 original bytes and 350 line endings.
Historical records, clauses, naming script anchors, and source custody
snapshots retain their original bytes.
PRs already in flight retain their own existing paired records.

The source-length gate continues to include Markdown with the existing
350-line ratchet. New records use normal paragraphs and named companions
when needed; this change adds no budget exclusion or generated index.
Validation checks the whole foundation inverse, existing record preservation,
Markdown block structure, relative links, and the four-file source scope.
Exact-head Actions and actual protected merge completion remain required.

Public documentation build selection is tracked separately under
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830); this decision changes
the recording workflow and collection navigation.
