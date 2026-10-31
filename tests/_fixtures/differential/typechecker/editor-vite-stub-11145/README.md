# Editor Vite stub ownership (#11145)

`project.json` retains the reported package, fluent Vite config, SFC, and manually
created Canon declaration stub. The report did not include a minimal project that
causes the editor to create a whole-install link; no such reproduction is claimed.

`installation.json` adds an explicit former editor-mirror whole-install link to a
controlled project installation. The Rust runtime materialization regression uses
its complete files, sentinel bytes, and package link and compares the full
external directory/file/link vector before and after two materializations. This
proves the ownership boundary without claiming native diagnostic parity.

The Node loader test consumes the exact reported declaration stub and runs real
project Vite+, a typed helper config, the embedded native script, and direct
module/runtime-authority controls. The CLI and source/packed API regressions use
small config adapters to make their complete public output observable. Published
consumer acceptance of the exact reported fluent config remains a release gate.
