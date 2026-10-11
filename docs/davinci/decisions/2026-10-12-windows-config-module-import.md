# Windows config module imports at the Vite bundle boundary

Issue: [#3956](https://github.com/ubugeeei-prod/vize/issues/3956), with the existing
config-loader requirement in [#8371](https://github.com/ubugeeei-prod/vize/issues/8371).

The fresh-project source qualification exposed a Windows production failure before
its first matrix cell could finish. This slice fixes only the wrapper import
specifier and retains the selected filesystem path and config identity.

## Observed failure

[Native Smoke run 38154287351](https://github.com/ubugeeei-prod/vize/actions/runs/38154287351)
ran the first JavaScript manager extension at source head
`538069c8953149eddd65759ebf83ee71fe8c79f2`. Its four Windows fresh-project
workers (x64/arm64, Node 22/24) stopped during the original first npm TypeScript
cell. Rust canonicalization supplied a legitimate Windows verbatim path, and
Vite's bundle resolver rejected the wrapper's `\\?\C:` import specifier.
The clean project check exited with status 2; none of those four workers completed
the ten-cell oracle.

The eight Unix fresh-project workers completed the original eight cells and the
new Bun JavaScript cell, then rejected the Vite+ runner's ordinary Vite JavaScript
installer expectation. That separate expectation repair follows the existing
production lockfile detection. These failures remain failures; retaining their
complete job logs does not grant completed qualification credit.

## Boundary correction

The wrapper decodes the already selected `configURL` with Node's `fileURLToPath`
before serializing its import specifier. On Windows this produces a regular drive
or UNC path from the existing file URL; on POSIX it preserves the filesystem
path, including literal backslashes. The Rust canonical path, `configURL`,
`createRequire` identity, config root, runtime callbacks, and cleanup remain
unchanged. There is no additional pipeline stage or public API change.

A direct file-URL import in this Vite wrapper would externalize TypeScript in the
existing Vite 7 bundle control. The wrapper therefore keeps an absolute
filesystem import specifier in the bundle graph. The existing no-Vite path still
imports its file URL directly.

The complete runtime custody is finite: base `a49936db24d9540e0630b42474654190d575294b`,
original SHA-256 `a5d247f627f4a45924f61d72253114349b512c23817487b45c0a068db69f7eda`
(2067 bytes), successor SHA-256
`6dc8d9af88a733dd53d6d38171f8793bfd5857d9b6c77bf506c06ce576c67312` (2478 bytes).
The original Rust loader, Vite runtime, and whole eight-control loader test remain
byte exact. The separate config qualification owner retains its original frozen
loader-context manifest and must record an explicit successor for this runtime;
this change does not rewrite historical manifests or authorities.

## Controls and qualification

The additive path fixture authors eleven complete filesystem paths and complete
JSON import literals: regular and verbatim drives, regular and verbatim UNC,
spaces and Japanese text, percent/hash/quote transport, lowercase drives, ordinary
POSIX paths, POSIX Unicode with literal backslashes, and a POSIX filename that
looks like a Windows verbatim path. Three controls reject encoded separators.
The Windows double-quote case proves transport only because that character is
not a valid Windows filename; the fixture makes no mounted-UNC-share claim.

A real project in a directory containing spaces, Japanese text, an apostrophe,
percent, and hash passes its namespaced config path through the production
loader. It compares the complete value, project directory and original source
URL, and the complete unchanged source file. It runs together with all eight
original native config-loader controls, including Vite 7 TypeScript, Vite 8,
JSON, missing imports, script concatenation and cleanup. Local qualification
passes 23 tests with zero skips.

The Windows fix is the parent of the JavaScript manager coverage PR in a native
GitHub Stack. Both layers require their own exact-head source Actions. One full
composed child Native Smoke campaign runs the unchanged original eight cells
plus the two new JavaScript cells on all twelve Node/host workers, alongside all
six native hosts. Shared production qualification requires byte-equal owned
product sources, loader bodies and build-graph/root-config inputs between parent
and child. The child's exact-head campaign is not attributed to the parent as an
exact-head run, and source-packed success does not prove public installed
acceptance.

TODO: retain the complete eighteen-worker outcome, whole ten-cell success
vectors and physical source hashes; pass protected queue suites; verify actual
merge ancestry; and carry the fix into an official release with installed
acceptance. The broader #3956 checklist stays open until its own remaining
requirements pass.
