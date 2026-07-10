import assert from "node:assert/strict";

import { transformHook } from "./load.ts";
import type { VizePluginState } from "./state.ts";

const state = {
  filter: () => true,
  mergedOptions: { vapor: false },
  root: "/src",
  logger: {
    log() {},
    info() {},
    warn() {},
    error() {},
  },
} as unknown as VizePluginState;

const result = await transformHook(
  state,
  `export const App = () => <div>Hello</div>;\n`,
  "/src/App.jsx",
  { ssr: false },
);

assert.equal(
  result,
  null,
  "Plain JSX modules should stay on Vite's regular JSX pipeline unless include opts them into Vize",
);

console.log("vite-plugin-vize JSX passthrough tests passed!");
