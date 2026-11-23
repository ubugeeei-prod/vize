# Sharing original Vue passes on one editor revision

A compiler pass, lint rule and editor request can all need the same template while its stylesheet has an independent error. Parsing the whole file for each consumer repeats work. Requiring every sibling to be valid also prevents a user from getting feedback on the part they are editing.

Maestro's opt-in `run_vue_passes` captures one open revision and passes its source, L1 descriptor and temporary arena to the caller. A template consumer selects the original template block. Its L1 syntax retains whitespace, comments and malformed text. L2 exposes compiler observations, complexity contributions and their original byte spans. A style consumer checks the stylesheet independently. A caller that needs only the descriptor can use `run_vue_descriptor`. The `experimental-component-passes` Cargo feature enables this host API on its own.

## Source ownership and pass locality

Roslyn represents each edit as a new immutable syntax snapshot and preserves skipped or missing tokens. Rust-analyzer keeps full-fidelity syntax separate from semantic analysis and returns syntax even for invalid input. Davinci uses that distinction at the Vue block boundary. A recoverable template can still round-trip while an independently valid stylesheet produces CSS. A stylesheet whose content lives in another file can leave the local template available. [Roslyn syntax model](https://learn.microsoft.com/en-us/dotnet/csharp/roslyn-sdk/work-with-syntax) · [rust-analyzer syntax design](https://rust-analyzer.github.io/book/contributing/syntax.html)

MLIR attaches analyses to operations and makes preservation explicit at pass boundaries. The shared Vue descriptor identifies the source each pass may read. The template pipeline then runs the static analysis and complexity analysis over the same L2 traversal. The lifetime of the callback keeps pointers into the descriptor and its arena inside that request. [MLIR pass infrastructure](https://mlir.llvm.org/docs/PassManagement/)

Adapton separates demand from change propagation. Mokhov, Mitchell and Peyton Jones separate scheduling a task from deciding whether it needs rebuilding. These distinctions motivate two independent decisions here. The caller chooses which passes to execute. The resident graph tracks reuse across revisions. The host separately checks whether a completed result can still be published on the current document. [Adapton, PLDI 2014](https://matthewhammer.org/adapton/) · [Build Systems à la Carte, ICFP 2018](https://simon.peytonjones.org/assets/pdfs/build-systems-original.pdf)

## Publication and evidence

The server assigns each open or updated document its own revision. Closing and reopening the URI can reproduce the same bytes, language and client version. A completed answer still carries its original server revision and cancellation handle. Publication checks both while the current document is read-locked. The guarded operation ends when the synchronous host callback returns.

The TLA+ model explores a single URI, one request, up to three server revisions, two template states and two stylesheet states. An edit, close, reopen or cancellation can occur between pass steps and before publication. TLC 1.7.4 explored 1,248 distinct states. Using only the client version admits an obsolete result after reopening. Omitting the final cancellation check admits an aborted result. A third witness reaches successful publication of a syntax error together with compiled CSS. The model records freshness at the publication decision. The Rust tests exercise the corresponding host actions, and a compile-fail example checks the callback's borrowing boundary. [Request model](../../../tests/formal/ir-graph/Request.tla) · [Editor request tests](../../../crates/vize_maestro/tests/original_template_requests.rs)

## Cost of a complete editor request

The Apple arm64 benchmark starts with an open document. Both executions return the original template, compiler observations, the complete complexity warning, mapped CSS, a verified formatter edit, type diagnostics, hover and a stylesheet definition. The baseline publishes script, style and template queries separately. The combined execution publishes the same outputs from one descriptor. Each timing uses 31 paired samples with alternating order. The allocation counter also records peak live bytes. [Benchmark and output comparison](../../../crates/vize_maestro/tests/component_tool_passes.rs)

| Template elements | Separate / combined ns | Allocation calls |   Peak live bytes |
| ----------------- | ---------------------: | ---------------: | ----------------: |
| 1                 |        24,257 / 22,427 |         100 / 93 |   38,147 / 38,115 |
| 20                |        59,723 / 56,367 |        321 / 315 |   68,024 / 68,024 |
| 64                |      145,677 / 138,393 |        769 / 763 | 153,693 / 153,693 |
