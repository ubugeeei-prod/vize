# Fresco plugins and providers

Fresco applications accept Vue plugins before their first mount. `defineFrescoPlugin` gives
component packages a small declarative surface for global SFC registration and typed provider
setup, while `defineFrescoProvider` owns the injection key and fallback contract.

```ts
import { createApp, defineFrescoPlugin, defineFrescoProvider } from "@vizejs/fresco";
import ExternalMeter from "./ExternalMeter.vue";

const meterTheme = defineFrescoProvider("meter-theme", {
  filled: "█",
  empty: "░",
  foreground: "cyan",
});

const meterPlugin = defineFrescoPlugin({
  components: { ExternalMeter },
  setup(app) {
    meterTheme.provide(app, { filled: "■", empty: "·", foreground: "green" });
  },
});

const app = createApp(App).use(meterPlugin);
await app.mount();
```

Components call `meterTheme.use()` during setup. When no plugin value exists, the declared fallback
is returned. A provider without a fallback throws a descriptive error when it is missing.

Plugins must be installed before the first `mount()`. This keeps application setup deterministic
and prevents providers or global component resolution from changing after the tree exists. The
complete external component fixture lives in
[`npm/fresco/examples/external-plugin`](../../npm/fresco/examples/external-plugin/README.md).
