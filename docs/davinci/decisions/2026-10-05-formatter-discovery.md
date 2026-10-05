# Limit formatter discovery to project source

Issue: [#7869](https://github.com/ubugeeei-prod/vize/issues/7869).

The original reproduction has unformatted TypeScript in `src/main.ts`,
`generated/out.ts` and `node_modules/dep/index.ts`, with a `.gitignore` that
excludes `generated/`. Before Git initialization the default walk selects all
three, and initializing Git still leaves the installed dependency selected.

Prune every `node_modules` path component before traversal, and reject these
paths at the shared file admission boundary as well. This includes nested
installations, explicitly named dependency files and Vize's generated
workspace under `node_modules/.vize`. Keep Git metadata excluded. Enable local
`.gitignore` rules without requiring a Git repository through the existing
`ignore::WalkBuilder::require_git(false)` API.

Preserve existing explicit input authority: absolute globs and parent-relative
globs retain their existing ignore policy, and directly named authored files
may override `.gitignore`. Installed dependencies remain excluded for every
form. An explicit pattern with no eligible files retains its existing exit 1;
default discovery with no eligible files retains exit 0. Hidden project source
and literal bracketed paths retain their current behavior.

The eight-case legacy corpus preserves the original reproduction and adds a
nested installation and hidden-source control. Existing source-built tooling
jobs exercise check, three writes and recheck both before and after real Git
initialization, with relative/directory/absolute patterns and literal inputs.
Every whole project file map, changed path vector and raw process stream is
retained under `target/differential/`. Files outside the selected set must stay
byte-identical. Rust additionally checks root/nested/generated dependencies.

Hosted source checks, protected full suites and unchanged instruction ceilings
must pass before actual merge and release. This legacy CLI correction makes no
native formatter or measured performance claim.
