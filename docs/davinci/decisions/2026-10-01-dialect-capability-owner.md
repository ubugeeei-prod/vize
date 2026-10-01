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
generic modules, including grouped imports and references behind `cfg`.
The existing conversion capability bridge remains transitional; removing
inline `caps` branches and isolating all legalization policy is separate work.
This slice introduces no pipeline stage, transport serialization, product-route
switch, fixture change, snapshot change or instruction-budget change.

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

## Remaining work under #6841

- Resolve one file descriptor across Vue version, petite, quirks, markup and
  embedded language axes; quirks remains a dialect.
- Move all dialect-specific legalization policy out of generic conversion
  modules, with a gate enforcing core isolation.
- Complete per-level residual dialect-op semantics and L4 target options.
- Dissolve MoonBit infrastructure into the appropriate per-level language
  modules after its provider contracts exist.

This is one structural provider slice. It does not close #6841 or establish
complete native parsing, embedded-language admission, product adoption or
legacy-removal parity.
