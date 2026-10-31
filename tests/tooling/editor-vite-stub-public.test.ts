import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, toNamespacedPath } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { Recorder, sha256, success } from "../../tools/support/config/windows_consumer/common.ts";
import {
  inventory,
  writeFixture,
} from "../../tools/support/release/editor_vite_stub/filesystem.ts";
import { injectLink } from "../../tools/support/release/editor_vite_stub/editor.ts";
import { verifyJournal } from "../../tools/support/release/editor_vite_stub/custody.ts";
import {
  bootstrapPnpm,
  resolvePnpm,
} from "../../tools/support/release/editor_vite_stub/install.ts";
import {
  auditPnpmState,
  capturePnpmState,
  joinPnpmRounds,
  onlyPnpmStateChanged,
  validateState,
} from "../../tools/support/release/editor_vite_stub/pnpm-state.ts";
import {
  bindLocalNpx,
  npxCacheSnapshot,
} from "../../tools/support/release/editor_vite_stub/config.ts";
import {
  auditConfigCache,
  cacheSnapshot,
  validateCacheBytes,
} from "../../tools/support/release/editor_vite_stub/cold-cache.ts";

function fixture(run: (root: string) => void): void {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "editor-custody-control-")));
  try {
    run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
test("public default logging rejects every explicit override before installs or host admission", () => {
  const entry = fileURLToPath(
    new URL("../../tools/support/release/editor_vite_stub/run.ts", import.meta.url),
  );
  for (const value of ["", "relative.log", join(tmpdir(), "explicit-lsp.log")]) {
    const packet = spawnSync(process.execPath, [entry], {
      env: { ...process.env, VIZE_LSP_LOG_FILE: value },
      encoding: "utf8",
    });
    assert.equal(packet.status, 1);
    assert.equal(packet.signal, null);
    assert.equal(packet.error, undefined);
    assert.match(
      packet.stderr,
      /public default logging requires VIZE_LSP_LOG_FILE to remain unset/,
    );
  }
});
test("full installation inventory detects same-size byte mutation and link-only retargeting", () =>
  fixture((root) => {
    mkdirSync(join(root, "pkg"));
    mkdirSync(join(root, "other"));
    writeFileSync(join(root, "pkg/index.js"), "export const value = 7;\n");
    writeFileSync(join(root, "other/index.js"), "export const value = 7;\n");
    const link = join(root, "alias");
    const kind = process.platform === "win32" ? "junction" : "dir";
    symlinkSync(join(root, "pkg"), link, kind);
    const before = inventory(root);
    writeFileSync(join(root, "pkg/index.js"), "export const value = 8;\n");
    assert.throws(() => assert.deepEqual(inventory(root), before));
    writeFileSync(join(root, "pkg/index.js"), "export const value = 7;\n");
    assert.deepEqual(inventory(root), before);
    rmSync(link);
    symlinkSync(join(root, "other"), link, kind);
    assert.throws(() => assert.deepEqual(inventory(root), before));
  }));
test("stale whole-link injection uses an actual Windows junction and preserves the external tree", () =>
  fixture((root) => {
    const temporary = join(root, "private"),
      mirror = join(temporary, "mirror"),
      modules = join(root, "external/node_modules");
    mkdirSync(join(mirror, "node_modules"), { recursive: true });
    mkdirSync(modules, { recursive: true });
    writeFileSync(join(modules, "sentinel"), "external byte custody\n");
    const before = inventory(modules);
    const injected = injectLink(mirror, temporary, modules);
    assert.equal(injected.kind, process.platform === "win32" ? "junction" : "dir");
    assert.equal(injected.resolved, modules);
    assert.deepEqual(inventory(modules), before);
    // This is a falsification control for the observer, not an imitation of the product repair.
    writeFileSync(join(mirror, "node_modules/sentinel"), "unintended write\n");
    assert.throws(() => assert.deepEqual(inventory(modules), before));
    assert.throws(() => injectLink(mirror, temporary, modules), /only an owned mirror directory/);
    assert.throws(() => injectLink(join(root, "external"), temporary, modules));
  }));
test("reported fluent config, app, declaration stub and installation controls remain immutable", () =>
  fixture((root) => {
    const directory = new URL(
      "../_fixtures/differential/typechecker/editor-vite-stub-11145/",
      import.meta.url,
    );
    const project = readFileSync(new URL("project.json", directory));
    const installation = readFileSync(new URL("installation.json", directory));
    assert.equal(
      sha256(project),
      "8e13ee81f8a0086e34e44f2ec49f7dc6ecde19d55694a75bf3415b212d120f5a",
    );
    assert.equal(
      sha256(installation),
      "cf67d99370a6339ed1144f22b9fa84f71a15ef7a67a809a7a4fa72462dc83daf",
    );
    writeFixture(root, { "src/app.vue": "original\n" }, () => true);
    assert.equal(readFileSync(join(root, "src/app.vue"), "utf8"), "original\n");
    assert.throws(() => writeFixture(root, { "../escape": "bad" }, () => true), /fixture escaped/);
  }));

test("only an early negative JS API phase may finish without loading the native normalizer", () =>
  fixture((root) => {
    const journal = join(root, "observer.ndjson");
    const identity = { binary: { path: join(root, "actual.node"), sha256: "a".repeat(64) } };
    const initial = {
      event: "initialized",
      nativePath: identity.binary.path,
      sha256: identity.binary.sha256,
    };
    const ended = { event: "exit", code: 1 };
    const save = (events: Record<string, unknown>[]) =>
      writeFileSync(
        journal,
        events
          .map((event) =>
            JSON.stringify({ schema: "vize-public-native-custody-event-v1", pid: 11145, ...event }),
          )
          .join("\n") + "\n",
      );
    save([initial, ended]);
    verifyJournal(journal, identity, 1, false);
    assert.throws(() => verifyJournal(journal, identity, 1));
    save([initial, { event: "failed" }, ended]);
    assert.throws(() => verifyJournal(journal, identity, 1, false));
    save([
      initial,
      { event: "attempt" },
      {
        event: "returned",
        expectedNative: true,
        actualPath: identity.binary.path,
        sha256: "b".repeat(64),
      },
      ended,
    ]);
    assert.throws(() => verifyJournal(journal, identity, 1, false));
    const attempt = {
      event: "attempt",
      expectedNative: true,
      actualPath: identity.binary.path,
      sha256: identity.binary.sha256,
    };
    const returned = { ...attempt, event: "returned" };
    save([initial, attempt, returned]);
    verifyJournal(journal, identity, null);
    assert.throws(() => verifyJournal(journal, identity, 2));
    save([initial, attempt, returned, { event: "exit", code: 2 }]);
    assert.throws(() => verifyJournal(journal, identity, null));
  }));

test("pnpm uses only its pinned manifest entry and rejects missing, wrong-version and escaped files", () =>
  fixture((root) => {
    const pkg = join(root, "node_modules/pnpm");
    mkdirSync(join(pkg, "bin"), { recursive: true });
    const save = (bin: string, version = "12.1.0") =>
      writeFileSync(
        join(pkg, "package.json"),
        JSON.stringify({ name: "pnpm", version, bin: { pnpm: bin } }),
      );
    assert.throws(() => resolvePnpm(root));
    for (const name of ["pnpm", "pnpm.exe"]) {
      writeFileSync(
        join(pkg, "bin", name),
        'throw new Error("control must never execute a fixture");\n',
      );
      save("bin/" + name);
      const selected = resolvePnpm(root);
      assert.deepEqual(selected.argv, [realpathSync(join(pkg, "bin", name))]);
      assert.equal(selected.package.entry.sha256, sha256(readFileSync(join(pkg, "bin", name))));
    }
    save("bin/missing.mjs");
    assert.throws(() => resolvePnpm(root));
    save("bin/pnpm", "12.0.0");
    assert.throws(() => resolvePnpm(root));
    save("../../../escape.mjs");
    assert.throws(() => resolvePnpm(root), /declared pnpm entry escaped/);
    const outside = join(root, "outside");
    mkdirSync(outside);
    writeFileSync(join(outside, "entry.mjs"), "throw 0;\n");
    symlinkSync(outside, join(pkg, "redirect"), process.platform === "win32" ? "junction" : "dir");
    save("redirect/entry.mjs");
    assert.throws(() => resolvePnpm(root), /owned regular file/);
  }));

test(
  "actual fresh pnpm tooling bootstrap follows its published manifest on this OS",
  { skip: process.env.VIZE_EDITOR_PNPM_BOOTSTRAP !== "1" },
  () => {
    assert.equal(process.env.GITHUB_ACTIONS, "true", "real tooling installs are Actions-only");
    assert.equal(process.version, "v26.6.0");
    assert.ok(process.env.RUNNER_TEMP);
    const output = join(realpathSync(process.env.RUNNER_TEMP), "editor-pnpm-bootstrap");
    mkdirSync(output);
    const record = new Recorder(join(output, "proof"));
    const argv = bootstrapPnpm(join(output, "package-manager"), record);
    assert.equal(argv.length, 1, "execute the package-declared native entry directly");
    const npx = bindLocalNpx(
      join(output, "package-manager"),
      { packageName: "pnpm", bin: "pnpm", version: "12.1.0", entry: argv[0] },
      record,
    );
    const local = record.run(
      "bundled-npx-local-tool",
      [...npx.command, "pnpm", "--version"],
      join(output, "package-manager"),
      npx.env,
    );
    success(local);
    assert.equal(local.stdout.trim(), "12.1.0");
    record.json("tooling-npx-cache.json", npxCacheSnapshot(npx.cache));
    const project = join(output, "inert-project"),
      dependency = join(output, "local-dependency");
    mkdirSync(project);
    mkdirSync(dependency);
    writeFileSync(
      join(project, "package.json"),
      '{"name":"inert-project","private":true,"type":"module","devDependencies":{"inert-local-dependency":"file:../local-dependency"}}\n',
    );
    writeFileSync(
      join(dependency, "package.json"),
      '{"name":"inert-local-dependency","version":"1.0.0"}\n',
    );
    success(
      record.run("inert-local-install", [...argv, "install", "--no-frozen-lockfile"], project),
    );
    auditPnpmState(realpathSync(project), argv, record);
  },
);

test("state transition proof protects every row except the named pre-child PNPM state", () => {
  const row = {
    path: ".pnpm-workspace-state-v1.json",
    kind: "file" as const,
    mode: 33152,
    bytes: 1,
    sha256: "a",
  };
  const payload = { ...row, path: "vite/package.json" };
  onlyPnpmStateChanged([row, payload], [{ ...row, bytes: 2, sha256: "b" }, payload]);
  assert.throws(() => onlyPnpmStateChanged([row, payload], [row, { ...payload, sha256: "b" }]));
  assert.throws(() => onlyPnpmStateChanged([row, payload], [row]));
  assert.throws(() => onlyPnpmStateChanged([row, payload], [{ ...row, kind: "link" }, payload]));
});

test("failed metadata validation keeps raw evidence and an intervening payload edit fails the round join", () =>
  fixture((root) => {
    const modules = join(root, "node_modules");
    mkdirSync(modules);
    writeFileSync(join(root, "package.json"), "{}\n");
    const state = join(modules, ".pnpm-workspace-state-v1.json");
    writeFileSync(state, '{"invalid":true}\n');
    const record = new Recorder(join(root, "proof"));
    const invalid = capturePnpmState(root, record, "invalid");
    assert.throws(() => validateState(invalid));
    assert.ok(
      readFileSync(join(record.root, "invalid.snapshot.json"), "utf8").includes(
        invalid.state.rawBase64,
      ),
    );
    writeFileSync(
      state,
      JSON.stringify({
        lastValidatedTimestamp: 1,
        projects: {},
        settings: {},
        pnpmfiles: [],
        filteredInstall: false,
      }),
    );
    writeFileSync(join(modules, "payload"), "first\n");
    const first = capturePnpmState(root, record, "first-after");
    validateState(first);
    writeFileSync(join(modules, "payload"), "other\n");
    const second = capturePnpmState(root, record, "second-before");
    assert.throws(() => joinPnpmRounds(first, second), /snapshots must join/);
  }));

test("declared cache transitions retain full bytes and reject package edits, foreign roots and formats", () =>
  fixture((root) => {
    const modules = join(root, "node_modules");
    mkdirSync(modules);
    writeFileSync(join(modules, "payload"), "original dependency bytes\n");
    const record = new Recorder(join(root, "proof")),
      version = "0.610.0";
    const initial = cacheSnapshot(root, version);
    const key = {
      format: 1,
      version,
      runtime: 1,
      config: join(root, "vite.config.ts"),
      explicit: false,
    };
    const value = {
      key,
      env: [],
      env_names: null,
      inputs: [
        [key.config, false, { kind: 1, len: 0, modified: 1, changed: 1, inode: 1, link: null }],
      ],
      evaluation: { config: {}, complete: true, watchDependencies: [], dependencies: [] },
    };
    if (process.platform === "win32") {
      const config = toNamespacedPath(key.config);
      assert.notEqual(config, key.config, "exercise the actual native namespace spelling");
      const native = {
        ...value,
        key: { ...key, config },
        inputs: [[config, ...value.inputs[0].slice(1)]],
      };
      validateCacheBytes(Buffer.from(JSON.stringify([native])), root, version);
      assert.throws(() =>
        validateCacheBytes(
          Buffer.from(
            JSON.stringify([
              { ...native, key: { ...key, config: toNamespacedPath(join(root, "foreign.ts")) } },
            ]),
          ),
          root,
          version,
        ),
      );
    }
    const directory = join(modules, ".vize/config-cache");
    mkdirSync(directory, { recursive: true });
    const cache = join(directory, "1234567890abcdef.json");
    writeFileSync(cache, JSON.stringify([value]));
    const cold = cacheSnapshot(root, version);
    auditConfigCache(root, initial, cold, record, "cold");
    writeFileSync(
      cache,
      JSON.stringify([{ ...value, env: [["PATH", { Value: "a".repeat(64) }]] }]),
    );
    const changed = cacheSnapshot(root, version);
    auditConfigCache(root, cold, changed, record, "updated");
    assert.notEqual(
      cold.files[".vize/config-cache/1234567890abcdef.json"].rawBase64,
      changed.files[".vize/config-cache/1234567890abcdef.json"].rawBase64,
    );
    writeFileSync(join(modules, "payload"), "wrong dependency bytes\n");
    assert.throws(() =>
      auditConfigCache(root, changed, cacheSnapshot(root, version), record, "corrupt"),
    );
    writeFileSync(join(modules, "payload"), "original dependency bytes\n");
    assert.throws(() =>
      validateCacheBytes(
        Buffer.from(
          JSON.stringify([{ ...value, key: { ...key, config: join(root, "foreign.ts") } }]),
        ),
        root,
        version,
      ),
    );
    for (const runtime of [-1, 0.5])
      assert.throws(() =>
        validateCacheBytes(
          Buffer.from(JSON.stringify([{ ...value, key: { ...key, runtime } }])),
          root,
          version,
        ),
      );
    for (const stamp of [{ kind: 4 }, { modified: 0.5 }])
      assert.throws(() =>
        validateCacheBytes(
          Buffer.from(
            JSON.stringify([
              {
                ...value,
                inputs: [
                  [
                    key.config,
                    false,
                    { kind: 1, len: 0, modified: 1, changed: 1, inode: 1, link: null, ...stamp },
                  ],
                ],
              },
            ]),
          ),
          root,
          version,
        ),
      );
    for (const field of ["runtime", "len", "modified", "changed", "inode", "kind"]) {
      const token = '"' + field + '":' + (field === "len" ? "0" : "1");
      const raw = (number: string) =>
        Buffer.from(JSON.stringify([value]).replace(token, '"' + field + '":' + number));
      for (const invalid of ["18446744073709551616", "1e100"])
        assert.throws(() => validateCacheBytes(raw(invalid), root, version));
      if (field !== "kind") validateCacheBytes(raw("18446744073709551615"), root, version);
    }
    const duplicate = join(directory, "fedcba0987654321.json");
    writeFileSync(duplicate, JSON.stringify([value]));
    assert.throws(() => cacheSnapshot(root, version), /one cache filename/);
    rmSync(duplicate);
    writeFileSync(
      cache,
      JSON.stringify([{ ...value, key: { ...key, version: "invalid-version" } }]),
    );
    assert.throws(() => cacheSnapshot(root, version, record, "invalid-cache"));
    assert.ok(
      readFileSync(join(record.root, "invalid-cache.cache-bytes.json"), "utf8").includes(
        Buffer.from(readFileSync(cache)).toString("base64"),
      ),
    );
    writeFileSync(cache, JSON.stringify([value]));
    assert.throws(() =>
      validateCacheBytes(
        Buffer.from(JSON.stringify([{ ...value, key: { ...key, version: "0.609.0" } }])),
        root,
        version,
      ),
    );
    assert.throws(() =>
      validateCacheBytes(
        Buffer.from(JSON.stringify([{ ...value, key: { ...key, format: 2 } }])),
        root,
        version,
      ),
    );
    writeFileSync(join(modules, "vite"), "an undeclared external stub\n");
    assert.throws(() =>
      auditConfigCache(root, changed, cacheSnapshot(root, version), record, "stub"),
    );
    rmSync(join(modules, "vite"));
    rmSync(cache);
    assert.throws(() =>
      auditConfigCache(root, changed, cacheSnapshot(root, version), record, "deleted-cache"),
    );
    rmSync(join(modules, ".vize"), { recursive: true });
    const foreign = join(root, "foreign-cache");
    mkdirSync(foreign);
    symlinkSync(foreign, join(modules, ".vize"), process.platform === "win32" ? "junction" : "dir");
    assert.throws(() => cacheSnapshot(root, version), /cache ownership/);
  }));

test("npx replacement-cache detection rejects a private cached package tree", () =>
  fixture((root) => {
    assert.equal(npxCacheSnapshot(root).length, 1);
    mkdirSync(join(root, "_npx/foreign/node_modules"), { recursive: true });
    assert.throws(() => npxCacheSnapshot(root), /cached replacement/);
    rmSync(join(root, "_npx/foreign/node_modules"), { recursive: true });
    const target = join(root, "linked-replacement");
    mkdirSync(target);
    symlinkSync(
      target,
      join(root, "_npx/foreign/node_modules"),
      process.platform === "win32" ? "junction" : "dir",
    );
    assert.throws(() => npxCacheSnapshot(root), /cached replacement/);
  }));
