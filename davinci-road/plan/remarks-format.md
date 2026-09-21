# Optimization remarks — contract and vocabulary

> [!NOTE]
> The P3-13 remark contract: what a remark is, how it travels, its two
> serialized forms, and the registered per-pass vocabulary. Code:
> `crates/vize_davinci/src/pass/observer/remark.rs` (the channel),
> `crates/vize_davinci/src/folio/remarks.rs` (the page and the JSON
> document). JSON schema: [`remarks.schema.json`](./remarks.schema.json).
> Record: [P3-13](./phase-3-records/p3-13.md).

## Shape

A remark is `{pass, kind, name, span, args}` — LLVM's optimization remark
with **structured** arguments (free-form strings are unfilterable, LLVM's
own regret, see [prior-art](../prior-art.md)).

| field   | meaning                                                                                                              |
| ------- | -------------------------------------------------------------------------------------------------------------------- |
| `stage` | the emitting pipeline's stage (`s2`)                                                                                 |
| `pass`  | the emitting pass — supplied by the pass manager from the running `PassEvent`, never by the pass itself              |
| `kind`  | `applied` / `missed` / `analysis` (below)                                                                            |
| `name`  | the remark's identity within its pass, from the registry below; lowercase kebab-case                                 |
| `span`  | the authored span the decision is about: byte offsets into the source the pipeline's artifact was lowered from       |
| `args`  | ordered `key=value` pairs; keys are static kebab-case and unique within a remark; values are string, integer or bool |

**Kinds.** `applied` — the pass performed the optimization it names, or,
for an analysis pass, established the fact that licenses it. `missed` —
the pass considered it and it did not apply; the args name the blocker.
`analysis` — a neutral explanatory fact with no verdict. The TS-32
remarks-diff reads an `applied → missed` transition as an optimization
regression; a `missed` remark is a mined backlog item (C-13).

## Channel

Remarks travel through the pass manager's observer, not a side channel.
`run_pipeline_remarked` hands each pass body a `PassRemarks` bound to the
running pass; `RemarkSink::emit` forwards to `PassObserver::on_remark`,
which fires between that pass's `before_pass` and `after_pass`. `Pair`
forwards to exactly the members whose `REMARKS` is true.

**Zero cost when nothing listens.** `PassObserver::REMARKS` is an
associated const (default `false`), so `RemarkSink::ENABLED` is known at
monomorphization time and a pass guards blocker classification and argument
construction on it. Under `NoObserver`, `BudgetObserver`, `TimingObserver`
or any composition of them the remark path compiles away: pinned as an
exact **zero** allocations under the counting allocator
(`crates/vize_davinci/tests/remark_zero_cost.rs`), with an attached control
that allocates exactly one label per pass. Pass bodies run outside the pass
manager (the production DOM path) take the detached `NoRemarks` sink.

## Canonical order

`RemarkCollector::finish` orders by the emitting pass's run position, then
span start ascending, span end descending (outer before inner), then remark
name; full ties keep emission order. The order is walk-independent, so a
changed line in a log diff is a changed decision, not a refactored walk.

## The `[remarks]` page

```text
[remarks]

[remarks.entries]
s2.hoist-static applied static-subtree @27:59 tag="h1"
s2.hoist-static missed static-props @60:99 tag="p" blocker="binding" op="ui.on"

```

One entry per line: `{stage}.{pass} {kind} {name} @{start}:{end}` then
` {key}={value}` per argument. Text values are JSON string literals (so they
may hold spaces and quotes); integers (canonical decimal, no `-0`) and
booleans are bare. Entry order is carried, never re-sorted — the collector
already made it canonical. An empty log is the bare `[remarks]` header.
Parse is strict except that any `\uXXXX` escape is accepted and normalized
by the first print; every rejection message is pinned by
`crates/vize_davinci/tests/remark_folio.rs`. `Display` drops the spans and
carries no round-trip law. `davinci-opt --roundtrip <file> --stage remarks`
checks canonicity.

## The JSON document

`RemarkLog::to_json(command)` — one line, key order `schema_version`,
`command`, `remarks`; each remark `stage`, `pass`, `kind`, `name`, `span`
(`start`, `end`), `args` (`[{key, value}]`). Consumers negotiate
`schema_version` (`RemarkLog::negotiate_schema_version`) before reading
anything else. `davinci-opt --remarks <path>` writes it for a pipeline run.

## TS-32 — the corpus remarks-diff

`crates/vize_s1_to_s2/tests/davinci_remarks_corpus.rs` (feature
`davinci-differential`, run by the required `clippy-and-test` job) sweeps
every `.vue` file under `tests/_fixtures` (the `_git` submodules and
`node_modules` excluded, so every checkout sweeps the same set), lowers each
inline HTML template and runs the S2 transform pipeline under a
`RemarkCollector`. Spans are shifted to file offsets. The result is a
`[remarks-corpus]` page — `files`, `entries` (`"path" <entry line>`),
`explained` — that must equal the committed
`tests/_fixtures/davinci-remarks-baseline.folio` byte for byte.

