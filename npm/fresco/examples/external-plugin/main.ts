import { createApp } from "@vizejs/fresco";

import App from "./App.vue";
import { externalMeterPlugin } from "./plugin.js";

const app = createApp(App, { exitOnCtrlC: true });
app.use(externalMeterPlugin);

await app.mount();
await app.waitUntilExit();
