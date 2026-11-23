import fs from "node:fs/promises";
import { captureAndCompare } from "../src/vrt/runner-comparison.ts";
const [root, source] = process.argv.slice(2);
const closed: string[] = [];
const page = {
  async goto() {},
  async waitForSelector() {},
  async waitForTimeout() {},
  async screenshot({ path }: { path: string }) {
    await fs.copyFile(source, path);
  },
  async close() {
    closed.push("page");
  },
};
const context = {
  async newPage() {
    return page;
  },
  async close() {
    closed.push("context");
  },
};
const runner = {
  getBrowser() {
    return {
      async newContext() {
        return context;
      },
    };
  },
  getOptions() {
    return { snapshotDir: root, threshold: 0 };
  },
  getCapture() {
    return { hideElements: [], maskElements: [] };
  },
  getComparison() {
    return {};
  },
  getHostedNavigation() {},
  getPreviewUrl() {
    return "http://localhost/preview";
  },
  async getSnapshotName() {
    return "baseline.png";
  },
};
const result = await captureAndCompare(
  runner as unknown as Parameters<typeof captureAndCompare>[0],
  { path: "Button.art.vue" } as Parameters<typeof captureAndCompare>[1],
  "Default",
  { width: 1, height: 1 },
  "http://localhost",
);
process.stdout.write(JSON.stringify({ result, closed }));
