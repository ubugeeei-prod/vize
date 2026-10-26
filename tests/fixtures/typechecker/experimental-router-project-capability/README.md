# Project-owned experimental route props capability

`artifacts.json` binds unchanged package, public-entry and shared-declaration
members of the actual published `vue-router@5.4.0` archive and the source build
at upstream commit `593d7ad9ff99e95c24be28e962717df0fb2be763`. Both manifests
report `5.4.0`. Only the source build exports `_RoutePropsForFilePath` from
`vue-router/experimental` and gives `definePage` a component type parameter.

Issue #10579: with the typed-page plugin enabled, `vize check` follows the
resolved declarations, never the version. The source build admits the plugin's
`props: true` projection (component generic, `defineProps` helper); the published
archive keeps the file-only projection.

- `typed_router/provider/tests.rs` follows the real export graph of both packs:
  simultaneous providers, star re-exports, removed targets and source shadowing.
- `typed_router/route_props_tests.rs` installs these members into a project and
  checks admission, revocation on package or config edits, and the generated
  public, template and setup contracts.
- `router-preview-conformance.yml` compares `vize check` with vue-tsc running
  the real plugin of both packs in Actions (`router-preview-automatic.ts`).

Known gap: a `props` function whose return type is wrong is not reported by
vue-tsc (its self-import resolves circularly), while vize reports it.
