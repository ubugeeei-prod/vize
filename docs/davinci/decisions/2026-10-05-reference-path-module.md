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

The original Program test borrows its genuine completed File, asserts the
same projection owner and SHA-256 source, compares every raw native
diagnostic field in returned order, and verifies the same diagnosing
session's before/after compiler options and configured project. Production
batch and source-built CLI tests compare their complete diagnostic vectors;
all authored source and configuration bytes must remain unchanged.

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
