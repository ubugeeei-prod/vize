# Nuxt modules may extend the host Vue compiler filters

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The unchanged pinned Elk consumer failed before startup in App E2E
[run 38866347055](https://github.com/ubugeeei-prod/vize/actions/runs/38866347055):
`TypeError: vue: unknown option script`. Elk enables Vue Macros setupSFC, which
adds `include` to Nuxt's resolved `vite.vue` options. Vize's private Nuxt
projection recognized only the four schema-owned compiler fields, so the extra
host filter made it pass the whole object to strict native configuration
validation. That validator then rejected `script`.

Admit `include` and `exclude` only alongside the existing host fields and only
when the Nuxt constructor explicitly supplies its private host context. The
native projection omits this host namespace; the original object, descriptors,
filter patterns, and script options remain intact for the host plugin. Canonical
Vue settings, mixed host/native objects, unknown keys, nonplain objects, symbols,
and accessor evaluation retain their existing strict behavior. The native
schema and parser are unchanged.

The authored input and provenance live in
`tests/_fixtures/differential/config/nuxt-elk-host-options/`. They retain the
resolved Nuxt 4.4.7 and Vue Macros 3.1.2 shape, an independent complete native
projection oracle, the failed development stack, and public source identities.
The focused test failed with the same error before the production change; all
26 focused host/projector/history tests passed afterward, including existing
refusal and single-evaluation controls.

This proves the projection boundary, not complete Elk compatibility. Acceptance
requires the existing preview, dev, and VRT Actions jobs with the same pinned
consumer. No upstream configuration, application assertion, timeout, or visual
snapshot is changed.
