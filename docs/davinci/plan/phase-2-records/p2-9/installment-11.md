# P2-9 Installment 11 — Hydrated P1-9 residual re-measure (2026-08-30)

> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget.

This closes the open P2-9 evidence gap from installment 10. The prior
run proved the witness binary and flags but did not measure the corpus
because the fixture gitlinks were unhydrated. This run uses a fully
hydrated `tests/_fixtures/_git` checkout and records the number rather
than carrying forward a prediction.

`crates/vize_s1_to_s2/src/emit/**` remains untouched. The old expression
rewrite subtree remains outside P2-9's implementation scope by contract;
this installment only measures whether the S2-pass ports shrank its
retained-`None` residual.

## Command

Run from `/Users/ubugeeei/Source/github.com/ubugeeei/vize--p2-9-corpus`
after fast-forwarding the branch to `1a717a95949be4f59b0ad43d4b00241b905fcd7c`:

```sh
VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git \
  cargo test -p vize_atelier_sfc --features davinci-differential \
  --test davinci_differential -- --nocapture
```

## Scope proof

```text
davinci-differential corpus scope: root=/Users/ubugeeei/Source/github.com/ubugeeei/vize--p2-9-corpus/tests/davinci_test_support/../../tests/_fixtures/_git scope=canonical closure_evidence=true submodules=146
davinci-differential corpus sweep: files=41580 compiled=41580 transform_rewrite_comparisons=801305 codegen_rewrite_comparisons=1 vapor_resolve_comparisons=167578 shape_comparisons=152542
davinci-differential corpus prefixer split: admitted=801305 legacy_total=106532 legacy_params=16123 legacy_unretained=82645 legacy_dialect_rejected=6874 legacy_ts_strip_rewrote=890 admitted_pct=88.27
davinci-differential totals: transform_rewrite_comparisons=802713 codegen_rewrite_comparisons=1 vapor_resolve_comparisons=167642 shape_comparisons=152794
test differential_lane_agrees_and_reports ... ok
test result: ok. 1 passed; 0 failed; finished in 343.44s
```

The corpus run compiled **41,580** Vue files from the canonical
146-submodule fixture inventory and the differential lane reported zero
divergence.

## Residual

Residual percent is:

```text
100 * legacy_total / (admitted + legacy_total)
= 100 * 106532 / (801305 + 106532)
= 11.73%
```

The retained-`None` class therefore did **not** shrink below the P1-9
baseline on this hydrated corpus. The class mix is:

- `legacy_params=16123`
- `legacy_unretained=82645`
- `legacy_dialect_rejected=6874`
- `legacy_ts_strip_rewrote=890`

That answer completes P2-9's measurement obligation and gives P2-5b the
current number for any later expression-contract widening. It does not
delete `steps/expression/`, retire the in-phase transform flag, or move
the DOM backend; those remain later phase-exit / P2-11 decisions.

## House

New record only plus ledger/checklist updates. No assertion-allowlist
change. No emit path change. No corpus inventory drift: the executable
manifest still owns the 146 gitlinks / 142 ecosystem projects count.
