import assert from "node:assert/strict";
import { attachMuseaOptions } from "../plugin/options.ts";

export default {
  plugins: [
    attachMuseaOptions(
      { name: "musea-fixture" },
      {
        vrt: {
          capture: {
            waitForNetwork: false,
            waitForPreviewReady: false,
            settleTime: 0,
            async beforeCapture(page, context) {
              assert.equal(context.variantName, "Submitted");
              await page.getByRole("button", { name: "Prepare" }).click();
              assert.equal(await page.locator("#result").textContent(), "Prepared by config");
            },
          },
        },
      },
    ),
  ],
};
