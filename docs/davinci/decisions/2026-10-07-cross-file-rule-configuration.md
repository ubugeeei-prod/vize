# Cross-file lint rule configuration

The decision is paired with [#7935](https://github.com/ubugeeei-prod/vize/issues/7935#issuecomment-6033331255).

`vize lint --cross-file` appends project findings after ordinary per-file
linting. Those appenders must use the already-resolved configuration for the
file that owns a finding, including declaration-ordered entry rules. The
fix retains one path-to-policy lookup over the existing resolved groups;
it does not re-resolve entries or add a project analysis stage.

The four existing `ecosystem/vue-router-*` IDs keep their output identity.
Croquis findings accept their exact structured `croquis/cf/<finding>` code
as a config key; `cross-file` configures the project group. The two existing
project IDs `html/cross-component-nesting` and
`vue/cross-file-attrs-fallthrough` use the same policy. The closed Croquis
code inventory and producer constants supply the known-ID validation;
unknown IDs remain errors and the ordinary single-file catalog is unchanged.

These opt-in project checks belong to the existing `suspicious` category,
matching the linter's ecosystem/HTML-conformance category. An `off` rule,
group or category suppresses its finding. Otherwise the individual rule
severity overrides the group, which overrides the category. A missing
setting retains the producer's default. This mirrors the linter's existing
disabled-category policy rather than allowing a rule to revive a disabled
category. Croquis severity is updated on the structured diagnostic before
rendering help, so its badge agrees with its outer severity; message text
is never parsed to select a config key. Default IDs, messages, spans,
labels, suggestions, help and diagnostic ordering remain unchanged.

For example, this suppresses only the unknown-route finding:

```json
{
  "linter": {
    "preset": "ecosystem",
    "rules": { "ecosystem/vue-router-unknown-route": "off" }
  }
}
```

`"croquis/cf/unmatched-inject": "warn"` changes that Croquis finding to a
warning. `"cross-file": "off"` suppresses the appended project findings.
`"categories": { "suspicious": "off" }` applies the same category policy;
other enabled single-file categories continue normally.

The legacy fixture retains the complete reported #7932 source blocks,
original off config and argv. Unspecified page bodies are labeled authored
carriers. A separate imported application root with unconditional
`createApp(App).use(router)` owns all four real route findings, independent
of the throwaway-router reachability correction. Complete JSON/plain
contracts also cover inject, composed HTML and attrs-fallthrough findings,
entry precedence and ignore/unmatched controls, warning limits and quiet
output. Inputs are copied from text carriers to genuine project paths and
remain byte-exact after each invocation. Expected reports are authored
from the existing producers, never recaptured or filtered from runtime.

Fresh exact-head Actions must qualify the complete corpus and current
source checks before independent squash admission. The protected full
suites, unchanged instruction ceilings, actual signed merge and installed
release proof remain separate obligations. No legacy replacement or native
default-readiness claim follows from this configuration fix.

## Checked policy authority

[The bounded source correction](https://github.com/ubugeeei-prod/vize/issues/7935#issuecomment-6033453880) replaces a new unchecked
index with complete input-coverage and checked index validation. An invalid
resolved policy fails explicitly with CLI status 2, rather than dropping a
configured file or substituting the default. The production resolver
constructs one valid index per input; all expected inputs and reports stay
unchanged. Fresh successor Actions must qualify this source independently.

The authentic original compiler job also rejected a new unchecked owning
file access in the composed HTML appender. Its existing checked
file/template/result guard now supplies that same path, preserving the
diagnostic and settings without a panic path. All whole corpus vectors
remain unchanged; the authentic failed compiler raw is retained separately.
