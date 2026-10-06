# Root comment attachment: original and finite current references

`original-issue.md` and `Example.vue.txt` retain the full original #7877 report
and its 141-byte carrier. `cases.json` has 20 whole inputs and independently
declared complete outputs. The two `.historical.snap.txt` files retain the
old root-comment snapshots byte-exactly; the original differential capture
inputs/outputs remain unchanged in their existing paths.

Only the original input's complete corrected current output can pass. The two
historical cases record a separate strict `different` comparison, while the
current output must be `equal`; old wrong blank lines are not an alternative
reference. All unrelated output and the existing #7826 current reference stay
unchanged. The #6694/#3346 controls reuse their whole existing source/output
and immutable byte hashes, not fresh captured output.

The Rust fixture requires full first/second/third output equality and `changed`
truth. The CLI fixture requires a current-source build receipt, original check
status, three complete writes and a successful final read-only check. Its
report retains full outputs, stdout/stderr, status and signal for every process.
Native Davinci formatter handling remains unsupported. Prepared fixtures alone
grant no execution, performance, protected merge or release credit.

The first twenty complete inputs/current expectations remain exact. Three
independently authored no-block controls (two comments with a blank separator,
raw text plus a comment, and BOM comments) retain the complete document with no
attachment consumer. The current whole corpus is 23 cases/115 CLI captures,
with the same three-pass/check/process assertions and two historical mismatches.
