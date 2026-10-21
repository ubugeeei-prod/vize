import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, realpath, rm, symlink, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import {
  failedConfigModuleRequests,
  importConfigModuleWithDependencies,
} from "../../crates/vize_carton/src/config/loader/module-runtime.mjs";

const require = createRequire(import.meta.url);
const vite = dirname(require.resolve("vite/package.json"));
const loader = new URL("../../crates/vize_carton/src/config/loader/js.rs", import.meta.url);
const rust = await readFile(loader, "utf8");
const embedded =
  (
    await Promise.all(
      [...rust.matchAll(/include_str!\("([^"]+)"\)/g)].map((match) =>
        readFile(new URL(match[1], loader), "utf8"),
      ),
    )
  ).join("") + [...rust.matchAll(/r#"([\s\S]*?)"#/g)][0][1];

test("Node retains the actual explicit import symlink route for retarget repair", async () => {
  const root = await realpath(await mkdtemp(join(tmpdir(), "vize-config-alias-")));
  try {
    const app = join(root, "app");
    await mkdir(app);
    const selected = join(app, "vite.config.mjs");
    const alias = join(root, "共有 policy.mjs");
    const targets = [join(root, "one.mjs"), join(root, "two.mjs")];
    const contents = targets.map(
      (_, index) => `export default { formatter: { printWidth: ${91 + index} } };`,
    );
    await Promise.all(targets.map((file, index) => writeFile(file, contents[index])));
    const counter = join(root, "evaluations.txt");
    const source = `import { appendFileSync } from 'node:fs';
import vue from '../共有 policy.mjs';
export default () => { appendFileSync(${JSON.stringify(counter)}, 'once\\n'); return { vue }; };`;
    await writeFile(selected, source);
    const invoke = () =>
      spawnSync(process.execPath, ["--input-type=module", "-e", embedded, selected], {
        cwd: app,
        encoding: "utf8",
      });
    for (const [index, target] of targets.entries()) {
      if (index) await rm(alias);
      await symlink(target, alias);
      const response = invoke();
      assert.equal(response.status, 0, response.stderr);
      const loaded = JSON.parse(response.stdout);
      assert.equal(loaded.config.formatter.printWidth, 91 + index);
      assert.equal(loaded.watchDependenciesComplete, true);
      assert.ok(loaded.watchDependencies.includes(alias), "the requested route stays watchable");
      assert.equal(loaded.dependencies.includes(alias), false, "the authored graph stays resolved");
      assert.ok(loaded.dependencies.includes(target), "the actual resolved target stays watchable");
      assert.equal(await readFile(selected, "utf8"), source);
      assert.deepEqual(await Promise.all(targets.map((file) => readFile(file, "utf8"))), contents);
    }
    assert.equal(await readFile(counter, "utf8"), "once\nonce\n");
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("real Vite bundles retain canonical targets but do not expose symlink request routes", async () => {
  const root = await realpath(await mkdtemp(join(tmpdir(), "vize-vite-config-alias-")));
  try {
    const app = join(root, "app");
    await mkdir(join(app, "node_modules"), { recursive: true });
    await symlink(vite, join(app, "node_modules/vite"), "junction");
    const selected = join(app, "vite.config.mjs");
    const alias = join(root, "policy.mjs");
    const targets = [join(root, "one.mjs"), join(root, "two.mjs")];
    await Promise.all(
      targets.map((file, index) =>
        writeFile(file, `export default { formatter: { printWidth: ${91 + index} } };`),
      ),
    );
    await writeFile(selected, "import vue from '../policy.mjs'; export default { vue };");
    const invoke = () =>
      spawnSync(process.execPath, ["--input-type=module", "-e", embedded, selected], {
        cwd: app,
        encoding: "utf8",
      });

    for (const [index, target] of targets.entries()) {
      if (index) await rm(alias);
      await symlink(target, alias);
      const response = invoke();
      assert.equal(response.status, 0, response.stderr);
      const loaded = JSON.parse(response.stdout);
      assert.equal(loaded.config.formatter.printWidth, 91 + index);
      assert.ok(loaded.dependencies.includes(target), "the canonical bundle target stays watched");
      assert.equal(
        loaded.dependencies.includes(alias) || loaded.watchDependencies.includes(alias),
        false,
        "Vite's public bundle result does not expose the requested symlink route",
      );
    }
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

for (const installedVite of [false, true]) {
  for (const policyName of ["shared-policy.mjs", "共有 policy.mjs"]) {
    test(`native loader retains an explicit outside failed request and repairs without replacing its loader (Vite: ${installedVite}, path: ${policyName})`, async () => {
      const root = await realpath(await mkdtemp(join(tmpdir(), "vize-config-repair-")));
      try {
        const app = join(root, "app");
        await mkdir(app);
        await writeFile(join(app, "package.json"), '{"type":"module"}');
        if (installedVite) {
          await mkdir(join(app, "node_modules"));
          await symlink(vite, join(app, "node_modules/vite"), "junction");
        }
        const selected = join(app, "vite.config.mjs");
        const policy = join(root, policyName);
        await writeFile(
          selected,
          `import vue from ${JSON.stringify(`../${policyName}`)}; export default { vue };`,
        );
        const invoke = () =>
          spawnSync(process.execPath, ["--input-type=module", "-e", embedded, selected], {
            cwd: app,
            encoding: "utf8",
          });
        const missing = invoke();
        assert.equal(missing.status, 1, missing.stderr);
        const observed = JSON.parse(missing.stdout);
        assert.equal("config" in observed, false, "failed options must not become a configuration");
        assert.ok(observed.dependencies.includes(selected));
        assert.ok(observed.dependencies.includes(policy));
        assert.equal(observed.watchDependenciesComplete, !installedVite);
        await writeFile(policy, "export default { formatter: { printWidth: 91 } };");
        const repaired = invoke();
        assert.equal(repaired.status, 0, repaired.stderr);
        const loaded = JSON.parse(repaired.stdout);
        assert.equal(loaded.config.formatter.printWidth, 91);
        assert.equal(loaded.watchDependenciesComplete, true);
        assert.ok(loaded.dependencies.includes(policy));
        await rm(policy);
        const deleted = invoke();
        assert.equal(deleted.status, 1, deleted.stderr);
        assert.deepEqual(JSON.parse(deleted.stdout), observed);
        await writeFile(policy, "export default { formatter: { printWidth: 87 } };");
        const recreated = invoke();
        assert.equal(recreated.status, 0, recreated.stderr);
        assert.equal(JSON.parse(recreated.stdout).config.formatter.printWidth, 87);
      } finally {
        await rm(root, { recursive: true, force: true });
      }
    });
  }
}

test("failed request observations use native structured fields without guessing package or extension resolution", () => {
  const importer = join(tmpdir(), "app", "vite.config.mjs");
  const requested = join(tmpdir(), "shared-policy.mjs");
  assert.deepEqual(
    failedConfigModuleRequests({
      code: "ERR_MODULE_NOT_FOUND",
      url: pathToFileURL(requested).href,
    }),
    [requested],
  );
  assert.deepEqual(
    failedConfigModuleRequests({
      errors: [{ code: "UNRESOLVED_IMPORT", id: importer, exporter: "../shared-policy.mjs" }],
    }),
    [importer, requested],
  );
  for (const exporter of [
    "policy",
    "@scope/policy",
    "../policy",
    "https://example.com/policy.mjs",
    "../hash#policy.mjs",
    "../query?policy.mjs",
    "../percent%25policy.mjs",
  ]) {
    assert.deepEqual(
      failedConfigModuleRequests({
        errors: [{ code: "UNRESOLVED_IMPORT", id: importer, exporter }],
      }),
      [importer],
    );
  }
  for (const error of [
    null,
    "Cannot resolve ../shared-policy.mjs",
    { message: `Cannot resolve ${requested}` },
    { errors: [{ text: `Cannot resolve ${requested}`, location: { file: importer } }] },
    { errors: [{ code: "CUSTOM_RESOLVE_ERROR", id: importer, exporter: "../shared-policy.mjs" }] },
    { code: "ERR_MODULE_NOT_FOUND", url: "https://example.com/policy.mjs" },
  ]) {
    assert.deepEqual(failedConfigModuleRequests(error), []);
  }
});

test("incomplete failed observations preserve the original exception and require another repair attempt", async () => {
  const root = await realpath(await mkdtemp(join(tmpdir(), "vize-config-incomplete-")));
  try {
    const selected = join(root, "vite.config.mjs");
    await writeFile(selected, "export default {};");
    const unresolved = (exporter: string) => ({
      errors: [{ code: "UNRESOLVED_IMPORT", id: selected, exporter }],
    });
    for (const error of [
      new Error("authored factory failure"),
      { code: "ERR_MODULE_NOT_FOUND" },
      unresolved("policy"),
      unresolved("../policy"),
      unresolved("../hash#policy.mjs"),
      unresolved("../query?policy.mjs"),
      unresolved("../percent%25policy.mjs"),
      { errors: [{ text: "esbuild has no structured requested path" }] },
      { errors: [...unresolved("../policy.mjs").errors, ...unresolved("policy").errors] },
    ]) {
      let observation: { paths: string[]; complete: boolean } | undefined;
      await assert.rejects(
        importConfigModuleWithDependencies(
          selected,
          undefined,
          undefined,
          () => {
            throw error;
          },
          (paths, complete) => {
            observation = { paths, complete };
          },
        ),
        (actual) => actual === error,
      );
      assert.ok(observation);
      assert.ok(observation.paths.includes(selected));
      assert.equal(observation.complete, false);
    }
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("actual loader selection watches nearer installation and broken-present repair without reevaluating the factory", async () => {
  const root = await realpath(await mkdtemp(join(tmpdir(), "vize-loader-selection-")));
  try {
    const app = join(root, "app");
    await mkdir(app);
    const selected = join(app, "vite.config.mjs");
    const counter = join(root, "evaluations.txt");
    const source = `import { appendFileSync } from 'node:fs'; export default () => {
      appendFileSync(${JSON.stringify(counter)}, 'evaluation\\n');
      return { vue: { formatter: { printWidth: 91 } } };
    };`;
    await writeFile(selected, source);
    const invoke = () =>
      spawnSync(process.execPath, ["--input-type=module", "-e", embedded, selected], {
        cwd: app,
        encoding: "utf8",
      });
    const nearer = join(app, "node_modules/vite");
    const hoisted = join(root, "node_modules/vite");
    const fallback = invoke();
    assert.equal(fallback.status, 0, fallback.stderr);
    assert.ok(JSON.parse(fallback.stdout).watchDependencies.includes(nearer));
    assert.ok(JSON.parse(fallback.stdout).watchDependencies.includes(hoisted));
    await mkdir(join(root, "node_modules"));
    await symlink(vite, hoisted, "junction");
    const installed = invoke();
    assert.equal(installed.status, 0, installed.stderr);
    const loaded = JSON.parse(installed.stdout);
    assert.equal(loaded.config.formatter.printWidth, 91);
    assert.ok(loaded.watchDependencies.includes(nearer));
    assert.ok(loaded.watchDependencies.includes(join(hoisted, "package.json")));
    assert.equal(
      loaded.dependencies.includes(join(hoisted, "package.json")),
      false,
      "loader decisions are watch-only, not authored config imports",
    );
    await mkdir(nearer, { recursive: true });
    await writeFile(
      join(nearer, "package.json"),
      '{"name":"vite","type":"module","exports":"./missing.mjs"}',
    );
    const broken = invoke();
    assert.equal(broken.status, 1, broken.stderr);
    const rejected = JSON.parse(broken.stdout);
    assert.equal("config" in rejected, false);
    assert.ok(rejected.dependencies.includes(nearer));
    assert.ok(rejected.dependencies.includes(join(nearer, "package.json")));
    assert.equal(
      rejected.dependencies.includes(hoisted),
      false,
      "a present broken nearer Vite must not select the hoisted loader",
    );
    assert.equal(rejected.watchDependenciesComplete, false);
    await rm(nearer, { recursive: true });
    await symlink(vite, nearer, "junction");
    const repaired = invoke();
    assert.equal(repaired.status, 0, repaired.stderr);
    assert.equal(JSON.parse(repaired.stdout).config.formatter.printWidth, 91);
    assert.ok(JSON.parse(repaired.stdout).watchDependencies.includes(join(nearer, "package.json")));
    assert.equal(await readFile(selected, "utf8"), source);
    assert.equal(
      await readFile(counter, "utf8"),
      "evaluation\nevaluation\nevaluation\n",
      "each successful attempt evaluates once; broken-present selection never evaluates or falls through",
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("failed real Vite bundles retain repair targets without claiming their hidden intermediate graph", async () => {
  const root = await realpath(await mkdtemp(join(tmpdir(), "vize-partial-config-")));
  try {
    const app = join(root, "app");
    await mkdir(join(app, "node_modules"), { recursive: true });
    await symlink(vite, join(app, "node_modules/vite"), "junction");
    const selected = join(app, "vite.config.mjs");
    const bridge = join(root, "bridge.mjs");
    const leaf = join(root, "leaf.mjs");
    const missing = join(root, "missing.mjs");
    const counter = join(root, "factory.txt");
    await writeFile(
      selected,
      `import vue from '../bridge.mjs'; import { appendFileSync } from 'node:fs';
      export default () => { appendFileSync(${JSON.stringify(counter)}, 'evaluation\\n'); return { vue }; };`,
    );
    await writeFile(bridge, "export { default } from './leaf.mjs';");
    await writeFile(leaf, "export { default } from './missing.mjs';");
    const invoke = () =>
      spawnSync(process.execPath, ["--input-type=module", "-e", embedded, selected], {
        cwd: app,
        encoding: "utf8",
      });
    const failed = invoke();
    assert.equal(failed.status, 1, failed.stderr);
    const observed = JSON.parse(failed.stdout);
    assert.equal("config" in observed, false);
    assert.equal(observed.watchDependenciesComplete, false);
    assert.ok(observed.dependencies.includes(leaf));
    assert.ok(observed.dependencies.includes(missing));
    assert.equal(
      observed.dependencies.includes(bridge),
      false,
      "the real bundle diagnostic does not expose the intermediate bridge",
    );
    const authored = await readFile(selected);
    const leafBytes = await readFile(leaf);
    await writeFile(bridge, "export default { formatter: { printWidth: 93 } };");
    const repaired = invoke();
    assert.equal(repaired.status, 0, repaired.stderr);
    assert.equal(JSON.parse(repaired.stdout).config.formatter.printWidth, 93);
    assert.deepEqual(await readFile(selected), authored);
    assert.deepEqual(await readFile(leaf), leafBytes);
    await assert.rejects(readFile(missing), { code: "ENOENT" });
    assert.equal(
      await readFile(counter, "utf8"),
      "evaluation\n",
      "only the repaired attempt executes its original factory, exactly once",
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
