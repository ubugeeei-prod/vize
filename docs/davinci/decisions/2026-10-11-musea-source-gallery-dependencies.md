# Source-gallery cold dependency startup

Issue: [#8562](https://github.com/ubugeeei-prod/vize/issues/8562).

The original native inline-Art browser law in Actions
[38103179312](https://github.com/ubugeeei-prod/vize/actions/runs/38103179312)
retains HTTP 504 `Outdated Optimize Dep` responses for the existing Highlight.js
CSS and Bash imports. Vite discovers the middleware-served source gallery
outside the application's initial HTML scan and replaces its optimizer
generation while that gallery is loading. This also blocks configuration
parent #8398; it is independent of the supplied-slot child-process failures.

Include the real source-gallery HTML entry in the initial client dependency
scan. This discovers its complete graph under the bare import IDs used by
browser requests, including the editor's dynamic import. Absolute source-file
includes were insufficient: Actions
[38106426042](https://github.com/ubugeeei-prod/vize/actions/runs/38106426042)
proved that bare Highlight.js/Marked and Monaco imports still caused reloads.
Preserve existing app entry patterns or explicit build inputs; otherwise retain
Vite's HTML inference and test/coverage exclusions. Existing includes, package
exclusions, user aliases and explicit optimizer disablement remain in force.
Build commands and the bundled gallery keep their existing paths. The Monaco
editor remains lazy at runtime; no test navigation, retry or deadline changes.

The regression uses a genuinely empty cold app: the original optimizer has no
dependencies. The corrected real dev server serves its optimized Vue import and
contains every highlighting module, Marked and Monaco under their browser
import IDs in that first generation, with no newly discovered dependencies.
It retains the explicit user Vue runtime, unrelated application HTML discovery,
default test/coverage exclusions, explicit entry patterns and aliased build
inputs (including literal glob characters). Separate controls retain optimizer
disablement, dependency exclusions, existing includes and build behavior.
Local optimizer/source laws and lint are preparation only. Fresh source and
unchanged native Self/global/document-identity browser Actions, protected actual
merge, and any included public-release acceptance remain required. No failed
historical run, installed campaign or issue-closure credit transfers.
