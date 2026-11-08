# External Fresco component provider

This fixture models a separately published component package. `plugin.ts` registers the authored
`ExternalMeter.vue` SFC and installs its typed theme provider; the application consumes the global
component through `createApp(...).use(...)` without importing renderer internals.

The provider owns its injection key and fallback. An application can install a different value in
its own plugin while keeping the component API unchanged.
