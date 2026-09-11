# Story testbed behavior contract

Normative contract for the issue #4898 story harness inventory. The harness is
source-local: it derives coverage from `src/catalog/family-catalog.ts`, keeps pending
Musea/browser/VRT work explicit, and lets future family PRs flip individual
artifacts to ready when the colocated files are added.

| #   | State             | Input                         | Outcome                                                                                                      | Proven by                                                            |
| --- | ----------------- | ----------------------------- | ------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------- |
| S1  | family catalog    | stable public family          | one deterministic story-testbed entry is emitted in canonical order                                          | `publishes a deterministic story-testbed inventory for each family`  |
| S2  | Musea surface     | catalogued family name        | planned story path is `<target-dir>/<family>.art.vue` and remains colocated                                  | `publishes a deterministic story-testbed inventory for each family`  |
| S3  | matrix controls   | story-testbed entry           | states, slots, parts, presets, RTL, reduced-motion, and forced-colors are required                           | `publishes a deterministic story-testbed inventory for each family`  |
| S4  | existing evidence | package source file inventory | supporting behavior tests must exist; planned Musea/VTU/browser/VRT files must stay pending until registered | `audits planned artifacts and supporting tests against source files` |
| S5  | browser suites    | Vitest browser lane           | focus, pointer, and layout suites are emitted for every public family                                        | `publishes a machine-readable harness manifest and run plan`         |
| S6  | Musea panels      | story-testbed harness         | accessibility tree, live-region transcript, event log, bundle explorer, and SSR/hydration lab hooks exist    | `publishes a machine-readable harness manifest and run plan`         |
| S7  | VRT review        | Playwright VRT lane           | screenshot review writes reviewable artifacts under the shared UI VRT artifact directory                     | `publishes a machine-readable harness manifest and run plan`         |
