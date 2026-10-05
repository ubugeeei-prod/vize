# Ready terminal progress (2026-10-05)

Tracked in [#7847](https://github.com/ubugeeei-prod/vize/issues/7847).

The maintainer requests a modern, rich Vize CLI implemented in Rust. The existing
`ready` command now presents its four existing stages with a title, numbered
labels, completion markers, per-stage elapsed times and a final total on stderr.
The completion marker follows the actual command return. A command's existing
process exit still stops the run before later stages or the final summary.
Configured skips retain their own explanation; completing a stage does not
assert that its disabled checks ran.

Use Fresco's existing capability resolver with **stderr's** terminal observation.
Interactive terminals receive append-only progress without cursor movement,
terminal-mode entry, spinner threads or additional processing stages. Color and
Unicode follow its existing environment conventions, including `NO_COLOR`,
`FORCE_COLOR`, `CLICOLOR`, locale and Fresco overrides. An ASCII profile retains
the same stage labels and completion meaning. The existing redirected, CI and
`TERM=dumb` stage log bytes remain exact, even with forced color. Subcommand
stdout, diagnostic output, exit codes and stage order retain their contracts.

The retained `App.vue` CLI fixture exercises real formatting and build output;
its config explicitly disables lint and type checking to avoid unrelated runtime
dependencies. Real subprocess tests exercise both stderr PTYs and redirected
output, including a missing-input failure before the second stage. Pure profile
laws cover monochrome, ASCII, styled terminal, CI, dumb and forced-color
redirected presentations, and interrupted-stage completion refusal.

First source `3d6961e856` passed all ten new Rust laws on Check `37244855543`,
but the tooling worker correctly rejected fifteen partial integration
assertions. The replacement compares complete stderr transcripts and exact
stdout/source bytes, normalizing only validated numeric elapsed-time fields.
No assertion allowlist or gate exception is added; fresh Actions are required.
Source `aaac7435e8` then exposed a missing existing `Built: App.vue ->
./dist/App.js` line in both success expectations. The complete oracle retains
that original output line; production source and all refusal laws stay exact.

Exact-head Actions and the protected merge queue remain required before actual
merge. Publication is coordinated with the current release; authored source or
passing PR checks alone do not establish publication or native-stage completion.
