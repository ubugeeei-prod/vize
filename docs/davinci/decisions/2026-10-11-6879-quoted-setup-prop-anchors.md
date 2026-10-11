# Preserve quoted setup props without invalid reference anchors (#6879)

The original release-445 App E2E at producer
`605429c660b328bb03bec5bbfef15961e59fcaf0` lost the deliberately injected
Element Plus TS2322. The retained native packet from
[run 38115632497](https://github.com/ubugeeei-prod/vize/actions/runs/38115632497)
shows that native TypeScript instead aborted on generated syntax errors.
The complete Popover and Tooltip projections emit `void onUpdate:visible;`
from a quoted prop key. Their authored SFCs are valid TypeScript.

Only setup reference-anchor selection changes: the existing OXC
`is_identifier_name` authority excludes nonlexical property keys before the
unchanged template-use filter and reserved-anchor handling. Original script,
property/type/emits generation, valid Unicode, dollar and underscore binding
names remain available. No selected config, dependency registration, source
member, diagnostic expectation or consumer deadline changes.

The corpus in
`tests/_fixtures/differential/typechecker/quoted-prop-setup-anchors/`
retains both complete pinned SFCs, both complete original-H generated modules,
the original MIT license and their source/provider hashes. Normal Rust tests
pin the entire anchor emission and unchanged prop inventory for a source
control declaring real Unicode, dollar and underscore lexical bindings beside
quoted reserved/colon/hyphen keys. They use OXC's actual identifier and parser
authorities and the production Vue document generator.

Fresh complete Popover/Tooltip projections are qualified in the dependent
native lane against their original pinned import graph after the unchanged
App E2E row hydrates it. A hand-built analysis summary is not a substitute for
that imported type context. Complete generated text and mappings are retained;
the original-H projections remain the actual parse-failure controls.

Actual source-built Actions, native Stack delivery and full original consumer
qualification remain pending. The independent residual JSX and authored
Hoppscotch syntax-abort paths require separate dependent fixes; this slice
does not resolve or waive those failures.
