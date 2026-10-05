# Standard document links for authored imports

Issue: [#8016](https://github.com/ubugeeei-prod/vize/issues/8016).

## Decision and compatibility

Standard `textDocument/documentLink` uses the existing definition resolver's
relative and project-alias branches. A directory import resolves to its source
index file, and the original `@/*` tsconfig alias produces a link. The shared
relative file probe retains its existing candidate order and additionally
recognizes TSX, JSX, Vue and modern JavaScript index files, including the previous
document-link JSX file contract. The existing alias probe,
Nuxt/project aliases and package-definition selection keep their order.

Bare-package links remain outside the standard document-link contract. Explicit
tsconfig aliases may shadow an installed package; they use the authored target.
Definition retains its existing package fallback. A missing relative file keeps
the legacy link for a file the user may create. A known directory without a
source index and an unresolved alias do not produce directory links.

File targets retain `Url`'s scheme, authority, drive spelling and Unicode/percent
encoding. Only literal path brackets become `%5B` and `%5D`; already encoded
brackets and literal percent filenames must remain stable. This change adds no
native IPC, child process, pipeline stage or public Rust API. Existing descriptor,
CSS and Art links still use their existing paths. Resident SFC descriptor reuse
is unchanged. No native provider or Davinci/default-path replacement is claimed.

## Original corpus and validation

The new `document-link-module-resolution-original` corpus retains the complete
reported App.vue, directory layout and tsconfig alias. Its described Detail.vue
is instantiated with a `name` prop. The three ordered quoted ranges and complete
file URIs are pinned; the existing inactive-imports three-link case and every
previous fix-history request remain intact.

Two production stdio laws run eight original/control sessions across LF and
CRLF. Each session checks the whole versioned empty diagnostic publication, two
whole ordered document-link replies and successful shutdown with stdin open.
Controls include an astral prefix and UTF-16 spans, TS/TSX/JS/JSX/Vue/declaration and
modern JS directory indices, dotted basenames, aliases and alias-over-package
priority, encoded brackets/spaces/Unicode/literal percent, missing-relative
compatibility, empty-directory and unresolved/conditional/bare-package refusal,
and ordinary JavaScript export links. Every existing positive link must decode
to the exact authored physical file and its whole bytes. Windows-drive and UNC
URI encoding are checked independently; Linux source Actions do not establish
Windows runtime execution.

Status at preparation: private source and pinned contracts exist. Source Actions,
full protected queue suites and unchanged 100+4 instruction ceilings have not
yet run for this source. They must pass on their own current heads. Completion
requires the actual merge and original corpus/accepted source identity in a
subsequent supported release; public installed RPC proof remains separate.

## TODO

- Finish independent source review and exact-head Actions, then supervise the
  protected candidate through its actual merge without transferring stale gates.
- Retain the full original contract for tagged/public CLI stdio verification.
- Preserve incoming canonical decisions and independent hover/symbol manifest
  additions on a genuine fresh-main replay.
