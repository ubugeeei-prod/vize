# Published release summaries

Related tracker: [#6239](https://github.com/ubugeeei-prod/vize/issues/6239).

The root changelog's latest dated entry was 0.362.0 even after 0.435.0 and
0.439.0 were published. Add short, grouped summaries for those two actual
releases, linking every selected change to its merged PR and every summary
to the complete public comparison. Keep the existing historical entries and
Unreleased section.

## Source evidence

| Version | Public release date (UTC) | Tagged commit                              | Comparison                                                                              |
| ------- | ------------------------- | ------------------------------------------ | --------------------------------------------------------------------------------------- |
| 0.439.0 | 2026-10-10                | `a26243855bcf81005049252a6e71b6dd55e457f1` | [0.435.0 to 0.439.0](https://github.com/ubugeeei-prod/vize/compare/v0.435.0...v0.439.0) |
| 0.435.0 | 2026-10-06                | `51f3778473a17cecfb31c6238207418c2232c299` | [0.434.0 to 0.435.0](https://github.com/ubugeeei-prod/vize/compare/v0.434.0...v0.435.0) |

The release API confirms publication at `2026-10-10T02:50:40Z` and
`2026-10-06T08:01:31Z`, respectively. Use the generated What's Changed
sections to select changes in each comparison. The cumulative migration
notice at the top of the 0.439.0 release is inherited context and is not
treated as newly implemented behavior in that release.

No published 0.436.0, 0.437.0 or 0.438.0 release appears in the current public
history. Those version metadata candidates do not become changelog release
entries. The full historical backfill remains unfinished; the changelog
explicitly links the complete public release history for omitted details.

The git-cliff configuration also points to the existing Rust helper instead
of a removed JavaScript path. This change does not run the release operator,
alter tags or rewrite published GitHub release bodies. The emergency 0.440
source pin and publication proceed independently.

Validation: compare each linked PR with its public release's generated
change list, check the actual tagged commits, format the edited source
files, and require exact-head Actions before protected delivery. No product
behavior changes or publication claims follow from this documentation.
