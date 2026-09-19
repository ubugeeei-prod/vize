import { describe, it, before } from "node:test";
import { runAppBuild } from "../../_helpers/app-build.ts";
import { elkApp } from "../../_helpers/apps.ts";

const app = elkApp;

describe(`${app.name} build`, () => {
  before(() => {
    if (!process.env.RUN_BUILD_TESTS) {
      console.log("Skipping: build tests are opt-in (RUN_BUILD_TESTS=1)");
      process.exit(0);
    }
    if (app.setup) app.setup();
  });

  it("build succeeds", () => {
    runAppBuild(app);

    console.log("Build completed successfully");
  });
});
