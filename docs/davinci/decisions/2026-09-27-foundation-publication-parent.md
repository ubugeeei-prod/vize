# Foundation publication parent (#6831/#6832)

The publication replay starts at #6919 head
`07ac185bac5ce8b6a99191be10c24a1dac854a78`. Earlier hashes and measured results
in the [first replay](./2026-09-27-foundation-stack-replay.md) remain evidence
about their original prepared heads. This new parent contains CI changes only;
foundation gates and product changes remain separate layers. Original
#6905/#6906/#6907 heads and worktrees remain intact.

Current `0.429.1` workspace dependency and lockfile package versions survive
the physical L3 rename. Its immutable first-name published baseline remains
`vize_impeto@0.429.0`; this work performs no release promotion.

The selected-CI parent's conservative `docs/davinci/plan/` classification is
retained: policy changes request every selected source lane as well as the
unconditional dependency gate. The old docs-only test expectation changes
accordingly. Four comment lines in the Check workflow are removed or condensed
to offset the four declaration-gate lines under the source350 growth ratchet.
Other workflow semantics retain the new parent's exact parsed values.

TODO: record fresh composed source proofs below. The parent owns publication,
exact-head Actions and actual merge-queue proof. A prepared or draft layer is
not a successful CI-first main merge. #6831 retains three legacy exceptions;
#6832 retains its later naming and shared-production-generator work.
