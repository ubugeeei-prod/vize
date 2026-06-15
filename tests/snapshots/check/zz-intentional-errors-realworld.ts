import { before, describe, it } from "node:test";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

import { directusApp } from "../../_helpers/directus-app.ts";
import {
  type AppConfig,
  elementPlusApp,
  elkApp,
  frontendPhpconApp,
  hoppscotchApp,
  misskeyApp,
  npmxApp,
  nuxtUiApp,
  rekaUiApp,
  requireVizeAndCorsaBins,
  voicevoxApp,
  vueVbenAdminApp,
  vuefesApp,
} from "../../_helpers/apps.ts";
import { naiveUiApp, primeVueApp, vuetifyApp } from "../../_helpers/ui-library-apps.ts";
import { runVizeCheckWithInjectedTypeError } from "../_helpers/realworld.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const testsDir = path.resolve(__dirname, "../..");
const misskeySourceFrontendDir = path.join(
  testsDir,
  "_fixtures",
  "_git",
  "misskey",
  "packages",
  "frontend",
);
const misskeySourceCheckApp: AppConfig = {
  ...misskeyApp,
  cwd: misskeySourceFrontendDir,
  check: {
    cwd: misskeySourceFrontendDir,
    patterns: ["src/**/*.vue"],
  },
};

const misskeyInjectedTsconfig = {
  relativePath: "__vize_injected_check_tsconfig.json",
  content: `{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "types": []
  }
}
`,
};

const naiveUiInjectedTsconfig = {
  relativePath: "__vize_injected_check_tsconfig.json",
  content: `{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "types": [
      "vue/jsx"
    ]
  }
}
`,
};

const realWorldApps = [
  directusApp,
  elementPlusApp,
  elkApp,
  frontendPhpconApp,
  hoppscotchApp,
  misskeySourceCheckApp,
  naiveUiApp,
  npmxApp,
  nuxtUiApp,
  primeVueApp,
  rekaUiApp,
  voicevoxApp,
  vueVbenAdminApp,
  vuefesApp,
  vuetifyApp,
] as const;

describe("real-world vize check injected type errors", () => {
  before(requireVizeAndCorsaBins);

  it("ant-design-vue injected semantic diagnostics are tracked in #1727", { skip: true }, () => {});

  for (const app of realWorldApps) {
    it(`${app.name} catches an injected TS2322`, () => {
      const summary = runVizeCheckWithInjectedTypeError(app, {
        timeoutMs: 300_000,
        tsconfig:
          app.name === "misskey"
            ? misskeyInjectedTsconfig
            : app.name === "naive-ui"
              ? naiveUiInjectedTsconfig
              : undefined,
      });
      console.log(
        `${app.name}: file=${summary.file}, fileCount=${summary.fileCount}, errorCount=${summary.errorCount}, durationMs=${summary.durationMs.toFixed(0)}`,
      );
    });
  }
});
