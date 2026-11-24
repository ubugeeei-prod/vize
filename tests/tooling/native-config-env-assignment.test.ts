import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { nativeLoaderPath } from "./support/native-config-script.ts";

const variable = "VIZE_CONFIG_ENV_ASSIGNMENT_TEST";

for (const initial of [undefined, "before"]) {
  for (const assignment of [
    `process.env.${variable} = "after";`,
    `Object.assign(process.env, { ${variable}: "after" });`,
  ]) {
    void test(`recorded config accepts ${assignment} with initial ${initial}`, async (t) => {
      const root = await realpath(await mkdtemp(join(tmpdir(), "vize-config-env-")));
      t.after(() => rm(root, { recursive: true, force: true }));
      const config = join(root, "vite.config.mjs");
      await writeFile(
        config,
        `${assignment}\nexport default { vue: { formatter: { printWidth: process.env.${variable} === "after" ? 93 : 80 } } };\n`,
      );
      const env = { ...process.env, VIZE_RECORD_CONFIG_INPUTS: "1" };
      if (initial === undefined) delete env[variable];
      else env[variable] = initial;
      const result = spawnSync(process.execPath, [await nativeLoaderPath(), config], {
        cwd: root,
        env,
        encoding: "utf8",
        timeout: 30_000,
      });
      assert.equal(result.error, undefined);
      assert.equal(result.status, 0, result.stderr);
      const output = JSON.parse(result.stdout);
      assert.equal(output.config.formatter.printWidth, 93);
      assert.deepEqual(
        output.inputs.env.filter(([name]: [string, boolean]) => name === variable),
        [[variable, true]],
      );
    });
  }
}
