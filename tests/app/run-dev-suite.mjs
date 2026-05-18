import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const specs = [
  "app/dev/vuefes.spec.ts",
  "app/dev/elk.spec.ts",
  "app/dev/misskey.spec.ts",
  "app/dev/npmx.spec.ts",
];

const playwrightBin = process.platform === "win32" ? "playwright.cmd" : "playwright";
const testsDir = fileURLToPath(new URL("..", import.meta.url));
const worktreeId =
  process.env.VIZE_TEST_WORKTREE_ID ?? (process.env.CI ? "ci-dev" : `pid-${process.pid}`);

for (const spec of specs) {
  console.log(`\n[dev-suite] Running ${spec}`);

  const result = spawnSync(playwrightBin, ["test", "--config", "app/playwright.config.ts", spec], {
    cwd: testsDir,
    env: {
      ...process.env,
      VIZE_TEST_WORKTREE_ID: worktreeId,
    },
    stdio: "inherit",
  });

  if (result.error) {
    throw result.error;
  }

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
