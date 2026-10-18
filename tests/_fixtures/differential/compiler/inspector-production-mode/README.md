# Original Elk production-reference control

`Mutes.vue.txt` retains every byte of the pinned Elk `app/pages/mutes.vue` source.
`source.json` records its upstream revision, content digest and compiler provider;
`LICENSE` retains the upstream MIT license. The four complete reference modules
use identical Inspector options (`isProd: true`) and differ only in the Node
environment selected before loading the compiler.

The development module produces `<!--v-if-->` when MainContent calls the original
false conditional slot without an SSR push callback. The production module emits
`<!---->`. Both environments agree for the true branch and the direct SSR-push
route. The test executes the complete emitted `ssrRender` with actual Vue helpers
and controlled child components that exercise those routes. It does not execute
the original script setup, whole application or browser hydration.

The positive test runs the actual embedded Inspector script under unset,
development and production parent environments and requires the complete
production DOM/SSR outputs. The retained development module is an executable
negative control, not an ignored-output normalization.
