# Owned-worker fault capture is deleted, not ported

This [#8840](https://github.com/ubugeeei-prod/vize/issues/8840) record follows
the [Python retirement ledger](./2026-10-13-8840-retired-tooling-languages.md).

The manual `owned-worker-fault` and `owned-worker-controls` walltime
components ([record](./2026-10-12-6830-owned-worker-fault-capture.md)) were
added to capture one historical Musea test-worker SIGSEGV. Only the authored
lifecycle controls ever ran; the original SDK capture was never authorized,
and the fault has no open issue.

The capture cannot become TypeScript without a redesign that weakens what its
record promises:

- `owned_worker_gdb.py` runs inside GDB, which embeds only Python and Guile.
- Worker stop and resume rely on owned pidfds (`pidfd_open`,
  `pidfd_send_signal`), and the root supervisor relies on
  `PR_SET_PDEATHSIG`. Node exposes neither.

So the reusable workflow, its two walltime choices and source input, the
workflow test, the probe and the ten Python helpers and laws are deleted.
The ordinary Linux native-fault observer of the JS package tests
([record](./2026-10-12-6830-linux-native-fault-evidence.md)) stays. If the
fault recurs, a new capture is designed in TypeScript from that observer.
The deleted files stay readable in git history at the parent of this change.

Python files: 29 → 19.
