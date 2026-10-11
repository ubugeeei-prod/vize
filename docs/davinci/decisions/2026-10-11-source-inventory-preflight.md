# Decision: validate source inventories before CI build fanout

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Implementation starts from main `e23c2911648907ec78e51e9d13ad555af2432dd9`.

The PR and merge-queue source planner runs the existing
`check-level-inventories` composite action without changing its commands,
followed by the original `davinci-module-layout.test.ts` control. Both run after
the declared Node runtime and authenticated comparison-base fetch, before source
selection, Rust toolchain setup and the existing four-lane source fanout.

An inventory or ordinary-module failure fails `pr-source-plan`. The Rust, JS
package, tooling and playground source jobs already require that plan to succeed;
their dependency conditions and source-report failure handling remain unchanged.
The regression law checks this actual workflow boundary and rejects a failed plan
through the original source-report implementation for PR and merge-group events.

The scheduled/manual `check-js` inventory caller remains. Its merge-group call
moves into the source plan, so a queue run has one inventory ledger uploader.
PRs gain the same early inventory check. The complete original tooling suite,
inventory assertions, module-layout negative controls, generated ledgers and
artifact rules remain. No new workflow job, workspace Rust build or JS dependency
installation is added to the planner.

## Evidence for the ordering

The following are existing terminal failed Actions runs, before this change.
Identities are the original run metadata's head SHAs; synthetic PR checkouts can
have a different SHA from the PR source branch.

| Run                                                                                   | Metadata head SHA                          | First visible relevant failure                                  | Successful build steps already completed                      |
| ------------------------------------------------------------------------------------- | ------------------------------------------ | --------------------------------------------------------------- | ------------------------------------------------------------- |
| [38117117581](https://github.com/ubugeeei-prod/vize/actions/runs/38117117581) (#8595) | `56320135b98de0e9c348b2f44b8ba96d586b1410` | Consumer inventory: 06:12:56 UTC; ordinary module: 06:13:35 UTC | Rust archive 109 s; Clippy 28 s; four tooling CLI steps 220 s |
| [38117521055](https://github.com/ubugeeei-prod/vize/actions/runs/38117521055) (#8596) | `940c630fea2176281ffcf7697cf051adee404418` | Croquis and consumer inventories: 06:23:04 UTC                  | Rust archive 117 s; Clippy 28 s; four tooling CLI steps 223 s |

The original module-layout assertion took 291.933 ms in #8595. The original
Croquis and consumer inventory assertions took 4416.396 ms and 731.898 ms in
#8596. These are individual tooling-test durations, not measurements of the whole
composite action. Build-step wall times overlap across runners and must not be
reported as CPU time or an end-to-end speedup.

This ordering prevents those source lanes from starting after a failed preflight.
Independent Check jobs can still run. Fresh exact-head Actions and protected
merge acceptance are required; no measured speedup or release completion is
claimed from these historical failures.

## Source-length gate

The original 350-line ratchet stays in its current tooling suite. Its Rust helper
is not an existing planner executable, and adding `rust-script` installation or a
helper compilation would introduce a separate cost. Any later early placement
must measure that cost and retain the authenticated comparison base; a missing
base must never become a successful inventory-only check. No ratchet, source
selection, queue suite, nightly/manual control or release-evidence policy changes
in this slice.
