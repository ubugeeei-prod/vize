# Focus Davinci on the Vue toolchain

The maintainer retired the experimental MoonBit dialect and component guest
architecture on 2026-10-17. These isolated experiments had no production Rust
consumers outside their own crates. They add no supported Vue workflow.

Removed implementation:

- `vize_dialect_moonbit`, including its L1 coordinate helpers and L2 expression scanner.
- `vize_guest`, `vize_extension_contract` and `vize_extension_host`, including
  their L0 wire records, private npm SDK and Volt example.
- Their standalone contract and MoonBit workflows, unused compiler bootstrap,
  Nix toolchain, release lock updates and CI package registrations.

The Compiler, Linter, TypeChecker, Formatter, LSP, Story, Editor and CI work
comes first. Supported Vue SFC, JSX/TSX and JavaScript integrations, including
VS Code, Zed and other editor extensions, remain in scope. The JavaScript lint
plugin SDK is independent of the retired component ABI and remains supported.
The generic Marquette compatibility classifier is also independent and remains.

Foreign language support is deferred to one low-priority backlog item. This
removal creates no replacement guest runtime, extension host or promised
MoonBit feature. Generic IR capabilities are retained where compiler consumers
still use them; removing an isolated experiment does not justify rewriting the
working compiler around a narrower representation.

Earlier phase-6 plans, measurements and decision records are historical evidence,
not current implementation commitments. Captured release and differential fixture
bytes retain their original identities. Git history preserves the removed code.
