# Decision records

The canonical decision record is this collection: the foundation, its
companions, and the issue-specific records in this directory. Start with:

- [Level restructure foundation](./2026-09-27-level-restructure.md)
- [History and deletion criteria](./2026-09-27-level-restructure-history.md)
- [Order of work](./2026-09-27-level-restructure-order.md)
- [Detailed designs](./2026-09-27-level-restructure-designs.md)

Browse [all records on GitHub](https://github.com/ubugeeei-prod/vize/tree/main/docs/davinci/decisions),
or list them locally with `rg --files docs/davinci/decisions -g '*.md'`.
The directory is the browsable index, including older filenames.
Local file search recursively enumerates the complete collection.
Adding a record requires no entry in this README or the shared foundation.

## Recording a decision

1. Use the existing issue for the affected work. In the same change, add a
   concise issue comment linking the record and explaining the decision.
2. Create a unique `YYYY-MM-DD-<issue>-<topic>.md` file in this directory.
   Independent decisions, including separate decisions on one issue, get
   separate files. Update an existing record only within its owned scope.
3. Link the issue and explain the concrete decision, its scope, the reason,
   and any material tradeoff. Keep reproductions, evidence, and unfinished
   work discoverable through links to their actual source or issue.
4. Write readable paragraphs. Each authored Markdown file keeps the existing
   350-line budget. Split a long new record into named companions and link
   them from its entry point rather than compressing prose into giant rows.
5. Link any decision that the new record supersedes and explain the exact
   changed scope. A later date alone does not establish precedence.

The foundation remains the architectural starting point. Existing history,
order, designs, and issue records retain their paths and reasoning. A record
describing a proposal or unfinished work does not prove implementation,
merge, or release completion; use its actual evidence and issue state.

This workflow applies after its change merges. Preserve the existing paired
records of PRs already in flight; their original clauses need no migration.
Historical source receipts continue to describe their original source bytes.
