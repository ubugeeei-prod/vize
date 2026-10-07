# Globally registered component lint options

Issue: [#7978](https://github.com/ubugeeei-prod/vize/issues/7978).

Expose the registration rule's existing global-name list through
`linter.ruleOptions["vue/require-component-registration"].globals`. The shared
Rust config, CLI and LSP constructors and Pkl/JSON Schema/TypeScript artifacts
carry the same string list. Name matching accepts Vue PascalCase and kebab-case
spellings while retaining the existing case-insensitive allowances. Regular
expressions are not interpreted.

Options do not enable an unselected rule. A later config layer replaces the
list, including an explicit empty reset. Runtime `previewSetup` code is not
executed or analyzed by the linter; the names explicitly record the components
it provides. `GlobalComponents` provider resolution remains a separate future
precision path.

The authored Art corpus retains `defineArt` implicit registration, both global
spellings and a missing-component positive. Tests cover the public lint API,
CLI configuration, editor configuration and strict layered options. Art editor
lint delivery itself is tracked in #7945. This legacy regression adds no
Davinci native acceptance credit.

The original paired 400-provider workflow explicitly qualifies this slice's
exact config, lint constructor, registration-rule, generated artifact and
test paths. Every original input/source custody check, all 79 response answers,
534 acknowledgements and notification controls remain required. The added
source paths permit the existing full comparison to execute; no broader path
wildcards, narrowed assertions or performance credit are introduced.

A qualified config/lint source change also cleans the changed `vize_l0` and
`vize_patina` packages before both builds. Cargo must attest those dependency
artifacts as newly compiled; a baseline artifact cannot satisfy the after build.
This adds dependency rebuild work and rejects stale artifacts without changing
the shipping build recipe or any runtime comparison assertion.
