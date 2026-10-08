# n8n keyed slot fallback regression

[#8281](https://github.com/ubugeeei-prod/vize/issues/8281) is the focused
`vue/require-v-for-key` producer slice of the n8n adoption work in
[#8142](https://github.com/ubugeeei-prod/vize/issues/8142).

The licensed master fixture at `e882e8a483f433facb47bab9b407d0ec00a81172`
contains `packages/frontend/@n8n/design-system/src/components/N8nDatatable/Datatable.vue`,
Git blob `56fc5c2ec8bc9df82f2de32b36b10d95a580a831`, SHA256
`541ce048e84512920a5ca3b8b135315433d81013bafefc7c52fa9f42b9e6d4ca`.
Its template loop contains a slot whose fallback row carries `:key`.
The existing authenticated source CLI artifact at
`5e1dceb1b117687bb6084d7a76d6fec4ab1e645a` reports an error at
92:15–92:41. The complete independent official Vue base packet for that
original is clean. The original has no suppression pragma for this rule.
That CLI receipt is evidence for this original only; the owned cases below
are observed independently through the public source Linter API.

## Decision

Extend the existing child-key search only through transparent `template`
and `slot` children. Stop at ordinary elements: an unkeyed wrapper cannot
borrow a key from its descendant. Retain the historical direct-key,
static-key, object-`v-bind`, dynamic slot forwarding, slot-outlet and
any-directly-keyed-child contracts. This is a focused key discovery fix,
not a replacement of the native rule's complete policy with ESLint's policy.
It adds no parse or pipeline stage and no child collection allocation.
The descendant search accepts bound child keys only. Keep the original
loop-root slot, static-key and object-binding exemptions at that root;
recursing through the complete root helper would incorrectly inherit them.

The independently authored reproduction is:

```vue
<script setup>
const rows = [{ id: 1 }];
</script>
<template>
  <table>
    <tbody>
      <template v-for="row in rows">
        <slot :row="row"
          ><tr :key="row.id">
            <td>{{ row.id }}</td>
          </tr></slot
        >
      </template>
    </tbody>
  </table>
</template>
```

## Evidence and limits

The [owned corpus](../../crates/vize_patina/tests/fixtures/slot-fallback-key/cases.json)
records twenty-one sources, complete pinned independent packets, source API
before/after packets and their producer metadata. The independent provider uses
all 51 selected adoption rules and the three explicit options, plus the
mandatory official `vue/vue` processor, `vue/comment-directive` and
`vue/jsx-uses-vars` infrastructure. Its versions are ESLint 10.4.1,
eslint-plugin-vue 10.9.2, vue-eslint-parser 10.4.1 and
@typescript-eslint/parser 8.65.0. Public entrypoint and package manifest hashes
are recorded and checked.

The [provider replay](../../tests/tooling/patina-slot-fallback-key-oracle.test.mjs)
preserves raw packets and provider errors before assertions. Its named
recording step normalizes only the known ephemeral absolute `filePath`;
every other finding, field, fix, suggestion, suppression, output and count
remains part of the whole comparison.

The [source API test](../../crates/vize_patina/tests/require_v_for_key_slot_fallback.rs)
uses the actual public `Linter::lint_sfc` with the owned rule registry.
All sixteen before packets are retained before the expected failing
assertion. The producer starts at fresh main `f399905053` and uses the
installed declared Rust 1.99.0 compiler directly with a private target.
The failed Nix selector attempt and the separate Rust 1.98.1 observation
are retained locally; they provide no declared-toolchain qualification.
The original sixteen Before/After packets are retained. Review of the first
PR producer at `5bf1959473` added five nested exemption inversions: all five
incorrectly became clean. A separate actual producer run with the historical
rule restored reports an error for each. The corrected descendant-only search
restores those errors while retaining the original keyed fallback fix. The
full provider packets also retain the slot-parent and template-key findings;
the controls do not imply their invalid template forms are supported.
The before, reviewed and after binary hashes, source hashes, patch hash and raw
packet hashes distinguish the fresh-main base from the modified producer.

ESLint recursively checks every ordinary fallback child. Native history
accepts any discoverable child key. The corpus explicitly records the empty
slot, mixed keyed/unkeyed children, static key and object-binding differences;
those packets are not discarded or described as parity. Script/template
comment boundaries, the adoption projection and existing file overrides
are unchanged. No upstream branch source is executed, redistributed or
changed. This slice does not qualify all 51 semantics, whole-monorepo
adoption, product migration, releases or performance.

## Verification

- Declared Rust 1.99.0 passes the twenty-one complete owned source API packets,
  the existing key/object-bind/template-child-key history regressions and
  the pinned complete independent provider replay.
- Verify exact-head Actions and the protected merge queue before completion.
