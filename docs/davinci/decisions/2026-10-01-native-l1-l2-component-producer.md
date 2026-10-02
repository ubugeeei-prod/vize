# Native Component to canonical L2 (#6836, #6838)

`lower_component_native` is an actual consumer of the ordinary native L1
Component parser and the retained-expression handoff. It constructs canonical
L2 nodes through checked factories during its source-tree walk, then directly
seals their preorder accounting. It never invokes the transitional lowering,
the legacy parser, `Artifact::try_new`, a second expression parser or a dump.

This first contract preserves whitespace and takes an explicit JS/TS language.
It admits ordinary elements/components, static attributes, text, comments and
mustaches. The L1 entity providers retain text/attribute context differences and
complete multi-scalar references. Mustaches call L1 `parse_once`, safely consume
the existing root, and transfer checked wrapper/decode coordinates to L2. The
handed AST pointer and original comments/full parser diagnostics remain retained.
`retain_expression_in` exposes that same real checked L1-to-L2 handoff for other
native expression consumers without another decoding or parse.

The integrated producer consumes the actual fallible `ComponentParse` carrier
and retains it intact as `NativeLowered::component`: tree, optional authored
projection, original surface errors and typed directive admission failures.
Each admission failure also creates a source hole, so a nonempty unsupported
list cannot become a false `is_supported` result. Surface errors are not copied
into another collection or discarded when fragments are admitted.

The subsequent [native Vue bind pattern](./2026-10-01-native-vue-bind-pattern.md)
adds explicit static-argument bindings using the same once-AST handoff and
checked attached factories, while retaining all other forms as holes.

Other unsupported directives, Vue-special carriers and recovered unavailable owner
extents remain typed source holes, ordered failed provenance and advisories.
Admitted descendants survive missing carriers as canonical partial fragments;
their owner extent is neither guessed nor obtained through a second subtree
walk. Unadmitted mustaches retain the complete L1 observations without publishing
a recovery AST or invoking an L2 fallback parser. `is_supported` means only that
this bounded contract has no holes; it is not whole Vue or product acceptance.

The shared identity-expression trivia guard still refuses line-comment-only
leading/trailing text outside a retained root. The original L1 comment facts stay
available. Admitting those positions safely requires the native writer's actual
separator contract; the legacy admission rule is not weakened.

Ten real parser-to-artifact laws cover exact retained pointers, comments and
syntax diagnostics, entity/Unicode coordinates, malformed partial fragments,
unsupported directives, namespace/component numbering, token-budget holes and
original component error/admission retention. They pass using the actual whole
L1 carrier/handoff source library and genuine selected L2
modules with pinned cached OXC/L0 dependencies; scoped production Clippy and the
unchanged storage rules pass. Full workspace/Actions, protected queue checks
and newly identified native instruction windows remain required. Existing
instruction identities and ceilings stay unchanged; their workloads do not
establish this new producer's performance.

The fresh source replay preserves signed actual main `d596d815` and its
completed provider/host changes. Whole current L1 compiles; the 34 selected L2
and ten native laws, whole-L2 and native production Clippy, ordinary module
discovery, the actual Rust assertion linter with its five self-tests, six
storage laws and complete canonical inventories/ledgers pass. No allowlist or
source-budget exception is added. Exact-head Actions remain required.

Per-file language resolution, embed identities, v-pre, other Vue dialects,
directive/control-flow/scope patterns, full namespace recovery and whitespace
normalization, JSX/TSX and every product route remain unfinished. No product is
switched and both roadmap issues remain open.
