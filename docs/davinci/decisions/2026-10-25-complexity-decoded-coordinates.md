# Retained expression coordinates for complexity diagnostics

Issues: [#10465](https://github.com/ubugeeei-prod/vize/issues/10465),
[#10473](https://github.com/ubugeeei-prod/vize/issues/10473).

The neutral complexity scorer projects retained AST ranges through `JsExpr`'s
checked wrapper and decoded-to-authored bridge. A contribution that cuts an
indivisible decoded segment refuses the whole expression's scored rows: one
explicit unknown row retains the original range and earlier facts remain intact.

The compatibility Vue lowering previously decoded references and discarded that
bridge. Its source preparation now retains the existing L1 correspondence, then
uses the same coordinate transfer as the native producer. The complete-expression
parser and nesting guard still run once. Reference-free inputs keep their current
parse and allocation path; unknown references remain literal and double-encoded
references decode once. Source custody failures have a distinct
`source-coordinates` opaque classification, separate from parser rejection.

The native L1 owner and compatibility lowering now agree on complete ordered facts
over twelve quote/line-ending/encoded-operator combinations with file-absolute
Unicode ranges. These are separately constructed original owners, not a native
claim from a compatibility facade. Partial multi-scalar entity selection refuses
exactly, and native later holes remain incomplete after fragment analysis.

The production `vue/max-template-complexity` control pins complete diagnostic
packets, including secondary labels and absent fixes, over four original SFC
inputs. Secondary ranges preserve encoded operators and nested conditionals.
Regular public JSON output omits those secondary labels, so its equality alone
does not establish this contract. The published rich renderer reproduced the
incorrect geometry before the correction; hosted product qualification is a
separate gate from the local native/fact laws.

Neither change selects a new default product route, certifies native JSX/Pug
markup consumption, or completes #6848/#6871. Compatibility output, native owner
proofs, hosted qualification and publication remain distinct evidence.
