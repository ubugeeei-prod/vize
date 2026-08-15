# Scanner parity report (P1-8)

> [!NOTE]
> Pre-deletion differential for P1-8 "Delete the fast/slow scanner split":
> the byte scanner (`crates/vize_croquis/src/drawer/helpers/identifiers/fast.rs`)
> dual-run against the AST walk that would replace it — one oxc
> `parse_expression` of the exact bytes the scanner consumed, walked by the
> same `walk_expr` the slow path runs, which is the post-deletion behavior
> by construction. Comparators armed by the P1-6 differential machinery
> extended to the fast dispatch class (recording divergences into a
> deduplicated inventory instead of panicking — the decision gate consumes
> the complete inventory, not the first mismatch). Reproduce with the
> P1-6 corpus command plus the P1-8 env vars:
> `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git` and
> `VIZE_DAVINCI_FAST_DIVERGENCE_OUT=<file>` on
> `cargo test -p vize_croquis --release --features davinci-differential`
> `--test davinci_differential -- --nocapture` (one command; wrapped
> here). The harness extension is part of the P1-8 working set and lands
> with the follow-up the waiver decision selects; this report is committed
> alone per the decision gate.

## Verdict (2026-08-15)

**Divergent — the deletion is blocked by the decision gate.** Over the
hydrated ecosystem corpus (41 580 `.vue` files analyzed), 3.85% of
fast-dispatch identifier extractions and 4.24% of fast-dispatch
identifier-ref extractions disagree between the byte scanner and the AST
walk: 4 914 distinct expression texts, 29 109 divergent extraction calls.
The majority class (bound keyword literals such as `false`) is filtered by
every consumer, but four classes reach observable output — the scanner
extracts assignment targets, statement tails, and numeric-literal alpha
tails the walk does not, and the walk extracts spread arguments the scanner
does not. Deleting the split without a waiver would change corpus-visible
bytes (croquis used/undefined tracking, canon anchor narrowing and
`$`-global mappings, maestro used-binding sets). Divergences are bugs in
one side — several in each side — and the ownership split is exactly the
waiver review this task's plan step anticipated.

## What was compared

- **Scanner side**: the production fast-dispatch result — the byte scanner
  over comment-stripped text for the name entries
  (`extract_identifiers_oxc`, `extract_identifiers_retained`) and over raw
  text for the offset entry (`extract_identifier_refs_oxc`).
- **Walk side**: one oxc `parse_expression` of the **same bytes** in an
  uncounted arena, walked by the slow path's `walk_expr`. Where the node
  carried a comment-free retained AST (P1-5 `js_ast`), the retained walk
  was additionally asserted equal to the one-parse walk — 314 108
  crosschecks, zero disagreements (parser determinism, the P1-6 proof
  shape), so "retained AST where available, one parse for the rest"
  collapses to a single behavior.
- **Input universes**: every drawer extraction during croquis analysis
  (`SfcCroquisOptions::full()`) of every corpus file, plus the two exported
  surfaces croquis does not call itself, driven over exactly the inputs
  their production consumers feed them: `extract_identifier_refs_oxc` over
  every recorded template expression (canon `instance.rs`) and
  `extract_identifiers_oxc` over every CSS `v-bind()` expression (canon
  `css_var_usage`). The committed battery + P0-2 ladder run first with
  exact-pinned counts; the same run reproduced the P1-6 lanes' corpus
  numbers (37 260 identifier, 1 649 component-reference comparisons, zero
  divergence; 13 235 v-for shapes), anchoring harness validity.

## Corpus counts (release, single sweep, 2026-08-15)

| lane                             | comparisons | divergent | share |
| -------------------------------- | ----------: | --------: | ----: |
| identifiers (names)              |     327 806 |    12 607 | 3.85% |
| identifier refs (names\@offsets) |     389 308 |    16 496 | 4.24% |
| retained-AST crosschecks         |     314 108 |         0 |    0% |

Committed-fixture pins (exact, the fast-lane ratchet): battery 9
identifier + 7 ref comparisons, zero divergent, 7 crosschecks; battery +
ladder cumulative 725 identifier + 753 ref comparisons, 1 + 5 divergent —
the P0-2 ladder's single diverging text is a bound `false` literal, the
majority class in miniature.

## Divergence classes

Distinct texts and per-lane divergent calls over the whole sweep — battery,
ladder, and corpus together. Classes 2, 4, 5, 6, 7 reach output bytes;
classes 1 and 3 are consumer-inert by audit (not by construction).

| #   | class                      | texts | id calls | ref calls |
| --- | -------------------------- | ----: | -------: | --------: |
| 1   | keyword-literal tokens     | 1 925 |    6 732 |     9 893 |
| 2   | assignment-target loss     | 2 450 |    4 815 |     5 496 |
| 3   | order/multiplicity/offsets |   443 |      926 |       991 |
| 4   | multi-statement tail loss  |    38 |       38 |        39 |
| 5   | spread-argument recovery   |    49 |       85 |        69 |
| 6   | walk parse failure         |     5 |        6 |         7 |
| 7   | numeric-literal alpha tail |     2 |        3 |         3 |
| 8   | mixed (2 + 5)              |     2 |        3 |         3 |

### 1. Keyword-literal tokens (scanner bug, consumer-inert)

