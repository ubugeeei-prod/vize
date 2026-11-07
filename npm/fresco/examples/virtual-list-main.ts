/** Run with a Vue-aware TypeScript/Vite loader. */

import { createApp } from "../src/index.js";
import VirtualList from "./virtual-list.vue";

const app = createApp(VirtualList, { exitOnCtrlC: true, mouse: true });
void app.mount();
void app.waitUntilExit();
