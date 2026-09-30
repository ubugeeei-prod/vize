# Inspection library owns crash reports (2026-10-01)

Decision for [#6833](https://github.com/ubugeeei-prod/vize/issues/6833).

The crash-report page/parser and failure formatter live in
`vize_curator::repro`. Curator already owns local artifacts describing what
Vize observed or generated, and the CLI already depends on it. This keeps
report decoding in the inspection library without making the isolated fuzz
harness depend on the entire CLI. CLI repro/reduce/ICE and the fuzz harness
import the same moved implementation; the old substrate has no repro export.

A move-only commit relocates the page and its nine exact parser/printer laws.
The embedded last-good artifact remains verbatim and terminal, scalar/config
order remains canonical, failure text is unchanged, and an unattributable pass
remains empty. Captured historical reasons and protocol fields are preserved.
This is artifact ownership, with no new pipeline stage, legacy behavior fix or
product route switch; it does not invent native per-pass capture.

The fuzz targets import shared Dump/pass/side-table contracts directly from L0
and host report decoding from Curator. Their direct compatibility-substrate
dependency drops. No native level depends on the inspection library. Native
storage accounting drops the host report row; source witnesses and local links
follow the moved owner.

Replay `python3 tools/support/levels/move-inspection-repro.py moves`, commit
those two moves alone, then run `integrate`, format Rust and regenerate source
inventories. Preflight checks every move before changing the index. Replaying
integration on the resulting tree changes no additional files.

Local verification passed nine exact page laws and thirty-five
storage/dependency/target checks. CLI repro/reduce/ICE and fuzz/WASM validation
remain required in Actions before merge. #6833 remains open for legacy test
plans, remaining consumers/tests and final compatibility-package deletion.
