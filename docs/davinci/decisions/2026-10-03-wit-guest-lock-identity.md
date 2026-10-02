# Standalone WIT guest lock identity

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Actual main `b2d0e510c2541c1252f628f515e7a9546c311541` records workspace
`vize_guest` version `0.430.0`, but the expression, output and typed-expression
standalone guest locks still record the path package as `0.429.2`.
[Davinci Contracts run 37058009569](https://github.com/ubugeeei-prod/vize/actions/runs/37058009569/job/111007372851)
fails all three expression-world cases before executing their guest because
the existing build command correctly uses `--locked`.

The actual pinned Cargo 1.98.0 resolver updates only `vize_guest` in each of
these three standalone locks with `cargo update --offline --manifest-path
<guest>/Cargo.toml -p vize_guest --precise 0.430.0`. Every other lock byte,
registry version, checksum and dependency edge is unchanged. The same command
with `--locked` accepts all three corrected graphs. Restoring the authentic
stale expression lock reproduces exit 101 under the same locked resolver.
The resolver uses isolated
temporary state and existing registry metadata; it does not build a guest or
run the host tests.

Guest manifests, released WIT 0.1.2 fixtures, SDK bytes, goldens, compiler code,
workflow commands and instruction ceilings are unchanged. Fresh exact-head
Davinci Contracts must execute the real guests in both hosting modes, including
typed expressions, legacy SDK, packed SDK, resource limits and contract policy.
Required/full checks, fuzz, all 100 instruction probes and protected queue
acceptance remain separate requirements; no execution or merge credit follows
from the lock resolver proof alone.
