# Shared Vue capability owner (2026-10-01)

Issue: [#6841](https://github.com/ubugeeei-prod/vize/issues/6841).

## Decision and scope

L1 owns Vue template syntax policy in `dialect/`. The existing full parser
capability model moves from Armature to `vize_l1::dialect`; its Vue 0.10,
0.11, 1, 2/2.7 and 3 tables live in the `vue0`, `vue1`, `vue2` and `vue3`
modules. The existing public historical names remain compatibility contracts.
Armature's `legacy` module becomes a feature-gated re-export adapter.

The three-boolean `LegacyCaps` legalization projection also moves to L1. It
derives its values from the shared parser capability table, including an
explicit `v2_event_sugar` fact. L1-to-L2 exposes this type through its dialect
module and preserves the existing `lower::LegacyCaps` and root exports. This
removes the second version-dependent capability derivation without making L1
depend on the conversion or on a legacy product crate. The compact lowering
layout, public debug spelling and all existing flag values stay unchanged.

The generic L1 tree, builder, renderer and lexer do not import dialect policy.
The L1 construction/rendering isolation gate rejects policy references in its
generic modules, `markup/profile.rs` and the explicit `markup/lex` scope,
including grouped imports and references behind `cfg`. A mutation law checks
the generic profile, lexer module entry and nested implementations while
retaining dialect hooks elsewhere in markup.
The existing conversion capability bridge remains transitional; removing
inline `caps` branches and isolating all legalization policy is separate work.
This slice introduces no pipeline stage, transport serialization, product-route
switch, fixture-source or expected-output byte change, snapshot change or
numeric instruction-budget change. Four rare-dialect grammar witness paths
follow the unchanged selector prose to L1; the generated inventory keeps all
seven input hashes, parser modes, selectors and input-only coverage states.

## Replay and validation

Moves are isolated in a move-only commit. Replay them with:

```sh
vp node tools/support/levels/move-dialect-capabilities.ts move
```

After applying the wiring commit, `check` validates the moved owners, narrow
compatibility bridges and per-version table modules. Existing seven parser
capability laws and three compact projection laws move with their owner. New
laws pin the six supported lines' legalization behavior and three-boolean
layout, and compile both adapters against the exact shared L1 types.
Three TypeScript replay laws verify exact bytes, idempotence, rejection of
conflicting/missing owners before mutation, and the integrated repository.

Focused local validation passes all eleven L1 capability laws and both
adapter type-identity laws. The per-file consumer and Croquis inventories
remain fresh, and the dependency gate reports no forbidden normal/build edge.

Actions must validate the exact PR head and protected queue candidate. The
existing differential corpus and all instruction ceilings remain acceptance
gates; local relocation or unit validation is not merge proof.

The first PR Check exposed the stale grammar-owner paths. Updating witness
metadata and regenerating the inventory preserves every generated input byte;
the corpus checker continues to validate witnesses and hashes without a new
fixture or expected-output acceptance.

### 2026-10-21 follow-up

The transitional three-boolean projection is removed. `LegacyCaps` is now a
compatibility alias for `LegacyDialectCapabilities`, and its historical
`for_version` and `needs_sugar` methods live on the shared descriptor. Parsing
and legalization therefore carry the same resolved value instead of copying
three fields into a second type. The Vue 3 legalization fast path remains the
same three-field predicate, while newly added dialect facts become immediately
available on both sides of the L1→L2 boundary without a synchronization test.

### 2026-11-07 per-file descriptor follow-up

`vize_l0::FileDialect` is now the single resolved value for the input axes a
Davinci file carries: Vue version, Vue versus petite-vue, standard/strict/
quirks recovery, document versus component markup, and JS versus TS embedded
language. `TemplateSyntaxMode`, `ProfileKind` and `Lang` have the same L0 owner;
Relief and L1 retain source-compatible re-exports of those exact types.

The original Vue SFC descriptor resolves the value after its sole container
walk determines the real script language and retains it beside the original
owner. The original Document owner resolves configured or structurally
detected petite-vue before its sole lexer run. Downstream scoped-file and L4
projection admission read that retained value instead of independently
matching version and framework selectors. Existing `DescriptorOptions` fields
remain a compatibility input surface; they are collapsed once and are not a
second retained descriptor.

Quirks is therefore an explicit descriptor feature, not a parser-only bool.
The currently bounded native SFC admissions continue to reject nonstandard
recovery with their existing `UnsupportedOptions` result, so this ownership
change does not broaden syntax admission or alter emitted bytes. The legacy
compiler consumes the same moved enum through its old public path.

### 2026-11-07 per-level registry follow-up

Each level now exposes one dialect registry. L0 is the sole constructor and
predicate owner for the resolved input axes. L1 maps configured Vue versions to
their historical line and capability table; each version table is a const
composition of named syntax features instead of a second field-by-field
derivation. A frozen six-version matrix pins every boolean and directive style
to the pre-registry values.

The L1-to-L2 registry selects the existing conditional filter and Vue 2 event
legalization patterns and the existing pipeline, while the baseline `.sync`
and `slot-scope` rewrites retain their order and traversal. It also owns the
bounded selected-SFC admission. L2 registers the remaining scoped-template
meaning. L2-to-L3 and L3 publish explicit empty registries because all current
input distinctions have been erased before those levels. L4 turns the retained
file descriptor into target options for Vue 3 projection and the Vue 2 runtime.
Consumers ask their level registry instead of matching concrete input variants.

The core-isolation owed list is empty. The retired MoonBit dialect, guest SDK
and component host remain absent; neutral foreign-expression test strings do
not restore that architecture. This change does not add syntax, broaden an
admission, change a diagnostic or alter output bytes. Existing replay laws,
the full capability matrix, level-isolation laws and differential corpora are
the behavior proof; exact-source and protected-candidate Actions remain the
merge authority.

## Remaining work under #6841

- Qualify this registry integration on its exact source and protected merge
  candidate, then verify the signed main result before closing the structural
  issue.
- Complete the native Vue 0.x, Vue 1 and quirks behavior tracked separately by
  #6842; registry ownership does not establish those runtime semantics.

This structural provider does not establish complete native parsing,
embedded-language admission, product adoption or legacy-removal parity.
