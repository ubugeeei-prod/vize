# P3-6 — S4 emission allocation recovery (2026-09-22)

The S4 integration exceeded the native allocation gate. Imports and event
modifier arrays now write directly into sized output buffers; unrecorded
expressions keep the resolved string, and final module assembly reserves its
known length. Escape growth is retained only when source links need rebasing.

The seven existing fixture ceilings are ratcheted down, with no fixture or
measurement-window change:

| Fixture      | Previous ceiling | Native calls | Retained calls |
| ------------ | ---------------: | -----------: | -------------: |
| text_runs    |               79 |           75 |             71 |
| events       |               84 |           74 |             69 |
| expressions  |              112 |          109 |            132 |
| components   |              121 |          100 |            119 |
| templates    |              156 |          134 |            131 |
| spreads      |              114 |          106 |            113 |
| control_flow |              174 |          158 |            161 |

Measured with `cargo test -p vize_atelier_vapor --test davinci_vapor_native_budget
-- --nocapture` on macOS arm64. Linux CI checks the same gate. This is
allocation evidence only; full P3-6 parity and throughput exit remain open.
