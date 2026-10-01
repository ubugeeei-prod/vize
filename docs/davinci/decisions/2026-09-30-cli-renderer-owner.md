# CLI owns the diagnostic renderer (2026-09-30)

Decision for [#6833](https://github.com/ubugeeei-prod/vize/issues/6833).

Move the terminal diagnostic renderer into `vize::render`, following its
existing CLI consumers. Diagnostic/fact/pass types come directly from L0.
Locale catalogs, ANSI styling, excerpts, witness explanations and fixes retain
their implementation. CLI explain and rich lint use the same moved renderer;
the old substrate has no renderer re-export or CLI dependency.

The move-only commit relocates 81 files, including all 64 exact en/ja/zh and
ANSI fixtures. Integration changes Rust paths and moves `unicode-width` into
the CLI manifest. The fixture-only exemption identifies its actual `vize`
owner; it still prints no proof note. Native storage inventory excludes these
CLI files after their relocation. TS-53's command and catalog/fixture paths,
natural v-on inventory and source witnesses follow their new owners.

Replay `python3 tools/support/levels/move-cli-renderer.py moves`, commit the
file moves alone, then run the script with `integrate`, format Rust, normalize
Cargo metadata and regenerate inventories. All existing byte fixtures are
preserved; this ownership change introduces no legacy behavior fix or product
route switch. Validate TS-53's exact bytes and CLI explain/lint in Actions.

The local catalog and storage checks passed 17 tests, and all seven renderer laws passed against the 64 untouched byte fixtures. Actual merge remains
pending behind the foundation/import ownership slices. #6833 still requires
Croquis/feed/repro pages, legacy test plans, remaining imports and final
compatibility-package deletion.
