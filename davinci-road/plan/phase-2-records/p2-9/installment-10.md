# P2-9 Installment 10 — P1-9 residual re-measure (2026-08-24)

> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget.

The P2-9 contract still asks whether region-structured lowering
shrunk `steps/expression/reparse.rs`'s residual class. The number
must come from the existing `retained::differential` counters, not a
prediction. Installments 1–8 last published **12.73%** from a hydrated
corpus; this installment re-ran the witness rather than copying that
figure.

`phase-2.md` stays open. `crates/vize_s1_to_s2/src/emit/**` untouched.

## The witness that produces the number

```sh
VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git \
  cargo test -p vize_atelier_sfc --features davinci-differential \
  --test davinci_differential -- --nocapture
```

The binary (`crates/vize_atelier_sfc/tests/davinci_differential.rs`)
prints, after the corpus sweep:

```text
davinci-differential corpus prefixer split: admitted=… legacy_total=…
  … admitted_pct=…
```

Residual percent is `100 * legacy_total / (admitted + legacy_total)`.
Installments 1–6 recorded that as admitted 196,236; legacy 28,636 of
224,872 (**12.73%**). This installment does **not** repeat 12.73% as
a new measurement.

## What ran here

Without the env var (battery + P0-2 ladder only), 2026-08-24:

```sh
cargo test -p vize_atelier_sfc --features davinci-differential \
  --test davinci_differential -- --nocapture
```

Green. Stderr: `transform_rewrite_comparisons=1408` (the committed
ladder pin). That is **not** the corpus residual.

With the env var pointed at `tests/_fixtures/_git`, the same binary
panicked at `davinci_differential.rs:241`:

```text
corpus sweep found no .vue files under
  /workspace/crates/vize_atelier_sfc/../../tests/_fixtures/_git
```

Mechanical reason: the 146 fixture paths are gitlinks
(`git ls-files -s tests/_fixtures/_git` → mode `160000`) and are not
hydrated in this worktree (`find tests/_fixtures/_git -name '*.vue'`
→ 0). Hydration is `git submodule update --init --depth 1 --
tests/_fixtures/_git/<id>` per project; this environment does not
carry the submodule bodies. A partial hydrate would be a different
hydration state (P2-5b already showed 11.73% vs 12.73% across
hydrations) and is not a substitute.

## Did the class move?

**Unknown this run — the corpus counters did not execute.** Do not
read "unmoved 12.73%" off installments 1–8. Filter wrapping and the
installment-9 text-projection wrap-equals live on S2 `ExprRef`s after
lowering; they still do not feed `rewrite_expression`. That is why
a movement was unlikely, and also why it must not be claimed without
the sweep.

## What remains

Re-run the env-var command on a fully hydrated
`tests/_fixtures/_git` and replace this record's "unknown" with the
printed `admitted` / `legacy_total` / residual percent. Until that
number exists, the series checkbox in `phase-2.md` stays open.

## House

New file only plus one table row and one series-log line. ≤ 350
lines. No assertion-allowlist change. No emit/.