A difference fails with the keyed remarks-diff
(`vize_davinci::folio::remarks::diff`): identity is
`(path, stage, pass, name, span, occurrence)`, and a changed identity is
`regressed` (`applied → missed`), `improved`, `rekinded`, `reargued`
(same kind, blocker moved), `added` or `removed`. Re-bless with
`UPDATE_REMARKS_BASELINE=1`; **the bless refuses every `regressed`
identity not listed in `[remarks-corpus.explained]`** as
`"path" stage.pass missed name @s:e reason="..."`, which makes "no
unexplained applied → missed transitions" a machine check rather than a
review convention. A plain run also rejects an explanation that names no
baseline `missed` remark, so the ledger cannot rot.

## Spolvero and the backlog

The Spolvero feed (`spolvero-feed.schema.json`) carries a `remarks` member
since P3-13: each item is a remark document item plus the `path` it was
produced for, spans in the frame of that file's pages (template bytes for
the inspector and `analyzeSfc`). The member is additive to feed v1 —
every producer emits it (possibly empty), and it is optional in the
schema so earlier v1 documents stay valid. Producers: the inspector payload
and wasm `analyzeSfc` (`vize_curator::inspector::template_remarks`, one run
of lowering + the S2 transform pipeline per inline HTML template) and
`davinci-opt --folio-dir` (the run's own log). This is what the playground's
decision view renders (C-5).

**C-13 backlog.** `vize_davinci::folio::remarks::backlog::mine_missed`
groups a corpus's `missed` remarks by `(stage, pass, name, reason)` and
ranks them by hits. It relies on one vocabulary rule: **a remark's first
argument is its subject** (what the decision is about — `tag`,
`component`); the remaining arguments are its **reason**. The TS-32 test
renders the backlog of its corpus into
[`remarks-backlog.md`](./remarks-backlog.md) and pins it exactly, so the
backlog is always the one the committed baseline implies.

## Vocabulary registry

Every remark name a pass may emit, with its arguments in emission order —
subject first. Adding a name or an argument key is a registry change in the
same PR.

### `s2.hoist-static` (`crates/vize_s1_to_s2/src/pass/hoist/remarks.rs`)

An analysis pass: `applied` means the hoist-licensing fact holds; whether
DOM realization then hoists is position/option-dependent and not claimed.

| name             | emitted for                                                    | applied when        | args                                                                          |
| ---------------- | -------------------------------------------------------------- | ------------------- | ----------------------------------------------------------------------------- |
| `static-subtree` | every `ui.element`                                             | level `FullyStatic` | `tag`; missed adds `blocker`, then `op` and `rule` where they apply           |
| `static-props`   | every owner not whole-hoistable with a non-empty props surface | `props_hoistable`   | `tag` (element) or `component` (component name); missed adds the same blocker |

Blockers, in the lattice's own evaluation order:

| `blocker`       | `op`                                                                                                                                         | `rule` (only for `op="ui.bind"`)                                       |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `svg-directive` | —                                                                                                                                            | —                                                                      |
| `ref-attribute` | —                                                                                                                                            | —                                                                      |
| `binding`       | the first unhoistable binding's mnemonic                                                                                                     | `dynamic-name`, `modifier`, `reserved-key`, `no-value`, `non-constant` |
| `child`         | the first child op making the subtree dynamic (`ui.element`, `ui.component`, `ui.if`, `ui.for`, `ui.slot`, `ui.comment`, `ui.interpolation`) | —                                                                      |

Blockers come from the same functions the facts do
(`consts::binding_blocker` / `props_blocker`, the region summary), so a
remark cannot explain a decision the analysis did not make. TS-17 pins the
full page for the `hoist` fixtures and re-derives every kind from the
published facts (`crates/vize_s1_to_s2/tests/hoist_pass_remarks.rs`).

### `s3.extract-placements` (`crates/vize_impeto/src/extract/report.rs`)

P3-10 try-measure-commit extraction, run as the second pass of the `s3`
`OPTIMIZE` pipeline (`crates/vize_impeto/src/optimize.rs`). One remark per
recorded placement candidate, named after the placement; `applied` exactly
when the candidate was measured and committed.

| name    | emitted for                                   | applied when | args                                                                     |
| ------- | --------------------------------------------- | ------------ | ------------------------------------------------------------------------ |
| `hoist` | every `hoist` alternative `annotate` recorded | committed    | `reason`, `emitted-size`, `reactive-edges`, `update-path`, `budget-left` |
| `cache` | every `cache` alternative `annotate` recorded | committed    | same                                                                     |
| `group` | every `group` alternative `annotate` recorded | committed    | same                                                                     |

`reason` is `committed`, `regressed-reactive-edge`, `regressed-update-path`,
`regressed-emitted-size`, `no-improvement`, `budget-exhausted`,
`not-contiguous`, or `subsumed`. The three metric args are the signed changes
the trial measured (0 when the candidate was not measured); `budget-left` is
the component's remaining candidate budget. The remarks mirror the
`[s3-extraction-folio]` decision rows one for one; TS-17 pins both pages
(`crates/vize_s2_to_s3/tests/extraction_snapshots.rs`), and a detached run
must return the identical extraction.
