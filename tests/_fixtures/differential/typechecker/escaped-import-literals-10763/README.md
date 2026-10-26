# Quoted module directory preservation

The three original inputs are copied byte-exact from the actual before producer
in Actions `40237094381`, artifact `12615870947`, source
`62652a4d4029fa11016dfa1ac8c24669b42c8d7a`. The directory contains both quote
characters, Japanese text and an astral character; the SFC retains CRLF.
The immutable generated output was invalid TypeScript in both the baseline
and the separate extension-policy prototype. Neither old failure is replaced.

The second case adds one authored assignment after the import. Its complete
TS2322 message and UTF16 location come from an independent TypeScript 5.9.3
program using the exact config and implementation, before candidate execution.
The reference's full diagnostics, extracted script and compiler hash are retained.

Actual production batch execution must accept the original graph and preserve
the negative diagnostic. The separate mandatory source import job asserts exact
decoded literal identity, parseability, every changed interior and exclusive
boundary in both directions, Unicode/CRLF, all module source kinds and existing
syntax-cache counts. The batch corpus does not confer navigation or LSP credit.

Safe module paths retain their existing delimiter without a temporary encoder
buffer. Unsafe paths use the existing TypeScript string encoder. No resolver,
filesystem pass or syntax parse is introduced by the encoding boundary.
