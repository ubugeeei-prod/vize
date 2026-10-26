import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { startTsserver } from "./tsserver-client.mjs";
import type { createInheritedConfig } from "./inherited-config.ts";

// This is an explicitly unqualified host boundary, separate from the Vue role.
// Its full wrong warm packet is never substituted for the correct fresh oracle.
export async function createOrdinaryHostFrontier(
  fixture: ReturnType<typeof createInheritedConfig>,
  consumer: string,
  tsserver: string,
  output: string,
) {
  const bytes = fs.readFileSync(
    fileURLToPath(
      new URL("../_fixtures/typescript-plugin/ordinary-host-frontier-10752.json", import.meta.url),
    ),
  );
  assert.equal(
    createHash("sha256").update(bytes).digest("hex"),
    "046eaaca3d9b35faa57b47204aea5d409e6b1321392c0c4d575623a10f873840",
  );
  const known = JSON.parse(bytes.toString("utf8"));
  assert.equal(known.issue, 10752);
  assert.equal(known.qualified, false);
  const root = fs.mkdtempSync(path.join(path.dirname(consumer), "ordinary-host-"));
  const sourceFile = path.join(root, "src/control.ts");
  fs.mkdirSync(path.dirname(sourceFile));
  fs.writeFileSync(sourceFile, fixture.controlSource);
  const packageRoot = path.join(root, "node_modules/vize-config-priority");
  fs.mkdirSync(packageRoot, { recursive: true });
  const directory = path.join(output, "ordinary-host-frontier");
  fs.mkdirSync(directory);
  const observations: Record<string, any>[] = [];
  const writeChanged = (file: string, bytes: string | Buffer) => {
    const content = Buffer.from(bytes);
    if (!fs.existsSync(file) || !content.equals(fs.readFileSync(file)))
      fs.writeFileSync(file, content);
  };
  function sync() {
    const config = JSON.parse(fs.readFileSync(fixture.root, "utf8"));
    delete config.compilerOptions.plugins;
    writeChanged(path.join(root, "tsconfig.json"), JSON.stringify(config));
    writeChanged(path.join(root, "inherited.json"), fs.readFileSync(fixture.base));
    for (const name of ["package.json", "tsconfig.json", "preferred.json"]) {
      const original = path.join(consumer, "node_modules/vize-config-priority", name);
      const target = path.join(packageRoot, name);
      if (fs.existsSync(original)) {
        const content = fs.readFileSync(original);
        writeChanged(target, content);
      } else if (fs.existsSync(target)) fs.unlinkSync(target);
    }
  }
  sync();
  const warmDirectory = path.join(directory, "warm");
  fs.mkdirSync(warmDirectory);
  const warm = startTsserver(tsserver, root, warmDirectory);
  await warm.send("open", { file: sourceFile, fileContent: fixture.controlSource }, false);
  // Materialize the actual initial ordinary project before replaying edits.
  await warm.send("semanticDiagnosticsSync", { file: sourceFile });
  const save = () =>
    fs.writeFileSync(
      path.join(output, "ordinary-host-frontier.json"),
      JSON.stringify(
        {
          role: known.role,
          issue: known.issue,
          qualified: false,
          status: "known-failure-frontier",
          witnessSha256: createHash("sha256").update(bytes).digest("hex"),
          observations,
        },
        null,
        2,
      ),
    );
  return {
    async observe(name: string, pluginWarm: any, pluginFresh: any, expected: unknown) {
      sync();
      const observation: Record<string, any> = {
        name,
        config: JSON.parse(fs.readFileSync(path.join(root, "tsconfig.json"), "utf8")),
        pluginWarm,
        pluginFresh,
      };
      assert.equal(observation.config.compilerOptions.plugins, undefined);
      observations.push(observation);
      save();
      await warm.send("reloadProjects", {}, false);
      observation.warm = await warm.send("semanticDiagnosticsSync", { file: sourceFile });
      save();
      const freshDirectory = path.join(directory, name);
      fs.mkdirSync(freshDirectory);
      const fresh = startTsserver(tsserver, root, freshDirectory);
      try {
        await fresh.send("open", { file: sourceFile, fileContent: fixture.controlSource }, false);
        observation.fresh = await fresh.send("semanticDiagnosticsSync", { file: sourceFile });
      } finally {
        await fresh.stop();
        save();
      }
      assert.equal(observation.warm.success, true);
      assert.equal(observation.fresh.success, true);
      assert.deepEqual(
        pluginWarm.body,
        observation.warm.body,
        "ordinary warm TypeScript behavior is unchanged by the Vue adapter",
      );
      assert.deepEqual(
        pluginFresh.body,
        observation.fresh.body,
        "ordinary fresh TypeScript behavior is unchanged by the Vue adapter",
      );
      if (expected)
        assert.deepEqual(
          observation.fresh.body,
          expected,
          "the complete correct fresh oracle remains strict",
        );
      if (JSON.stringify(observation.warm.body) !== JSON.stringify(observation.fresh.body)) {
        assert.deepEqual(observation.warm.body, known.expected.warm);
        assert.deepEqual(observation.fresh.body, known.expected.fresh);
        observation.status = "known-failure";
        observation.qualified = false;
      } else {
        // Equality here is merely a boundary observation, never feature credit.
        observation.status = "frontier-not-observed";
        observation.qualified = false;
      }
      save();
    },
    async stop() {
      await warm.stop();
      save();
      for (const file of [
        path.join(warmDirectory, "tsserver.log"),
        ...observations.map((row) => path.join(directory, row.name, "tsserver.log")),
      ])
        assert(
          !fs.readFileSync(file, "utf8").includes("[vize] TypeScript native provider"),
          "the ordinary comparison workers must not load Vize",
        );
    },
    summary() {
      return {
        role: known.role,
        issue: known.issue,
        qualified: false,
        status: "known-failure-frontier",
        cases: observations.map(({ name, status, qualified }) => ({ name, status, qualified })),
      };
    },
  };
}
