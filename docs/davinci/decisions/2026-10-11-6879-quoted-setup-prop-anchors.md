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
the original MIT license and their source/provider hashes. The test requires
authored parse success, original projection parse failure, and fresh complete
projection parse success. It also checks quoted reserved/colon/hyphen keys
alongside valid Unicode and dollar setup names.

Actual source-built Actions, native Stack delivery and full original consumer
qualification remain pending. The independent residual JSX and authored
Hoppscotch syntax-abort paths require separate dependent fixes; this slice
does not resolve or waive those failures.
