# Vize is a proof-carrying toolchain

The maintainer set the goal on 2026-11-25: Vize is to be a proof-carrying
toolchain in earnest, with excellent design and excellent performance at the
same time. This record turns that goal into rules. It builds on the
[assurance doctrine](../assurance.md), the
[Creusot decision](./2026-11-17-creusot.md) and its
[inventory](./2026-11-17-creusot-inventory.md), and the
[unsafe model-checking record](./2026-11-24-unsafe-model-checking.md). It
extends the `formal.rust-only` gate of
[#10825](https://github.com/ubugeeei-prod/vize/issues/10825). It does not
replace any law or verifier those records name.

## What "proof-carrying" means here

A guarantee counts when a checker that runs in CI proves it. That checker is
Creusot, Kani, or a Lean, TLA+ or Alloy model with a Rust refinement lemma.
The guarantee must also hold for the code the product ships, not for a copy.
Tests and fuzzing stay as oracles, but they do not make a guarantee
proof-carrying.

The program aims at guarantees a user can rely on, not at isolated lemmas.
Each guarantee below is built from laws in the inventory. Its status is the
weakest status among its laws.

| Guarantee                                                                  | Today                                                                                                                        |
| -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| The reactivity lattice and its classifier are sound and least              | proved (Creusot, all 12 lattice rows)                                                                                        |
| The HTML content-model checker has no false positive and no false negative | proved (Creusot, all 8 rows, chains of every length)                                                                         |
| The L3 schedule contract and the C-23 checker are exact                    | proved (Creusot, 20 rows, programs of every size)                                                                            |
| Spans, ranges, UTF-16 columns and VLQ are correct                          | Spans, ranges, mappings: proved (Creusot, every `u32`/`i32`). UTF-16 columns, VLQ, `to_generated`: `kani`, ports in progress |
| Incremental updates equal recomputation                                    | `kani` with fixed tree shapes (L3 reference machine)                                                                         |
| Every product `unsafe` block is sound                                      | 45 sites inventoried. 16 are model-checked or proved. The rest are covered by fuzzing, Miri or tests, or are FFI             |
| Lossless L1 render equals source                                           | owed                                                                                                                         |
| The formatter is idempotent and preserves meaning                          | owed                                                                                                                         |

A guarantee moves from `kani` to proved only by an unbounded proof. Until then
the bound is stated next to it.

## Design rules

Proofs are cheap when the code has the right shape. The same shape is also
the fast and readable one, so these rules are design rules, not proof
workarounds.

- **Pure kernels and thin adapters.** Every decision a guarantee depends on
  lives in a function over bytes, integers and plain data. The adapter
  around it handles arenas, `dyn` dispatch, `str` APIs, I/O and AST walks. A
  kernel that cannot be specified has the wrong boundary. The fix is to move
  the boundary, not to weaken the law.
- **The proof sits next to the kernel.** A `creusot.rs` or `kani.rs` root
  beside the kernel includes the shipped modules by ordinary module
  discovery. There are no copies, no `#[path]` and no reimplementations.
- **Specifications come from the declarative source.** The source is the
  HTML or ECMAScript specification, the Unicode standard, the phase plan or
  the Lean model, never the Rust body. Each proof comes with mutants: broken
  kernels that fail the expected goal, so that no contract is vacuous.
- **Nothing is trusted.** No `#[trusted]`, no `extern_spec!`, no axioms and
  no assumptions. An item outside a law is compiled out of the proof crate
  and listed with its Rust oracle.

## Proofs are free, and they may buy speed

Annotations never reach the product build. An annotation-only change shows
identical LLVM IR at `opt-level=3` with one codegen unit, plus an instruction
count run. Making a kernel provable may require a rewrite, as with
`classify` and `walk` in the content-model kernel. Such a rewrite needs the
no-regression evidence of the fleet rules instead: instruction counts against
base, interleaved A/B wall time with an A/A control, and byte-identical
output. The maintainer decides any trade-off case by case.

A proof may also remove a runtime check. Examples are a bounds check that a
proved invariant makes redundant, and a UTF-8 revalidation of bytes that are
proved valid. Removing the check is allowed when all of these hold:

1. The invariant is proved in CI on the shipped code, unbounded unless the
   domain is finite.
2. The unchecked operation's precondition is the proved fact. Its
   `// SAFETY:` comment names the lemma, and the unsafe record lists the
   lemma as the site's verifier.
3. The speed-up is measured with the same evidence as any performance change.

## Choosing the tool

Creusot is the default, because its proofs cover every input. Kani is for
small, closed models: integer arithmetic, fixed domains and single unsafe
steps. A Kani harness states its bound and has `kani::cover!` for its
interesting cases. A law shaped like a string or a loop belongs in Creusot.
Lean, TLA+ and Alloy describe protocols and abstract models. Each model that
a product relies on needs a Creusot lemma that states the same law on the
Rust kernel, and the model's record names that lemma.

## CI budget

Formal verification must never become the CI bottleneck. Measured on main on
2026-11-25:

- Each Kani job takes between 30 s and 7 min.
- The Creusot job takes 7 min 24 s, of which the cached toolchain restore is
  about 6 min.

These budgets apply:

- A Kani job finishes in five minutes and its `timeout-minutes` is at most 10. Locally every harness passes under one cbmc at a time, 4 GB and
  10 minutes. A harness that exceeds that is shrunk, moved to Creusot or
  recorded as owed. It is never moved to CI only.
- The Creusot job's `timeout-minutes` is at most 30, which covers a cold
  toolchain build. A new proof package adds at most 60 s to the replay, and
  the per-goal limit in `why3find.json` is not raised.
- Formal work adds jobs to the existing Kani and Creusot workflows, filtered
  by the paths of the kernels it covers. It adds no new workflow files.

`tests/tooling/formal-ci-budget.test.ts` holds the timeout limits for every
generated job that runs Kani or Creusot. On the day of this record, one job
is over the five-minute target: style-module products "Authored coordinates
and style header proofs", which takes 6 min 56 s. Most of that is the
`style_format` Kani harness. It is the first port to Creusot in the
product-crate track.

## Merging

The maintainer said formal work, including records like this one, merges on
green CI through the merge queue, with the evidence in the PR body. Work that
would break a budget above, or regress a metric, goes back to the maintainer
instead.
