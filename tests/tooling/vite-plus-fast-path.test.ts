import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../", import.meta.url));
const plugin = path.join(root, "npm/builder/vite");
const require = createRequire(path.join(plugin, "package.json"));
const vpPackage = require.resolve("vite-plus/package.json");
const vp = path.join(path.dirname(vpPackage), "bin/vp");

function run(cwd: string, args: string[], expected = 0) {
  const result = spawnSync(process.execPath, [vp, ...args], {
    cwd,
    encoding: "utf8",
    timeout: 120_000,
    env: { ...process.env, NO_COLOR: "1", FORCE_COLOR: "0" },
  });
  assert.equal(result.error, undefined);
  const output = result.stdout + result.stderr;
  assert.equal(result.status, expected, output);
  return output;
}

test("packed withVue runs native and Vite+ tools using the consumer's installed peer", () => {
  run(path.join(root, "npm/cli"), ["pack"]);
  run(plugin, ["pack"]);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vp-consumer space-"));
  try {
    fs.mkdirSync(path.join(directory, "node_modules/@vizejs"), { recursive: true });
    for (const [name, source] of [
      ["vite-plus", path.dirname(vpPackage)],
      ["@vizejs/vite-plugin", plugin],
      [
        "vue",
        path.dirname(
          createRequire(path.join(root, "tests/package.json")).resolve("vue/package.json"),
        ),
      ],
    ])
      fs.symlinkSync(
        source,
        path.join(directory, "node_modules", name),
        process.platform === "win32" ? "junction" : "dir",
      );
    const write = (name: string, value: string) =>
      fs.writeFileSync(path.join(directory, name), value);
    write("package.json", JSON.stringify({ name: "vize-consumer", type: "module", private: true }));
    write(
      "vite.config.ts",
      `import { withVue } from "@vizejs/vite-plugin/vite-plus";
export default withVue({ linter: { preset: "essential", rules: { "vue/no-v-html": "error" } } }).vp({
  plugins: [{ name: "vite:vue", transform() { throw new Error("Old Vue compiler must be replaced"); } }],
  lint: { rules: { "no-debugger": "error" } },
  fmt: { ignorePatterns: ["dist/**"] },
});
`,
    );
    write(
      "App.vue",
      '<script setup lang="ts">\nconst html = "hello";\n</script>\n<template><div v-html="html" /></template>\n',
    );
    write("script.ts", "debugger;\nexport const value=1;\n");
    const lint = run(directory, ["run", "lint"], 1);
    assert.match(lint, /vize:vue\/no-v-html/);
    assert.match(lint, /no-debugger/);
    assert.doesNotMatch(lint, /not found in plugin|oxlint-plugin-vize/);

    write(
      "App.vue",
      '<script setup lang="ts">\nconst greeting="hello";\n</script>\n<template><div>{{greeting}}</div></template>\n',
    );
    write("script.ts", "export const value=1;\n");
    run(directory, ["run", "fmt:check"], 1);
    run(directory, ["run", "fmt"]);
    const formatted = fs.readFileSync(path.join(directory, "App.vue"), "utf8");
    run(directory, ["run", "fmt:check"]);
    run(directory, ["run", "fmt"]);
    assert.equal(fs.readFileSync(path.join(directory, "App.vue"), "utf8"), formatted);
    run(directory, ["run", "lint"]);

    write("index.html", '<div id="app"></div><script type="module" src="/main.ts"></script>');
    write(
      "main.ts",
      'import { createApp } from "vue"; import App from "./App.vue"; createApp(App).mount("#app");',
    );
    run(directory, ["run", "build"]);
    assert.ok(fs.existsSync(path.join(directory, "dist/index.html")));
    write(
      "tsconfig.json",
      JSON.stringify({
        compilerOptions: {
          strict: true,
          skipLibCheck: true,
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "bundler",
          types: [],
        },
        include: ["App.vue", "main.ts", "script.ts"],
      }),
    );
    run(directory, ["run", "fmt"]);
    run(directory, ["run", "check"]);
    assert.ok(!fs.readdirSync(directory).some((name) => name.startsWith(".vize-vp-")));
    checkConsumerTypes(directory, write);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

function checkConsumerTypes(directory: string, write: (name: string, value: string) => void) {
  write(
    "tsconfig.consumer.json",
    JSON.stringify({
      compilerOptions: {
        noEmit: true,
        strict: true,
        skipLibCheck: true,
        target: "ES2022",
        module: "NodeNext",
      },
      files: ["consumer.mts"],
    }),
  );
  write(
    "consumer.mts",
    `import { defineConfig } from "vite-plus";
import { withVue } from "@vizejs/vite-plugin/vite-plus";
declare module "vite-plus" { interface UserConfig { futureOption?: { enabled: boolean }; } }
export default defineConfig(withVue().vp({ futureOption: { enabled: true } }));
// @ts-expect-error Consumer Vite+ owns this option's type.
withVue().vp({ futureOption: { enabled: "wrong" } });
`,
  );
  const tsc = path.join(path.dirname(require.resolve("typescript/package.json")), "bin/tsc");
  const result = spawnSync(process.execPath, [tsc, "-p", "tsconfig.consumer.json"], {
    cwd: directory,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stdout + result.stderr);
}
