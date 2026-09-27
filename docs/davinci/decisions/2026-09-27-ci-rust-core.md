# Affected Rust archive and shard wiring

Tracked in [#6861](https://github.com/ubugeeei-prod/vize/issues/6861),
[#6862](https://github.com/ubugeeei-prod/vize/issues/6862) and
[#6863](https://github.com/ubugeeei-prod/vize/issues/6863).

Wire the affected Rust plan to one nextest archive and four independent PR
shards. The merge queue retains the complete timed workspace test phases,
required TSGO environment, shared differential feature tail and fixture checks.
The strict Rust report accepts success only from every required executing job.

Install the declared Node runtime before source planning and Rust 1.98.0 only
when the coarse plan requires Cargo metadata. Restrict documentation exemptions
to Markdown so a Rust file under docs still receives conservative validation.

For PRs, use the tested merge's first parent only when the full checkout SHA,
exactly two parents and matching payload head prove the relationship. Read
commit headers before fetching so shallow checkouts can prove the base. Fetch
that exact base and share it across source and Rust planning; otherwise retain
the event base. Merge groups retain their event base and full validation.
Synthetic shallow Git tests cover stale event bases and rejected relationships.

Keep this Rust change separate from tooling selection, inventory tiers,
workflow security selection and generated ledgers. Existing JS, tooling and
playground work remains unchanged in this intermediate branch. Preserve the
baseline decisions and split source assurances with move-only commits.

Actual archive/runtime and PR latency require fresh Actions evidence. The
recorded phase baseline and executable fake-Cargo tests prove timing behavior;
they do not establish a speedup or native product completion.

Focused verification on this intermediate branch: 44 executable planner,
archive, timing and workflow tests pass with no skips or cancellations. Root
`vp check` passes on all 15 changed JS/TS/MJS files with zero warnings or errors.
Both changed workflows pass actionlint, and the 350-line growth ratchet passes
against `8ae28f16c`. Fresh PR Actions and actual archive/shard execution remain
required before merging.
