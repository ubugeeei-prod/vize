import { runInNewContext } from "node:vm";
import { readRepoFile } from "../support/github-workflows.ts";
import {
  MALFORMED_CONFIG_DEPENDENCIES,
  REFERENCE_DIAGNOSTICS,
  REFERENCE_FORMATTED,
  REFERENCE_POLICY,
  REFERENCE_SOURCE,
  tsconfigBase,
  vueHelper,
  BUILD_FILE_OUTPUTS,
  assertBuildFileOutput,
} from "../../../tools/support/compat/npm/smoke-release-config-fixture.ts";
import { runConfigDependencyCell } from "../../../tools/support/compat/npm/smoke-release-config-lsp.ts";
import assert from "node:assert/strict";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  runConfigHarness,
  verifyInstalledImages,
} from "../../../tools/support/compat/npm/smoke-release-config-evidence.ts";
import { installPackedWorkspaceDependencies } from "../../../tools/support/compat/npm/smoke-release-init-workspace.ts";
import { PACKAGE_MANAGERS } from "../../../tools/support/compat/npm/smoke-release-init-managers.mjs";
import { packedFileSpec } from "../../../tools/support/compat/npm/smoke-release-init-fresh.mjs";
import { runManager } from "../../../tools/support/compat/npm/smoke-release-init-project.mjs";

const expected = "a".repeat(64);
const fixture = () =>
  ["app", "control"].map((name) => ({
    project: `/workspace/packages/${name}`,
    workspace: "/workspace",
    native: {
      binaries: [{ path: "/workspace/node_modules/native/provider.node", sha256: expected }],
    },
    corsa: {
      name: "@typescript/typescript-linux-x64",
      manifest: {
        path: "/workspace/node_modules/corsa/package.json",
        sha256: expected,
        contents: "{}",
      },
    },
  }));

