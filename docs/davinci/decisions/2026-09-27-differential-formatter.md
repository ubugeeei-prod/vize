# Shared differential fixtures: first executable formatter path

Tracked in [#6891](https://github.com/ubugeeei-prod/vize/issues/6891), with
formatter history provenance from [#6882](https://github.com/ubugeeei-prod/vize/issues/6882).

- Start with two repository-authored formatter regressions: split-opening-tag
  `v-pre` indentation and plain-style nested comments. Preserve complete
  existing literal output bytes, including LF and final newline. Release candidate
  run 36267966910 / artifact 10915050776 from exact `f59e69c38`
  matches both unchanged references on passes 1/2/3; raw capture retains its
  candidate identity and publication-not-confirmed state. This observation
  is separate from the later source-built branch CI gate.
- The common helper initially registers formatter only. Its real legacy adapter
  runs the explicit source-built CLI on isolated `App.vue`, validates immutable
  input/config/output hashes, compares raw bytes and checks passes two/three.
  A source-build receipt binds the existing CI build's exact HEAD, executable
  SHA and recipe; stale/missing/failed binaries fail every planned row.
- Record per-pass input/output linkage and raw process failures. Reports retain
  all planned case identities. No sorting, trimming, CRLF normalization,
  automatic blessing or implicit skipped pass is permitted.
- Native formatter is unavailable: unsupported 2, paired comparisons 0,
  nativeHandled=0 and nativeEquivalent=0. Legacy regression execution grants no
  native acceptance. Other product adapters, complete T1/T2 execution and
  acceptance integration remain TODO; neither issue is closed by this slice.
- Authored `.vue` inputs deliberately join the existing broad fixture corpus.
  Raw expected outputs use `.expected.txt`, preserving bytes and Vue metadata,
  so they do not become accidental second inputs. Include L2/L3 membership
  Folios and generated remarks backlog in the same fixture PR, preserving old
  rows, and measure Canon's companion diagnostic count. `_projects` resident,
  TS-42 explicit roots and canonical real-project registration are unchanged.
- Candidate CLI capture is measured; branch CI, native execution, and existing
  corpus baseline generation remain pending. The downloaded Release artifact
  receives its real run/artifact receipt, never a fabricated CI build stamp.
  Required corpus measurements follow the verified release boundary.
