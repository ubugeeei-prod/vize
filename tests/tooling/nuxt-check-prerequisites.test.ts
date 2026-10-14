import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { test } from "node:test";

import { execNpxCommand } from "../_helpers/app-fixture-runtime.ts";
import { prepareNuxtCheckFixture } from "../_helpers/nuxt-check-prepare.ts";
import { execNuxtPrepareNpx, inspectPrepareNode } from "../_helpers/nuxt-check-runtime.ts";
import { prepareNuxtPrerequisites } from "../_helpers/nuxt-check-prerequisites.ts";

test("real npm preparation binds Node after an ancestor .bin overrides its initial PATH", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-npm-runtime-"));
  const cwd = path.join(root, "checked");
  const pkg = path.join(root, "probe-package");
  const output = path.join(root, "runtime.json");
  try {
    // The test runner may be pnpm's bare Node package. Use its real binary and
    // the environment's real npm CLI; this is a local distribution fixture.
    const distribution = path.join(root, "distribution");
    fs.mkdirSync(distribution);
    const executable = path.join(distribution, path.basename(fs.realpathSync(process.execPath)));
    fs.copyFileSync(fs.realpathSync(process.execPath), executable);
    const npxName = process.platform === "win32" ? "npx.cmd" : "npx";
    const pathKey = Object.keys(process.env).find((key) => /^path$/i.test(key)) ?? "PATH";
    const npxSource = [
      path.dirname(process.execPath),
      ...(process.env[pathKey] ?? "").split(path.delimiter),
    ]
      .map((directory) => path.join(directory, npxName))
      .find((candidate) => fs.existsSync(candidate));
    assert.ok(npxSource, "the runtime regression requires the actual npm CLI");
    fs.symlinkSync(fs.realpathSync(npxSource), path.join(distribution, npxName), "file");
    if (process.platform === "win32") {
      const npmCli = path.join(distribution, "node_modules/npm/bin/npx-cli.js");
      fs.mkdirSync(path.dirname(npmCli), { recursive: true });
      fs.symlinkSync(
        path.join(path.dirname(npxSource), "node_modules/npm/bin/npx-cli.js"),
        npmCli,
        "file",
      );
    }
    fs.mkdirSync(cwd);
    fs.mkdirSync(pkg);
    const ancestorBin = path.join(root, "node_modules/.bin");
    fs.mkdirSync(ancestorBin, { recursive: true });
    fs.writeFileSync(
      path.join(ancestorBin, process.platform === "win32" ? "node.cmd" : "node"),
      process.platform === "win32" ? "@exit /b 97\r\n" : "#!/bin/sh\nexit 97\n",
      { mode: 0o755 },
    );
    fs.writeFileSync(
      path.join(pkg, "package.json"),
      JSON.stringify({ name: "vize-runtime-probe", version: "1.0.0", bin: { probe: "probe.cjs" } }),
    );
    fs.writeFileSync(
      path.join(pkg, "probe.cjs"),
      `#!/usr/bin/env node
const fs = require('node:fs');
const { execFileSync } = require('node:child_process');
const expression = 'JSON.stringify({execPath:process.execPath,version:process.version})';
fs.writeFileSync(process.argv[2], JSON.stringify({
  parent: {execPath:process.execPath, version:process.version},
  child: JSON.parse(execFileSync('node', ['-p', expression], {encoding:'utf8'}))
}));
`,
      { mode: 0o755 },
    );
    const selected = inspectPrepareNode(executable, {
      ...process.env,
      npm_config_cache: path.join(root, "npm-cache"),
      npm_config_offline: "true",
      npm_config_ignore_scripts: "true",
      npm_config_update_notifier: "false",
    });
    const args = ["-y", `--package=file:${pkg}`, "probe", output];
    const { VIZE_APP_CHECK_PREPARE_NODE: _binding, ...unbound } = selected.env;
    assert.throws(() => execNpxCommand(args, { cwd, env: unbound, timeout: 60_000 }));
    assert.equal(fs.existsSync(output), false, "the competing Node must actually intercept npm");
    execNuxtPrepareNpx(args, { cwd, env: selected.env, timeout: 60_000 });
    const actual = JSON.parse(fs.readFileSync(output, "utf8"));
    for (const child of [actual.parent, actual.child]) {
      assert.equal(fs.realpathSync(child.execPath), selected.observation.executable);
      assert.equal(child.version, selected.observation.version);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("npmX's real generator order supplies Nuxt and retains outputs without authored rewrites", () => {
  for (const failGenerator of [false, true]) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-generator-order-"));
    const sourceDir = path.join(root, "tests/_fixtures/_git/npmx.dev");
    const captureDir = path.join(root, "evidence");
    const git = (cwd: string, args: string[]) =>
      execFileSync("git", args, { cwd, encoding: "utf8" });
    const commit = (cwd: string) => {
      git(cwd, ["add", "."]);
      git(cwd, [
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.com",
        "commit",
        "-qm",
        "fixture",
      ]);
    };
    try {
      fs.mkdirSync(sourceDir, { recursive: true });
      const manifest = JSON.stringify({
        dependencies: { nuxt: "4.5.2" },
        scripts: {
          postinstall: "vp run generate:lexicons && vp run generate:sprite && nuxt prepare",
        },
      });
      fs.writeFileSync(path.join(sourceDir, "package.json"), manifest);
      git(sourceDir, ["init", "-q"]);
      commit(sourceDir);
      const revision = git(sourceDir, ["rev-parse", "HEAD"]).trim();
      git(root, ["init", "-q"]);
      git(root, [
        "update-index",
        "--add",
        "--cacheinfo",
        `160000,${revision},tests/_fixtures/_git/npmx.dev`,
      ]);
      commit(root);
      const invocations: string[][] = [];
      const prepare = () =>
        prepareNuxtCheckFixture({
          name: "npmx.dev",
          sourceDir,
          authorityRoot: root,
          captureDir,
          run(args, options) {
            invocations.push(args);
            assert.equal(options.cwd, sourceDir);
            if (args.includes("install")) {
              fs.mkdirSync(path.join(sourceDir, "node_modules/nuxt"), { recursive: true });
              fs.writeFileSync(
                path.join(sourceDir, "node_modules/nuxt/package.json"),
                '{"name":"nuxt"}',
              );
            } else if (args.includes("generate:lexicons")) {
              assert.equal(options.timeout, 120_000);
              if (failGenerator) throw new Error("authentic lexicon generator failed");
              fs.mkdirSync(path.join(sourceDir, "shared/types/lexicons"), { recursive: true });
              fs.writeFileSync(
                path.join(sourceDir, "shared/types/lexicons/dev.ts"),
                "export const lexicon = {}\n",
              );
            } else if (args.includes("generate:sprite")) {
              for (const dir of ["public", "public-dev", "public-staging"]) {
                fs.mkdirSync(path.join(sourceDir, dir));
                fs.writeFileSync(path.join(sourceDir, dir, "file-tree-sprite.svg"), "<svg/>\n");
              }
            } else {
              assert.ok(fs.existsSync(path.join(sourceDir, "shared/types/lexicons/dev.ts")));
              assert.ok(fs.existsSync(path.join(sourceDir, "public/file-tree-sprite.svg")));
              fs.mkdirSync(path.join(sourceDir, ".nuxt"));
              fs.writeFileSync(path.join(sourceDir, ".nuxt/tsconfig.json"), "{}\n");
            }
          },
        });
      if (failGenerator) assert.throws(prepare, /authentic lexicon generator failed/);
      else assert.equal(prepare(), sourceDir);
      assert.deepEqual(
        invocations.slice(1),
        failGenerator
          ? [["-y", "pnpm@10", "exec", "vp", "run", "--no-cache", "generate:lexicons"]]
          : [
              ["-y", "pnpm@10", "exec", "vp", "run", "--no-cache", "generate:lexicons"],
              ["-y", "pnpm@10", "exec", "vp", "run", "--no-cache", "generate:sprite"],
              ["-y", "pnpm@10", "exec", "nuxt", "prepare"],
            ],
      );
      const receipt = JSON.parse(
        fs.readFileSync(path.join(captureDir, "npmx.dev/preparation/receipt.json"), "utf8"),
      );
      assert.equal(receipt.failed, failGenerator);
      assert.equal(receipt.authoredPreserved, true);
      assert.deepEqual(receipt.sourceMutations, []);
      assert.equal(fs.readFileSync(path.join(sourceDir, "package.json"), "utf8"), manifest);
      if (!failGenerator) {
        const outputs = receipt.files.filter(
          (file: { role: string }) => file.role === "generated-provider-prerequisite",
        );
        assert.equal(outputs.length, 4);
        for (const output of outputs) {
          assert.deepEqual(
            fs.readFileSync(path.join(captureDir, "npmx.dev/preparation", output.retainedBytes)),
            fs.readFileSync(output.path),
          );
        }
      }
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  }
});

test("npmX prerequisites refuse removed authored hooks and never apply to another project", () => {
  let runs = 0;
  prepareNuxtPrerequisites("other", undefined, () => {
    runs++;
  });
  assert.equal(runs, 0);
  for (const postinstall of [
    undefined,
    "nuxt prepare",
    "vp run generate:lexicons && nuxt prepare",
  ]) {
    assert.throws(
      () =>
        prepareNuxtPrerequisites("npmx.dev", postinstall, () => {
          runs++;
        }),
      /authored postinstall|no longer authors/,
    );
  }
  assert.equal(runs, 0);
});