The scanner emits any identifier-shaped token not preceded by `.`, so bare
keyword literals come back as "identifiers"; the walk sees literal nodes
and correctly emits nothing. Repro: `false` → scanner `["false"]`, walk
`[]`. Token census over the class (a call can carry several): `false`
8 175, `true` 5 508, `null` 2 354, `new` 378, `typeof` 370, `in` 207,
`void` 68, `this` 54, `instanceof` 20, `delete` 2. Every consumer filters
or collision-guards these — croquis `check_expression_refs` drops them via
`is_keyword`, canon documents the defense outright
(`is_safe_value_identifier`: "Literal tokens can surface from expression
identifier extraction"), and a reserved word can never name a script
binding — so the class changes no output bytes today. It is still a
raw-surface divergence: post-deletion the noise stops being produced, and
the unit pin `test_extract_identifiers_ignores_comment_words`
(`["disabled", "true", "undefined"]`) stops holding.

### 2. Assignment-target loss (walk bug, output-visible)

`walk_expr` walks only the right side of `AssignmentExpression`, so the
target name vanishes. Repro: `x = 0` → scanner `["x"]`, walk `[]`;
`step += 1` → scanner `["step"]`, walk `[]`. Corpus-heavy (identifier-lane
calls): `model = false` 111, `show = false` 82, `show = true` 77,
`visible = true` 75. The
scanner-side name is the one production consumes: croquis marks the target
used (v-for/slot scope usage) and reports it when undeclared; canon's
template-referenced-name sets (`spans.rs`, `css_var_usage`) keep the
`void <binding>;` anchor that suppresses `TS6133`; maestro's
used-script-bindings set includes it. Post-deletion all of those lose the
name — unused diagnostics appear, undefined-ref diagnostics disappear.
Note the walk is arguably the buggy side here (Vue's own transform
rewrites assignment targets as references), and the same drop already
ships for slow-dispatch text — fixing it is also a corpus-visible change,
just in the opposite direction.

### 3. Order / multiplicity / offsets (conditionally visible)

Same name set, different sequence or spans. Repro: `show = !show` →
scanner `["show", "show"]` (`show@0`, `show@8`), walk `["show"]`
(`show@8` only). Croquis re-derives offsets from content (first
occurrence) and `mark_used` is idempotent, but duplicate undefined-ref
rows collapse, and canon `instance.rs` maps `$`-names at the extracted
ref's own span — first-occurrence spans shift to the surviving occurrence.

### 4. Multi-statement tail loss (walk hazard, output-visible)

oxc `parse_expression` parses the first expression and does **not** fail
on trailing statements, so multi-statement v-on bodies silently lose every
identifier after the first statement. Repro: `close();modifyInventory()` →
scanner `["close", "modifyInventory"]`, walk `["close"]` with no parse
error; `$emit('selected', $event); close()` keeps `$emit`/`$event`, drops
`close`. A body whose `$`-global sits in the tail loses its canon mapping.

### 5. Spread-argument recovery (scanner bug, output-visible)

The scanner treats a name after `.` as a property access, and `...x` ends
with `.`, so spread arguments disappear. Repro: `[...data]` → scanner
`[]`, walk `["data"]`. The walk restores the reference — usage tracking
and anchors gain names, so today's false "unused" outcomes flip.

### 6. Walk parse failure (output-visible)

Text that parses as no single expression returns empty from the walk while
the scanner still harvests names: bare `class` (4 calls), statement bodies
starting with a keyword (`if (active) toggle(); splitView = true;`), a
leading-semicolon body, and one Pug-leaked interpolation.

### 7. Numeric-literal alpha tails (scanner bug, output-visible)

The scanner starts an "identifier" at the first alphabetic byte inside a
numeric literal. Repro: `[1e22, 3e30]` → scanner `["e22", "e30"]`, walk
`[]`. Today those are undefined-ref **false positives** on real corpus
files; the walk removes them — a fix, and still a byte change.

## The explicitly hunted classes

- **Regex literals / division / string literals containing `//`**: contain
  `/`, which the dispatch heuristic sends to the slow path — structurally
  outside the fast class; 0 inventory rows contain `/`.
- **`strip_js_comments` regex-mangling** (`/[/*]/.test(x)`, the P1-6
  finding): orthogonal to the split. The stripper runs **before** dispatch
  and the planned single implementation keeps stripping before its one
  parse, so the mangled-then-failed parse behavior is byte-identical pre-
  and post-deletion. Deleting the scanner neither fixes nor changes it;
  fixing the stripper is its own waiver-reviewed change.
- **Template literals**: `${` contains `{` → slow path; interpolation-free
  literals stay fast and both sides skip their contents (the two backtick
  inventory rows diverge on a bare `null`, not the literal).
- **Unicode identifiers**: non-ASCII text → slow path by dispatch; 0
  non-ASCII inventory rows.
- **Keywords as property names**: `foo.new`-shaped members agree (both
  sides yield `["foo"]`); every keyword-after-dot inventory row diverges
  for an unrelated reason (bare literals elsewhere in the text).

## What deletion needs before it can merge

Two coherent resolutions, both waiver territory, both re-runnable through
this harness:

1. **Accept walk semantics as-is** — waive classes 1–8 wholesale. Output
   bytes change on the corpus exactly as classified above (class 2's lost
   usage tracking is the big one: 2 450 texts).
2. **Fix the walk first** — teach `walk_expr` assignment targets and teach
   the single implementation statement sequences (classes 2, 4, 6), then
   delete the scanner and waive only the keyword-token noise (class 1),
   the duplicate/offset artifacts (class 3), and the scanner-bug fixes
   (classes 5, 7). Note both fixes also touch **slow-dispatch** behavior
   that ships today, so they carry their own corpus diffs and cannot ride
   the deletion as byte-neutral refactors.

Either way the P1-8 bench condition (walk closes the scanner's `stress-*`
gap) is unmeasured until a deletion variant exists to measure.
