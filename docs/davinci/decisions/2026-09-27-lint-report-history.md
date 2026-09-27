# Selected historical linter reports through the actual public API

Issue: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881).
Capture parent: `d7a9b74bf90b77cc93cd7096105fcf1a2e76b56c`.

## Decision

The historical JSON line/column repair
`3a87d25f029f731be72705a19787743a996640eb` has an existing current test
that asserts only a line, column and documentation path. The earlier prefix
change `08eb760bae05426251031bd9b6523edae23e7b2f` adds `[vize:RULE]`
to rendered and JSON messages while retaining the raw diagnostic message.
Implementation presence and partial assertions are not complete executable
coverage. Preserve existing assertions and add a separate complete oracle.

Four strict JSON cases retain the original historical source, a Unicode /
emoji / CRLF SFC, a three-file result with differently ordered supplied
sources and an omitted source, and a clean control. Six authored file inputs
execute real `Linter::with_preset(Incremental).with_enabled_rules(...)`,
English locale and full help through public `lint_sfc`. All selected rules
have no actual offered fix; assert that fact and query unchanged bytes again.

The complete unfiltered Debug oracle retains every input/configuration and
ordered `LintResult` field, the public `formatted_message()` output, and the
exact public `format_results` JSON and Text bytes, including ANSI escapes.
Repeat the entire observation and require identical bytes before matching
its golden. No result, span, message, line or column is reconstructed.

The original input actually reports JSON line 6 / column 8 / end column 29;
the Unicode/CRLF case reports line 2 / character columns 30..46 at authored
byte span 47..63. Supplied sources resolve by filename rather than list
position. The omitted source actually uses JSON's existing line-1 byte
fallback; the Text formatter omits that diagnostic when rendering fails.
Unicode graphical locations also differ from JSON character columns. These
existing observations remain explicit rather than being described as newly
fixed behavior. This preparation changes no production output.

## Actual capture and remaining work

The locked offline source build of `-p vize_patina --test report_history
--no-run --message-format=json-render-diagnostics` used the task-owned APFS
clone target with incremental compilation disabled. Rust 1.98.0 /
aarch64-apple-darwin completed the initial build in 43.58 seconds; checked fixture indexing and
repository formatting refreshed the selected target in 14.40 seconds. The
receipt binds that final actual executable, which
captured four full goldens and then matched them with `INSTA_UPDATE=no`;
no tests or cases were ignored. The
[capture receipt](./2026-09-27-lint-report-history-receipt.json) binds full
source, input, executable, target/profile/features and raw log identities.
The capture source identity remains historical after later replay; fresh
Actions proof must identify its own head and executable.

TODO: register these exact inputs and oracles with shared #6891; retain raw
legacy observation identities; review every independent requirement and
supersession in these compound historical commits; verify source-built
Actions and merge queue runs. The broad inventory pinned at `b4f25fb6511075aa531be80d645bb0db8cc151e0`
has 258 fix-title candidates, alongside non-fix behavior changes and
merge resolutions; it remains separate from this later capture parent.
No whole historical commit is accepted by this selected pack. Native
handling and native comparison counts remain zero; #6881 stays open.

The published replay refreshes only the existing linter surface witness
shard: the official generator adds one `test/dev` L0 row for
`report_history.rs` line 11. Its complete 19-artifact check passes; no
aggregate report is recommitted. Latest-head Actions proof remains pending.
