# Source-gallery cold dependency startup

Issue: [#8562](https://github.com/ubugeeei-prod/vize/issues/8562).

The original native inline-Art browser law in Actions
[38103179312](https://github.com/ubugeeei-prod/vize/actions/runs/38103179312)
retains HTTP 504 `Outdated Optimize Dep` responses for the existing Highlight.js
CSS and Bash imports. Vite discovers the middleware-served source gallery
outside the application's initial HTML scan and replaces its optimizer
generation while that gallery is loading. This also blocks configuration
parent #8398; it is independent of the supplied-slot child-process failures.

Before server startup, register the source gallery's existing eager external
imports in the resolved client optimizer. Resolve each from the actual gallery
importer with Vite's existing resolver, preserving the user's Vue alias and
package-local module identity. Preserve application entry inference, existing
includes and explicit dependency exclusions. Explicit optimizer disablement,
build commands and the bundled gallery retain their existing paths. The lazy
Monaco editor remains lazy; no test navigation, retry or deadline changes.

The regression uses a genuinely empty cold app: the original Vite optimizer
has no dependencies, while the corrected initial optimizer contains every
actual highlighting module and the explicit user Vue runtime. Separate
controls retain disablement, package exclusions, existing includes and builds.
Local optimizer/source laws and lint are preparation only. Fresh source and
unchanged native Self/global/document-identity browser Actions, protected actual
merge, and any included public-release acceptance remain required. No failed
historical run, installed campaign or issue-closure credit transfers.
