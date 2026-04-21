import { describe, it, before } from "node:test";
import { execSync } from "node:child_process";
import { elkApp } from "../../_helpers/apps.ts";

const app = elkApp;
const VITE_PLUS_BIN = `${process.env.HOME ?? ""}/.vite-plus/bin`;

describe(`${app.name} build`, () => {
  before(() => {
    if (!process.env.RUN_BUILD_TESTS) {
      console.log("Skipping: build tests are opt-in (RUN_BUILD_TESTS=1)");
      process.exit(0);
    }
    if (app.setup) app.setup();
  });

  it("build succeeds", () => {
    const build = app.build!;
    const cmd = `${build.command} ${build.args.join(" ")}`;
    console.log(`Running: ${cmd} (cwd: ${app.cwd})`);

    execSync(cmd, {
      cwd: app.cwd,
      env: {
        ...process.env,
        PATH: `${VITE_PLUS_BIN}:${process.env.PATH}`,
        NODE_ENV: "production",
        ...app.env,
      },
      stdio: "inherit",
      timeout: build.timeout,
    });

    console.log("Build completed successfully");
  });
});
