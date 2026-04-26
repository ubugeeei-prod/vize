import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./dev",
  testMatch: "**/*.spec.ts",
  testIgnore: ["**/node_modules/**", "**/_git/**"],
  timeout: 300_000,
  expect: { timeout: 30_000 },
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
  use: {
    headless: true,
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
    video: "retain-on-failure",
  },
  projects: [
    {
      name: "dev",
      testMatch: "**/*.spec.ts",
    },
  ],
  outputDir: "./results",
});
