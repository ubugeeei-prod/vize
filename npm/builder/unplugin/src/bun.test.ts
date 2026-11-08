import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { prepareOutputDir, resolveFixturePath } from "./test/helpers.ts";

const bun = spawnSync("bun", ["--version"], { encoding: "utf8" });
const enabled = bun.status === 0;

void test(
  "the actual Bun bundler compiles an imported Vue SFC and emits its scoped CSS",
  { skip: enabled ? false : "Bun is not installed in this host" },
  () => {
    const root = prepareOutputDir("bun-basic");
    const report = path.join(root, "report.json");
    const driver = path.join(root, "build.ts");
    const outdir = path.join(root, "dist");
    const plugin = pathToFileURL(path.join(import.meta.dirname, "bun.ts")).href;
    const entrypoint = resolveFixturePath("basic", "entry.ts");
    fs.writeFileSync(
      driver,
      `import vize from ${JSON.stringify(plugin)};
const result = await Bun.build({
  entrypoints: [${JSON.stringify(entrypoint)}],
  outdir: ${JSON.stringify(outdir)},
  target: "browser",
  format: "esm",
  sourcemap: "external",
  external: ["vue"],
  plugins: [vize({ isProduction: true, root: ${JSON.stringify(root)} })],
});
await Bun.write(${JSON.stringify(report)}, JSON.stringify({
  success: result.success,
  logs: result.logs.map((log) => ({ level: log.level, message: log.message })),
  outputs: await Promise.all(result.outputs.map(async (output) => ({
    path: output.path,
    kind: output.kind,
    text: await output.text(),
  }))),
}, null, 2));
if (!result.success) process.exitCode = 1;
`,
    );
    const result = spawnSync("bun", ["run", driver], {
      cwd: root,
      env: process.env,
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    });
    assert.equal(result.status, 0, result.stderr || result.stdout);
    assert.equal(result.signal, null);
    const observed = JSON.parse(fs.readFileSync(report, "utf8")) as {
      success: boolean;
      logs: { level: string; message: string }[];
      outputs: { path: string; kind: string; text: string }[];
    };
    assert.equal(observed.success, true);
    assert.deepEqual(observed.logs, []);
    const javascript = observed.outputs.find((output) => output.kind === "entry-point")?.text ?? "";
    const css =
      observed.outputs.find((output) => output.kind === "asset" && output.path.endsWith(".css"))
        ?.text ?? "";
    assert.match(javascript, /Hello from Vize unplugin/);
    assert.doesNotMatch(javascript, /<template>/);
    assert.match(css, /\.root\[data-v-[0-9a-f]{8}\]/u);
    assert.match(css, /color:\s*tomato/u);
  },
);
