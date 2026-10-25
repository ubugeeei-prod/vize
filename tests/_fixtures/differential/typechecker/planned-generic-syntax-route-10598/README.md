# Planned generic syntax routing (#10598)

The sixteen authored projects retain actual input bytes and complete diagnostic
vectors from run 40181898864 at source
957827706dd439d9810a24f65b4a16c45f266b5e. LF, CRLF and Unicode variants each
keep an independent TS2322 body error while a malformed generic is introduced,
repaired and then the body is repaired. The broken state retains TS1109 before
TS2322; the last state has no diagnostics. No prior pack or expectation changes.

Each `reference.ts.txt` is the independently authored plain TypeScript control
from the same packet. The original full project-session diagnostic DTOs were
independently compared with every intended vector before this pack was added.
The pinned source revision identifies the observed pre-change source, not an
inferred historical bug-introducing commit.

This batch pack checks every returned production diagnostic in order. Separate
executor controls use a successful fake CLI and a refusing real subprocess
session to prove that known-incomplete program checks never fall back to CLI.
The full native/content-mapper conformance suite retains unchanged whole-output,
status and empty-stderr requirements; these batch vectors do not waive the
foreign provider's independently tracked shutdown failure (#10599).
