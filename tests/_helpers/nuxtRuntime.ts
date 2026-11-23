import { fileURLToPath } from "node:url";
import { installNuxtGlobals } from "./browser/nuxt-globals.ts";
import type { Page } from "@playwright/test";

export async function disableViteHmrClient(page: Page): Promise<void> {
  await page.addInitScript(installNuxtGlobals);
  await page.route("**/_nuxt/@vite/client", (route) =>
    route.fulfill({
      path: fileURLToPath(new URL("./browser/vite-client.mjs", import.meta.url)),
      contentType: "text/javascript",
      status: 200,
    }),
  );
}
