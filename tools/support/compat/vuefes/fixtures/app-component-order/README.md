# VueFes explicit component import oracle

This private oracle executes the complete Nuxt 4.0.3 component TransformPlugin
with its original source bytes and unimport 5.2.0. The adjacent MIT license
applies to `nuxt-transform.ts.txt`. The source is materialized unchanged in
ignored `.generated` before execution. The dependency lock is independent
of the existing Vue alpha.2 oracle and does not alter its pins. Its stable
Vue 3.5.35 dependency satisfies the Nuxt/router peer contract; this oracle
executes only the source transform. The existing alpha.2 compiler and runtime
remain the full-SFC/DOM/SSR/hydration oracle.

The original 840-byte `app/app.vue`, all original static imports, and the
complete compiled script are checked against the actual component transform.
A source fixture pass does not prove the complete app cascade; the original
38-case VueFes VRT remains required.

Source: <https://raw.githubusercontent.com/nuxt/nuxt/v4.0.3/packages/nuxt/src/components/plugins/transform.ts>.
