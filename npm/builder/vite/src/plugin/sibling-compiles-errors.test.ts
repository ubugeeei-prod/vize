/**
 * The SSR result a client batch compiled with errors is not kept: the SSR
 * environment compiles the file itself, so an error is raised by its own
 * environment's compile.
 */

// Must stay the first imports: the SSR target of a multi-target call fails for
// `Broken.vue`, and the recorder wraps that failing function.
import { INJECTED_SSR_FAILURE } from "../test/fail-ssr-target.ts";
import { labelledComponent, scratchProject } from "../test/native-compile-calls.ts";

import assert from "node:assert/strict";
import { test } from "node:test";

import { buildApp } from "../test/app-build.ts";
import { linkWorkspaceVue } from "../test/startup-scan-project.ts";

void test("an SSR result compiled with errors is compiled again by the SSR environment", async () => {
  const components = ["src/components/Broken.vue", "src/components/Item0.vue"];
  const scratch = scratchProject({
    "src/app.ts": `import A from "./components/Broken.vue";\nimport B from "./components/Item0.vue";\n\nexport const components = [A, B];\n`,
    "src/main.ts": `import { components } from "./app";\n\nconsole.log(components);\n`,
    "src/entry-server.ts": `import { components } from "./app";\n\nexport const render = () => components;\n`,
    ...Object.fromEntries(components.map((file) => [file, labelledComponent(file)])),
  });
  try {
    linkWorkspaceVue(scratch.root);
    const built = await buildApp(scratch.root, {
      client: { entry: "src/main.ts" },
      ssr: { entry: "src/entry-server.ts", consumer: "server" },
    });
    assert.equal(built.error, undefined);
    assert.deepEqual(built.calls, {
      client: [{ api: "targets", ssr: false, alsoSsr: true, files: components }],
      ssr: [{ api: "single", ssr: true, files: ["src/components/Broken.vue"] }],
    });
    const ssr = Object.values(built.outputs.ssr).join("\n");
    assert.equal(ssr.includes('"src/components/Broken.vue"'), true);
    assert.equal(ssr.includes(INJECTED_SSR_FAILURE), false);
  } finally {
    scratch.cleanup();
  }
});
