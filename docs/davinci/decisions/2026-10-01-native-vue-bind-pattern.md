# Native Vue binding pattern consumer (#6836, #6838)

The actual native Component producer now consumes explicit static-argument
`v-bind:foo="expression"` and `:foo="expression"` through a const Vue 3 pattern
table of plain function pointers. The table selects real native L1 directive
decompositions and lowers them during the element/component's construction.
It does not use legacy capabilities, Croquis facts or a second attribute parser.

The producer collects directive observations in the same attribute walk as
static attributes. Inside the checked owner's callback it applies those
patterns before child construction, producing attached `ui.bind` nodes in
authored directive order. Exact native head ranges must show the immediate
colon/static argument and no modifiers; recovered dynamic/tail syntax cannot
claim a complete static binding. Static names are borrowed from checked authored
ranges. Native L1 decodes the value once, `parse_once` parses it once, and the
consuming Expr handoff transfers that same root and wrapper/decode coordinates
to the checked L2 factory. Interpolations share this retained-expression path.

Every admitted or rejected parsed value retains the complete original L1
comments, diagnostics and source observation keyed by its optional binding id.
Syntax/coordinate/token-budget rejection creates no fallback node and consumes
no id. Supported later bindings and child fragments survive; failed provenance
remains beside their actual canonical owner. Successful sealing remains direct,
with no subsequent tree validation/counting walk or serialization.

Modifiers, property shorthand, same-name values, dynamic arguments, spread,
event handlers, model/control-flow/slot scopes and custom directives remain
typed unsupported and do not parse their values. In particular, events require
real Expr-versus-HandlerBody grammar selection and safe ownership before they
can be admitted. A `v-pre` carrier remains a typed hole and is observed before
unsupported-tag or missing-owner child fallbacks, preventing retained raw inner
attributes from being admitted as bindings. Known extents span the entire
carrier. Missing extents use only the known opening-name range while retaining
the original complete L1 carrier and missing-close fact. Supported siblings and
non-pre fallback fragments survive. This is explicit incompleteness, not v-pre
implementation.

Nine genuine parser-to-artifact laws cover attached order, same-AST/comments
retention, entity/Unicode coordinates, unsupported forms, full syntax failure
observations, multi-scalar entity interiors, whole pre carriers and token-budget
holes, plus pre guards on unsupported/missing owners and non-pre fallback
fragments. They pass with the existing ten native consumer laws (19/19) using the
actual whole L1 handoff library and genuine selected L2 canonical modules.
Scoped strict production Clippy passes. Whole-crate exact-head Actions and
unchanged allocation/instruction gates remain required, as does terminal
protected merge after the actual prerequisite branches integrate.

No product route changes. All five products, complete Vue/dialect/JSX semantics,
file-language and embed identity integration, remaining native patterns and
both roadmap issues remain unfinished. `is_supported` still describes only
this bounded native preserve-whitespace contract.