void test("config harness retains whole large stdout/stderr on success and nonzero exit", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-transport-"));
  try {
    const cwd = path.join(directory, "project with spaces");
    fs.mkdirSync(cwd);
    const packets = ["stdout", "stderr"].map((name, index) => {
      const bytes = Buffer.concat([
        Buffer.alloc(2 * 1024 * 1024 + 19, 0x61 + index),
        Buffer.from("\0λ\r\ncomplete tail\n"),
      ]);
      const file = path.join(directory, `${name} source.bin`);
      fs.writeFileSync(file, bytes);
      return { file, bytes };
    });
    const program = `import fs from "node:fs";
fs.writeFileSync(1, fs.readFileSync(process.argv[1]));
fs.writeFileSync(2, fs.readFileSync(process.argv[2]));
process.exitCode = Number(process.argv[3]);`;
    const env = { ...process.env, VIZE_SMOKE_LSP_LOG_DIR: directory };
    const args = ["--input-type=module", "-e", program, ...packets.map(({ file }) => file), "0"];
    const previous = spawnSync(process.execPath, args, { cwd, env });
    assert.equal(previous.error?.code, "ENOBUFS", "the old pipe transport loses this packet");
    for (const status of [0, 17]) {
      args[args.length - 1] = String(status);
      const run = () => runConfigHarness(process.execPath, args, { cwd, env, directory });
      if (status === 0) run();
      else assert.throws(run, /config harness failed/u);
      const receipts = fs
        .readdirSync(directory)
        .filter((name) => name.startsWith("config-harness-"));
      assert.equal(receipts.length, status === 0 ? 1 : 2, "invocations have independent custody");
      const receipt = receipts
        .map((name) =>
          JSON.parse(fs.readFileSync(path.join(directory, name, "process.json"), "utf8")),
        )
        .find((record) => record.status === status);
      assert.ok(receipt);
      assert.equal(receipt.executable, process.execPath);
      assert.deepEqual(receipt.args, args);
      assert.equal(receipt.cwd, cwd);
      assert.equal(receipt.environment.VIZE_SMOKE_LSP_LOG_DIR, directory);
      assert.equal(receipt.environment.PATH, env.PATH ?? null);
      assert.equal(receipt.status, status);
      assert.equal(receipt.signal, null);
      assert.equal(receipt.error, null);
      assert.equal(receipt.timeout, null);
      assert.equal(receipt.stdin, "pipe");
      for (const [index, name] of ["stdout", "stderr"].entries()) {
        const expected = packets[index].bytes;
        assert.deepEqual(fs.readFileSync(receipt[name].path), expected);
        assert.equal(receipt[name].bytes, expected.length);
        assert.equal(receipt[name].sha256, createHash("sha256").update(expected).digest("hex"));
      }
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("config harness records spawn failure and closes both output descriptors", (context) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-spawn-error-"));
  const close = fs.closeSync;
  const descriptors: number[] = [];
  const observe = context.mock.method(fs, "closeSync", (descriptor: number) => {
    close(descriptor);
    assert.throws(() => fs.fstatSync(descriptor), { code: "EBADF" });
    descriptors.push(descriptor);
  });
  try {
    const executable = path.join(directory, "absent executable");
    assert.throws(
      () => runConfigHarness(executable, ["exact", "argv"], { cwd: directory, env: {}, directory }),
      { code: "ENOENT" },
    );
    assert.ok(descriptors.length >= 2);
    assert.notEqual(descriptors[0], descriptors[1], "both independently opened outputs close");
    const [name] = fs.readdirSync(directory);
    const receipt = JSON.parse(fs.readFileSync(path.join(directory, name, "process.json"), "utf8"));
    assert.equal(receipt.executable, executable);
    assert.deepEqual(receipt.args, ["exact", "argv"]);
    assert.equal(receipt.status, null);
    assert.equal(receipt.signal, null);
    assert.equal(receipt.error.code, "ENOENT");
    for (const stream of [receipt.stdout, receipt.stderr]) {
      assert.equal(stream.bytes, 0);
      assert.deepEqual(fs.readFileSync(stream.path), Buffer.alloc(0));
      assert.equal(stream.sha256, createHash("sha256").digest("hex"));
    }
  } finally {
    observe.mock.restore();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("compiler file counterparts retain the complete snapshots with exactly one framing LF", () => {
  for (const [kind, snapshotName] of [
    ["configured", "build_vite"],
    ["defaults", "build_defaults"],
  ] as const) {
    const snapshot = readRepoFile(
      `crates/vize/tests/snapshots/project_config_cli__project_settings_${snapshotName}.snap`,
    ).replace(/^---\n[\s\S]*?\n---\n/u, "");
    const file = BUILD_FILE_OUTPUTS[kind];
    assert.equal(snapshot, `${file}\n`);
    assert.equal(Buffer.byteLength(snapshot) - Buffer.byteLength(file), 1);
    assert.equal(file.endsWith("}"), true);
    const configured = kind === "configured";
    assertBuildFileOutput(file, configured);
    const changedBody = file.replace('"p"', '"span"');
    assert.notEqual(changedBody, file);
    for (const invalid of [`${file}\n`, file.slice(0, -1), changedBody]) {
      assert.throws(
        () => assertBuildFileOutput(invalid, configured),
        /complete configured compiler output/u,
      );
    }
  }
});

void test("real pinned pnpm updates a nonempty authored lock and links both edited siblings in CI", () => {
  // This is an offline package-linker control, not a native Vize runtime claim.
  const directory = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-workspace-link-law-")),
  );
  const manager = PACKAGE_MANAGERS.pnpm;
  const isolatedVariables = [
    "CI",
    "XDG_CONFIG_HOME",
    "npm_config_userconfig",
    "NPM_CONFIG_USERCONFIG",
  ];
  const previousEnvironment = new Map(isolatedVariables.map((name) => [name, process.env[name]]));
  try {
    process.env.CI = "true";
    process.env.XDG_CONFIG_HOME = path.join(directory, "config");
    fs.mkdirSync(process.env.XDG_CONFIG_HOME);
    const userconfig = path.join(directory, "empty-user.npmrc");
    fs.writeFileSync(userconfig, "");
    process.env.npm_config_userconfig = userconfig;
    process.env.NPM_CONFIG_USERCONFIG = userconfig;
    assert.equal(
      runManager(manager, ["--version"], { cwd: directory }).trim(),
      manager.corepackSpec.split("@")[1],
    );
    const workspace = path.join(directory, "pnpm workspace");
    const packs = path.join(directory, "packs");
    fs.mkdirSync(packs);
    fs.mkdirSync(workspace);
    fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true}\n');
    fs.writeFileSync(
      path.join(workspace, "pnpm-workspace.yaml"),
      'packages:\n  - "packages/*"\noffline: true\nignoreScripts: true\n',
    );
    assert.equal(
      runManager(manager, ["config", "get", "offline"], { cwd: workspace }).trim(),
      "true",
    );
    const dependencies = ["vize", "@vizejs/vite-plugin"];
    const peer = "vize-fixture-authored-peer";
    const packed = new Map<string, string>();
    for (const [index, name] of [peer, ...dependencies].entries()) {
      const provider = path.join(directory, `provider-${index}`);
      fs.mkdirSync(provider);
      fs.writeFileSync(
        path.join(provider, "package.json"),
        JSON.stringify({ name, version: "1.2.3", files: ["identity.txt"] }) + "\n",
      );
      fs.writeFileSync(path.join(provider, "identity.txt"), `${name}: independent local packet\n`);
      runManager(manager, ["pack", "--pack-destination", packs], { cwd: provider });
      packed.set(name, path.join(packs, `${name.replace(/^@/u, "").replace("/", "-")}-1.2.3.tgz`));
    }
    const manifests = new Map<string, Record<string, unknown>>();
    for (const name of ["app", "control"]) {
      const project = path.join(workspace, "packages", name);
      fs.mkdirSync(project, { recursive: true });
      const manifest = {
        name,
        private: true,
        description: `${name} sentinel`,
        devDependencies: { [peer]: packedFileSpec(packed.get(peer)!) },
      };
      manifests.set(name, manifest);
      fs.writeFileSync(path.join(project, "package.json"), JSON.stringify(manifest) + "\n");
    }
    runManager(manager, manager.bootstrapArgs, { cwd: workspace });
    const bootstrapLock = fs.readFileSync(path.join(workspace, manager.lockfile));
    assert.ok(
      bootstrapLock.includes(Buffer.from(peer)),
      "bootstrap lock retains its authored peer",
    );
    for (const name of ["app", "control"]) {
      const installedPeer = fs.realpathSync(
        path.join(workspace, "packages", name, "node_modules", peer),
      );
      assert.equal(
        fs.readFileSync(path.join(installedPeer, "identity.txt"), "utf8"),
        `${peer}: independent local packet\n`,
      );
    }
    const control = path.join(workspace, "packages/control");
    const controlManifest = {
      ...manifests.get("control"),
      devDependencies: {
        [peer]: packedFileSpec(packed.get(peer)!),
        ...Object.fromEntries(
          dependencies.map((name) => [name, packedFileSpec(packed.get(name)!)]),
        ),
      },
    };
    fs.writeFileSync(path.join(control, "package.json"), JSON.stringify(controlManifest) + "\n");
    runManager(
      manager,
      [
        ...manager.installArgs,
        ...dependencies.map((name) => `${name}@${packedFileSpec(packed.get(name)!)}`),
        ...manager.installFlags,
      ],
      { cwd: path.join(workspace, "packages/app") },
    );
    assert.throws(
      () => fs.realpathSync(path.join(control, "node_modules/vize")),
      { code: "ENOENT" },
      "the previous app-only add leaves the edited control manifest unlinked",
    );

    // Restore the pre-install graph so the repaired single root install cannot
    // borrow either lockfile updates or links from the old app-only command.
    fs.writeFileSync(path.join(workspace, manager.lockfile), bootstrapLock);
    for (const directory of [
      workspace,
      ...["app", "control"].map((name) => path.join(workspace, "packages", name)),
    ]) {
      fs.rmSync(path.join(directory, "node_modules"), { recursive: true, force: true });
    }
    // Make the runner's CI default explicit, independent of host pnpm config.
    fs.appendFileSync(path.join(workspace, "pnpm-workspace.yaml"), "frozenLockfile: true\n");
    assert.throws(
      () => runManager(manager, manager.bootstrapArgs, { cwd: workspace }),
      /ERR_PNPM_OUTDATED_LOCKFILE/u,
      "a changed dependency plan cannot reuse the original frozen bootstrap lock",
    );
    assert.deepEqual(fs.readFileSync(path.join(workspace, manager.lockfile)), bootstrapLock);
    installPackedWorkspaceDependencies(workspace, dependencies, packed);
    for (const name of ["app", "control"]) {
      const project = path.join(workspace, "packages", name);
      assert.deepEqual(JSON.parse(fs.readFileSync(path.join(project, "package.json"), "utf8")), {
        ...manifests.get(name),
        devDependencies: controlManifest.devDependencies,
      });
      for (const dependency of [peer, ...dependencies]) {
        const installed = fs.realpathSync(path.join(project, "node_modules", dependency));
        const relative = path.relative(workspace, installed);
        assert.ok(relative !== "" && !path.isAbsolute(relative) && !relative.startsWith(".."));
        assert.equal(
          fs.readFileSync(path.join(installed, "identity.txt"), "utf8"),
          `${dependency}: independent local packet\n`,
        );
        assert.equal(
          JSON.parse(fs.readFileSync(path.join(installed, "package.json"), "utf8")).version,
          "1.2.3",
        );
      }
    }
  } finally {
    for (const [name, value] of previousEnvironment) {
      if (value === undefined) delete process.env[name];
      else process.env[name] = value;
    }
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("both independent installed siblings use the source-built native image", () => {
  verifyInstalledImages(fixture(), expected);
});

void test("an installed sibling with a stale image or outside fallback is rejected", () => {
  const stale = fixture();
  stale[1].native.binaries[0].sha256 = "b".repeat(64);
  assert.throws(() => verifyInstalledImages(stale, expected), /installed native image/u);
  const outside = fixture();
  outside[1].native.binaries[0].path = "/global/provider.node";
  assert.throws(() => verifyInstalledImages(outside, expected));
});

void test("a missing sibling, duplicate sibling, or multiple loaded images cannot pass", () => {
  const images = fixture();
  assert.throws(() => verifyInstalledImages(images.slice(0, 1), expected));
  assert.throws(() => verifyInstalledImages([images[0], images[0]], expected));
  images[1].native.binaries.push({ ...images[1].native.binaries[0] });
  assert.throws(() => verifyInstalledImages(images, expected), /one loaded native provider/u);
});

void test("a partial failed campaign saves its failure receipt and remains nonzero", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-evidence-"));
  try {
    const script = fileURLToPath(
      new URL(
        "../../../tools/support/compat/npm/smoke-release-config-evidence.ts",
        import.meta.url,
      ),
    );
    const child = spawnSync(process.execPath, [script, "collect"], {
      env: { ...process.env, PACKED_CONFIG_OUTPUT: directory },
      encoding: "utf8",
      timeout: 10_000,
    });
    assert.equal(child.error, undefined);
    assert.equal(child.signal, null);
    assert.notEqual(child.status, 0);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(directory, "installed-images.json"), "utf8"),
    );
    assert.equal(receipt.passed, false);
    assert.match(receipt.failure, /source\.json/u);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

await test("malformed dependency controls are syntax-invalid and differ from valid settings", async () => {
  await assert.rejects(
    import(
      `data:text/javascript,${encodeURIComponent(MALFORMED_CONFIG_DEPENDENCIES["vue-settings.ts"])}`
    ),
    SyntaxError,
  );
  assert.throws(() => JSON.parse(MALFORMED_CONFIG_DEPENDENCIES["tsconfig.base.json"]), SyntaxError);
  const valid = await import(`data:text/javascript,${encodeURIComponent(vueHelper(true))}`);
  assert.equal(valid.default.formatter.singleQuote, true);
  assert.equal(JSON.parse(tsconfigBase(true)).compilerOptions.strictNullChecks, true);
});

void test("installed reference controls retain the whole existing Vim authored oracle", () => {
  const vim = readRepoFile("editors/vim/test/config_project_spec.vim");
  function literal(name: string): unknown {
    const prefix = `let s:${name} = `;
    const start = vim.indexOf(prefix);
    assert.ok(start >= 0, `missing authored Vim ${name}`);
    const rest = vim.slice(start + prefix.length);
    const source =
      name === "modern_diagnostics"
        ? rest.slice(0, rest.indexOf("\ncall ")).replace(/^\s*\\\s*/gmu, "")
        : rest.split("\n")[0]!;
    // These owned Vim dictionary/list/string literals use JavaScript-compatible
    // syntax. Parse the complete authored value, without running an editor/LSP.
    return JSON.parse(JSON.stringify(runInNewContext(`(${source})`, {}, { timeout: 1_000 })));
  }
  assert.equal(REFERENCE_SOURCE, `${(literal("source") as string[]).join("\n")}\n`);
  assert.equal(REFERENCE_POLICY, literal("valid_policy"));
  assert.deepEqual(REFERENCE_DIAGNOSTICS, literal("modern_diagnostics"));
  assert.deepEqual(REFERENCE_FORMATTED, literal("formatted"));
});

await test("dependency restoration preserves the first scenario failure and records cleanup errors", async () => {
  for (const [scenarioFails, restoreFails] of [
    [true, true],
    [true, false],
    [false, true],
    [false, false],
  ]) {
    const first = new Error("whole packet scenario assertion");
    const cleanup = new Error("restore publication failed");
    let restores = 0;
    const records: { name: string; result: unknown }[] = [];
    const run = runConfigDependencyCell({
      dependency: "vue-settings.ts",
      failure: "malformed",
      run: async () => {
        if (scenarioFails) throw first;
      },
      restore: async () => {
        restores++;
        if (restoreFails) throw cleanup;
      },
      record: (name, result) => records.push({ name, result }),
    });
    if (scenarioFails || restoreFails)
      await assert.rejects(run, (error: unknown) => error === (scenarioFails ? first : cleanup));
    else await run;
    assert.equal(restores, 1, "restoration is attempted even after the scenario fails");
    assert.deepEqual(
      records,
      restoreFails
        ? [
            {
              name: "configuration dependency restoration failure",
              result: {
                dependency: "vue-settings.ts",
                failure: "malformed",
                error: { name: cleanup.name, message: cleanup.message, stack: cleanup.stack },
              },
            },
          ]
        : [],
    );
  }
});
