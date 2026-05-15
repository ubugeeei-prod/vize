/**
 * Fresco SFC Example
 *
 * This example demonstrates how to use Vue SFC with Fresco.
 * Since Fresco uses a custom renderer, templates compile to our custom elements.
 *
 * Run with: pnpm dev
 */

import { createApp } from "@vizejs/fresco";
import App from "./App.vue";

async function main() {
  const app = createApp(App, {
    exitOnCtrlC: true,
  });

  await app.mount();
  await app.waitUntilExit();
  console.log("Goodbye!");
}

main().catch(console.error);
