# Linter upstream rule contract

Date: 2026-11-06

## Decision

Patina treats the upstream Vue lint ecosystem as explicit compatibility
catalogues, not as an informal source of ideas. A catalogue may claim coverage
only when every rule in the pinned upstream release has a checked-in status:

- `mapped`: a registered Patina rule owns the supported behavior;
- `partial`: the documented subset and omitted options are explicit;
- `intentional-divergence`: Patina deliberately has different semantics and a
  reason is recorded;
- `unimplemented`: the upstream checker remains necessary.

The existing exhaustive `eslint-plugin-vue` fixture is the first catalogue.
Its rule target, default severity and preset membership remain test-checked.
The same ledger and drift checks are required before claiming coverage of the
other upstream catalogues.

## Namespace and preset boundaries

| Upstream | Patina ownership | Default policy |
| --- | --- | --- |
| `eslint-plugin-vue` | `vue/*` and `script/*` | Broad correctness may enter `happy-path`; style remains stronger-preset or opt-in. |
| `@intlify/eslint-plugin-vue-i18n` | `ecosystem/vue-i18n-*` | Explicit rule opt-in or the explicitly selected `ecosystem` preset only; never `essential` or the general default merely because Vue is present. |
| Stylelint core rules | `css/*` | CSS semantics only. Convention-heavy rules remain in stronger presets. |
| Markuplint rules | `html/*`, with accessibility behavior in `a11y/*` | Broad conformance and correctness may be default; project-specific policy remains opt-in. |
| Musea | `musea/*` | Independent opt-in catalogue; Musea rules do not count as CSS or markup parity. |

CSS and Musea therefore have separate catalogue pages, navigation entries and
evidence links. A rule cannot be counted toward two upstream catalogues merely
because it walks the same SFC block.

## Default preset boundary

`essential` remains the low-noise correctness floor. `happy-path` is allowed to
be strong for general Vue applications: undefined template references, unread
setup bindings, reactivity-loss patterns and lifecycle misuse belong there.
Version- or product-specific policy (Vue 2 migration, Vapor migration, Nuxt,
Vue I18n, Musea and project naming conventions) requires an explicit preset or
rule selection.

## SSR, Vapor and ecosystem breadth

The current dedicated SSR and Vapor catalogues are not considered complete.
Rules implemented under a general namespace may be linked from a product page,
but that does not replace a gap ledger for the product. Each expansion must
record:

1. the platform or upstream behavior being followed;
2. whether the rule is general, SSR-only, Vapor-only or ecosystem-only;
3. a failing carrier and a passing control;
4. the preset decision and false-positive boundary;
5. the exact unsupported cases.

Router coverage does not stand in for Pinia, Vue I18n or Vue Test Utils
coverage. Ecosystem progress is reported per integration.

## Acceptance for a new upstream catalogue

1. Pin an upstream package release and retain its complete rule-name list.
2. Give every rule one status from this decision.
3. For mapped and partial entries, assert that the Patina target is registered
   and that recorded severity and preset membership are current.
4. Differentially exercise representative invalid, valid and option-bearing
   cases against the pinned upstream implementation.
5. Generate migration documentation from the same ledger.
6. Do not publish a parity claim while any entry is unclassified.

Until those ledgers exist, Stylelint, Markuplint and Vue I18n compatibility is
direction and partial implementation, not full parity.
