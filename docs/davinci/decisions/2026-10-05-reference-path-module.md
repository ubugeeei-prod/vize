# Relative path references to module declarations

[#7832](https://github.com/ubugeeei-prod/vize/issues/7832) follows #6994: a
triple-slash `path="types/builder-env.d.ts"` loses the referenced module and
its side-effect declarations because the dependency walk treats the path as
an import specifier. The ordinary import resolver recognizes relative paths
only when they start with `./` or `../`, and otherwise consults import aliases
and packages.

Resolve path references against the containing file's directory regardless
of their leading spelling. Preserve existing candidate probing, canonical
identity, explicit declaration edges and transitive declaration import
rewriting. Unresolved path references must not consult import aliases or
the package resolver. Ordinary import behavior is unchanged.

The corpus retains all five authored files and the original compiler options
from the issue in `tests/fixtures/typechecker/reference-path-module/`, with
SHA-256 input pins. Seven explicit variants cover the original spelling,
leading `./`, nested `.nuxt/` ownership, aliased side-effect imports and
`skipLibCheck`. Each is checked both clean and with a numeric assignment that
must retain the exact string-to-number diagnostic. A structural negative
control refuses import-alias resolution for a missing reference target.

The configured TSGO oracle sends the exact authored source through the existing
read-only configured diagnostic request, compares every raw diagnostic field
in returned order, and verifies the same diagnosing session's before/after
compiler options and configured project. Production batch and source-built
CLI tests compare their complete diagnostic vectors; all authored source and
configuration bytes must remain unchanged.

Source `61982d926e` exposed an existing native File gap: the exact reported
`export const text: string = source` is refused as `UnsupportedSyntax` at
span 50..71 before checking. A separate test retains that incomplete File
and its projection refusal. The authored TSGO oracle does not claim native
File/projection acceptance and does not close this unfinished native work.
The same source's intentional triple-slash lint warning requires every
reported TS/declaration input to use an unchanged `.txt` carrier; runtime
paths and all source bytes remain exact.

Actions run these native, batch and actual CLI checks with required TSGO in
the existing native-phase workflow. Ordinary Rust checks retain the graph
laws; protected full suites retain the CLI regression. Hosted exact-head
success, protected queue acceptance, actual merge and release remain pending
until their external results exist. The first source `183afea69a` failed the
repository formatter on the original one-line tsconfig; an unchanged `.txt`
carrier preserves the report bytes while obeying formatting policy. The same
first source also rejected unsupported `LowerHex` formatting of the pinned
SHA-256 array; explicit byte-to-hex encoding corrects the witness. Native
reports, full configuration snapshots, exact input bytes and CLI raw process
outputs are retained in the source-bound native-phase artifact. No fixture or native comparison closes
the legacy replacement gate #6879.

Source `5351efbeda` passed the exact configured authored TSGO report and
explicit native File refusal, then an unscoped `scan_project` witness
reported `TS2300 Duplicate identifier 'source'`. That API scans every source
on disk, whereas the reported CLI project selects `env.d.ts` and `src/a.ts`
from its original tsconfig. The batch witness now uses those exact selected
roots and retains its complete transitive declaration graph. The unscoped
observation is preserved separately; configured-root and actual CLI results
remain required. Failure captures retain actual virtual source identities.
The consumer migration inventory must also be regenerated for the newly
observed native test imports; no production native migration is claimed.

Source `e76d852b2e` retains the same `TS2300` with configured roots. The
previous broad-scan hypothesis was insufficient. The recorded declaration
imports already point into the mirror, so complete authored and mirrored
native CLI outputs with `--listFiles`, generated config bytes and actual CLI
failures are now captured independently before assertions. Document-local
authored diagnostics alone do not prove an entire project is duplicate-free.
Structural witnesses also use configured roots; declaration import aliases
must participate in reachability even when their targets are `.d.ts`. This
additional fast-path scope applies only to declaration importers.

Source `9694cdcf93` proved the original complete native CLI succeeds. The
mirrored Batch program instead loads `vite/client` through unused shared Vue
helpers, colliding with the authored `*?raw` default export. Shared helpers
now require actual lowered Vue/JSX sources; plain scripts and declarations
retain their configured ambient scope. The source-built CLI passes original
and leading-dot clean/error controls. Its nested `.nuxt` case drops an
explicitly included reference-only manifest from hidden ambient roots; keep
that manifest as a graph root, including missing targets so TS6053 remains
observable. Hidden generated declarations remain unreported type context.
