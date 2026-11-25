# Nuxt modules may extend the host Vue compiler filters

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Extracted from [#9200](https://github.com/ubugeeei-prod/vize/pull/9200).

The pinned Elk consumer failed before startup in historical App E2E
[run 38866347055](https://github.com/ubugeeei-prod/vize/actions/runs/38866347055):
`TypeError: vue: unknown option script`. Elk enables Vue Macros setupSFC, which
adds `include` to Nuxt's resolved `vite.vue` options. Vize's private Nuxt
projection recognized only the four schema-owned compiler fields, so the extra
host filter made it pass the whole object to strict native configuration
validation. That validator then rejected `script`.

The same error recurred in source-built
[App E2E run 39106425397, Elk preview job 117613203829](https://github.com/ubugeeei-prod/vize/actions/runs/39106425397/job/117613203829)
at `4a2f2788e9eb77f1393328fa0f1a476ccb2d7481`. Client compilation failed
before the preview server could start. That source retains the same uncorrected
host boundary. The log proves a current failure, not a successful repair or an
actual resolved-options dump.

Admit `include` and `exclude` only alongside the existing host fields and only
when the Nuxt constructor explicitly supplies its private host context. The
native projection omits this host namespace; the original object, descriptors,
filter patterns, and script options remain intact for the host plugin. Canonical
Vue settings, mixed host/native objects, unknown keys, nonplain objects, symbols,
and accessor evaluation retain their existing strict behavior. The native
schema and parser are unchanged. Host filters are not translated into native
`files` or `ignores`, or transferred to Vize's explicit compiler options.

The three runtime and provenance fixtures under
`tests/_fixtures/differential/config/nuxt-elk-host-options/` are preserved byte
for byte from #9200 head `27c8b1b9e2615de54b4e61b1de7daecbe84d38e7`;
the README receives the factual correction about the absent historical log.
These fixtures contain an authored resolved-input reproduction, not a runtime dump:
the Nuxt 4.4.7 and Vue Macros 3.1.2 shape, an independent complete native
projection oracle, and public source identities. The original README refers
to `original-failure.log`, but that log was not tracked at the original head.
It is not reconstructed here; the historical run URL is the retained failure
provenance. Historical results do not establish current-source acceptance.

The focused host-context test keeps all four existing laws and restores both
original whole-input/filter-form laws, including development, client build,
server build, exact filter identities, mixed canonical/unknown refusals, and
single-evaluation controls. The existing Nuxt 3 build workflow executes this
suite directly alongside compiler-default checks, with fixture and test changes
included in its PR and main-push triggers. Its required merge-queue call also
runs the same suite; no extra build or installation is introduced.

This proves the projection boundary, not complete Elk compatibility. Acceptance
requires the existing preview, dev, and VRT Actions jobs with the same pinned
consumer. No upstream configuration, application assertion, timeout, or visual
snapshot is changed.

## Superseded: the host owns the whole `vue` key

Nuxt 4.7's Vite builder adds the app's `vue/compiler-sfc` module to the same
options as `compiler`, and the field allow-list refused it the same way. Each
new plugin-vue option a Nuxt release or module adds would break the build
again. Nuxt runs Vite with `configFile: false` and its schema always fills
`vite.vue` with plugin-vue options, so that key has never carried Vize settings
in practice: a mixed object was always refused. Inside the explicit Nuxt
constructor context the projection now omits the `vue` key by provenance,
without reading it. Vize settings for Nuxt keep coming from the project Vite
config, `vize.config.*`, and the module options. Other hosts stay strict.
