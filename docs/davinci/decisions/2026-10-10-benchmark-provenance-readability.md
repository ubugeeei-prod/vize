# Benchmark provenance readability (#8364)

The published Blacksmith benchmark previously joined every tool and full
SHA-256 digest into a single paragraph. That made reproduction metadata
hard to scan and pushed the measured results farther down the page.

## Decision

Render one labeled row per tool with its recorded version and complete
binary SHA-256. Keep the metadata in a native `details` element after the
result table and comparison explanation. The English report, README and
all five locales use the same renderer; each locale has a translated
summary and column headings. Runtime-only versions and unavailable values
remain explicit as `n/a`. Recorded values are HTML-escaped so tool output
cannot inject markup or split the Markdown table into extra cells.

Keep one complete provenance disclosure per page. Later type-check and
Vite sections already link to the complete snapshot and do not repeat the
same metadata table. This preserves the existing 350-line source limit
without hiding facts or increasing any source-file ceiling.

Regenerate only the published presentation from the existing committed
JSON artifact. Do not rerun benchmarks, adjust ratios, remove rejected
comparators, shorten checksums or change diagnostic observations. The
measured commit, runner, commands, raw samples and backend readiness remain
available. The theme's scoped `.benchmark-provenance` styles are handled
by the navigation/readability change so long values can wrap on mobile.

## Verification and remaining acceptance

The provenance tests cover the full tool/version/checksum associations,
missing values, unsafe recorded text and unready-backend refusal. The
publication tests check all localized rows, results-first ordering,
unchanged diagnostic evidence and reproducible generated outputs.

Exact-head Actions, the rendered desktop/mobile disclosure and full-copy
browser checks, actual protected merge and deployed-page proof remain
required before #8364 is closed. Broad documentation issues are not closed
by this bounded metadata fix. Queue admission remains with the release
coordinator during the current release pin.

Queue projection against `0200d7a50ef41ad3079d9630bcd640052a139968` rejected only the shared canonical row330; the benchmark content merged cleanly. Move only this change's canonical clause into the existing T2 nightly benchmark cell, preserving every inherited clause, measurement, budget and the 350-line ceiling. Dequeue the conflicting entry and require a clean projection plus fresh exact-head Actions, Docs and browser proof before re-admission.

The preceding `957e1e57` source passed [Check](https://github.com/ubugeeei-prod/vize/actions/runs/38026729962), [Docs](https://github.com/ubugeeei-prod/vize/actions/runs/38026731208) and all 20 source-bound browser cases across five locales, including complete hash clipboard copying and native keyboard disclosure controls. The canonical relocation requires fresh qualification on its new source.

A later clean replacement queue prefix changes the T1 table cell, so Markdown column padding conflicts with this change's T2 cell even though the benchmark content is independent. Move only the owned clause into the existing performance/benchmark paragraph and restore the original table bytes. The new source requires fresh exact-head Actions, Docs, artifact browser proof and a clean current-prefix projection before re-admission; previous acceptance does not transfer.
