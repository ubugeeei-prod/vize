import assert from "node:assert/strict";
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
import { join } from "node:path";
import { test } from "node:test";
import { Recorder, sha256 } from "../../tools/support/config/windows_consumer/common.ts";
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

function fixture(run: (root: string) => void): void {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "editor-custody-control-")));
  try {
    run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
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
    for (const name of ["declared.mjs", "declared.cjs"]) {
      writeFileSync(
        join(pkg, "bin", name),
        'throw new Error("control must never execute a fixture");\n',
      );
      save("bin/" + name);
      const selected = resolvePnpm(root);
      assert.deepEqual(selected.argv, [process.execPath, realpathSync(join(pkg, "bin", name))]);
      assert.equal(selected.package.entry.sha256, sha256(readFileSync(join(pkg, "bin", name))));
    }
    save("bin/missing.mjs");
    assert.throws(() => resolvePnpm(root));
    save("bin/declared.mjs", "12.0.0");
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
    assert.equal(argv[0], process.execPath);
  },
);
