# Original HTML profile producer controls

These 45 cases and whole expected sets were authored before executing the new
private Rust producer. Each case runs once per pinned profile (90 intended
observations). They prove selected originals, request provenance, declared
configuration/ignore custody and bounded refusals. They do not qualify wrapper
execution, stock HTML diagnostics, formats, exits, installed use or n8n adoption.
The older literal Vue and `Standalone.html` transport corpora remain unchanged.

Every positive uses genuine `.html` / `.htm` bytes and an owned `git init -q`
repository. Git commands occur only in test setup: journaled `add -f` and
`ls-files --error-unmatch` establish the tracked ignored control; a journaled
local `core.excludesFile` plus `check-ignore` establishes a genuinely configured
ignored control. Selection itself uses `ignore = 0.4.33`, never Git processes.
No process-wide home/configuration mutation is involved. An empty `.git` marker
is source-supported metadata but grants no genuine repository-test credit here.

`cases.json` freezes ordered CLI strings, raw root JSON and ignore bytes, source
ownership, full selected paths, root decision, authority paths including absent
sources, and refusal kind/path/readable malformed authority bytes. `@scripted`,
`@scriptless` and `@clean` refer to these independently authored original files:

| Original | SHA-256 |
| --- | --- |
| `scripted.html` | `b1872d0153db2e4e6a1732ce40662e0d953a0cd9dc7bee579db92f0e2955bd70` |
| `scriptless.htm` | `572cd2e726a62ea30f9e3ff7e7993e1344ad89928df8311ade2036b613477908` |
| `clean.html` | `b2a94c6a15966c3e6c52e93bb20e161b1dfea69efd7f75125e314bd3da310b06` |

The source profiles are Oxlint 1.78.0
(`c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd`) and 1.86.0
(`2ae2939bb2fd98796393658b21556b2a2467e047`). Their explicit-file VCS root
behavior differs; directory roots and discovered originals still respect VCS in
both, including with `--no-ignore`. Actual custom-ignore and CLI prefiltering,
configured traversal and root JSON matching retain their provider ordering.
Sorted paths are the private producer's deterministic contract and do not assert
equality with Oxlint's parallel diagnostic ordering.

The finite input is one existing literal target within an absolute POSIX UTF-8
cwd and one regular `.git` directory ancestor, explicit regular root JSON in cwd,
ordered CLI patterns and a single custom-ignore filename. It refuses globs,
`..` targets/cwd, outside-cwd targets, linked/Jujutsu/nested metadata, symlinks and
special files, inherited/override/nested configuration, suppression state and
malformed authority. Descendant refusal occurs inside the same configured walk,
without a recursive admission prewalk. Arbitrary CLI modes/config discovery,
external custom sources, concurrent filesystem mutation and performance remain
unqualified. Profile refusals are not stock CLI error-packet parity: hosts discard
some ignore parse/traversal errors.

Hosted tests write receipts under
`target/nextest/${NEXTEST_PROFILE}/oxlint-html-profile`. Receipts persist raw
setup command results before assertions, whole before-tree metadata/file bytes,
the complete producer result/refusal before any postread, and whole after custody
before result assertions. Every owned directory, regular file and symlink is
recorded, including all `.git` metadata; special files are classified without
reading them. The source-built archive receipt is retained separately from the
pinned profile identity. Missing hosted source receipts or an absent/nonterminal
Actions result grant no qualification.

Source authority: OXC's MIT-licensed `apps/oxlint/src/lint.rs`,
`apps/oxlint/src/walk.rs`, `crates/oxc_config/src/walk.rs` and
`crates/oxc_linter/src/config/ignore_matcher.rs` at both pinned commits. The
producer calls the actual locked `WalkBuilder`, `IncrementalIgnore`,
`OverrideBuilder` and `GitignoreBuilder`; no private provider API or approximate
glob implementation is substituted.
