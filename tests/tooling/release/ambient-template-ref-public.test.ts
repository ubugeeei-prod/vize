import assert from "node:assert/strict";
import test from "node:test";
import { sha256 } from "../../../tools/support/config/windows_consumer/common.ts";
import {
  assertRole,
  release,
  sdkFiles,
} from "../../../tools/support/release/ambient_template_ref_public/authority.ts";
import {
  checkerSha256,
  installedManifest,
  providerVersions,
} from "../../../tools/support/release/ambient_template_ref_public/install.ts";
import {
  assertCliOutput,
  assertJournal,
  expectedReport,
  loadFixture,
  phases,
} from "../../../tools/support/release/ambient_template_ref_public/observe.ts";
import { hook } from "../../../tools/support/release/editor_vite_stub/custody.ts";

test("literal primary issue bytes and ES2024 options stay separate from historical variants", () => {
  const fixture = loadFixture();
  assert.equal(
    fixture.pins.rawCaptureSha256,
    "98d0053ae97b9e84995e7c611962a317e53624698c6df36c80dafb7d7dd89919",
  );
  assert.equal(
    sha256(fixture.probe),
    "158a66cf752f4ba13fe3715e5a71b01a4d5f9fce9157468fb20086fd427c58b3",
  );
  assert.equal(
    sha256(fixture.global),
    "6e0097d60f560ef938b1fcadda7dd7070747f652648ce79a469c6a128ed76b0f",
  );
  assert.equal(fixture.probe.split("\n").length, 6);
  assert.ok(fixture.probe.includes("focus();\n</script>"));
  assert.equal(
    fixture.repaired,
    fixture.probe.replace(
      "useTemplateRef('target')",
      "useTemplateRef<HTMLButtonElement>('target')",
    ),
  );
  assert.deepEqual(phases, ["original", "typed-repair", "restored", "missing-global"]);
  const config: unknown = JSON.parse(fixture.config);
  assert.deepEqual(config, {
    compilerOptions: {
      noEmit: true,
      strict: true,
      skipLibCheck: true,
      module: "ESNext",
      moduleResolution: "Bundler",
      lib: ["ES2024", "DOM"],
      types: [],
    },
    include: ["*.vue", "*.ts"],
    vueCompilerOptions: { composables: { useTemplateRef: [] } },
  });
});

test("this receiver admits only C12 and its nine-declaration installed provider population", () => {
  assertRole(release.version, release.run, release.candidate, release.source);
  for (const values of [
    ["0.621.0", release.run, release.candidate, release.source],
    [release.version, "999", release.candidate, release.source],
    [release.version, release.run, "a".repeat(40), release.source],
    [release.version, release.run, release.candidate, "b".repeat(40)],
  ])
    assert.throws(() => assertRole(values[0], values[1], values[2], values[3]));
  assert.equal(sdkFiles.length, 9);
  assert.ok(!sdkFiles.some((file: string) => file === "virtual-ts-slots.d.ts"));
  assert.deepEqual(installedManifest().dependencies, {
    vize: "0.620.0",
    "@vizejs/native": "0.620.0",
    vue: "3.6.0-rc.13",
    typescript: "6.0.3",
    "vue-tsc": "3.3.13",
    "@typescript/typescript-linux-x64": "7.1.0-beta",
  });
  assert.equal(providerVersions.vue, "3.6.0-rc.13");
  assert.equal(checkerSha256, "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70");
});

test("all four commands require complete unnormalized JSON and exact stderr/status", () => {
  const project = "/owned/original-10980";
  for (const phase of phases) {
    const expected = expectedReport(phase),
      status = phase === "typed-repair" ? 0 : 1;
    const stdout = Buffer.from(JSON.stringify(expected, null, 2) + "\n");
    const stderr = Buffer.from(
      `Building Corsa virtual project for ${expected.fileCount} files under ${project}...\nRunning Corsa diagnostics for ${expected.fileCount} files...\n`,
    );
    assert.deepEqual(assertCliOutput(phase, stdout, stderr, status, project), expected);
    for (const changed of [
      Buffer.concat([stdout, Buffer.from("extra\n")]),
      Buffer.from(stdout.toString().replace("ES2024", "ESNext")),
      Buffer.from(stdout.toString().replace('"warningCount": 0', '"warningCount": 1')),
    ])
      assert.throws(() => assertCliOutput(phase, changed, stderr, status, project));
    assert.throws(() => assertCliOutput(phase, stdout, Buffer.alloc(0), status, project));
    assert.throws(() => assertCliOutput(phase, stdout, stderr, status === 0 ? 1 : 0, project));
    if (phase !== "typed-repair") {
      const lost = structuredClone(expected);
      lost.files[0].diagnostics = [];
      lost.errorCount = 0;
      assert.throws(() =>
        assertCliOutput(
          phase,
          Buffer.from(JSON.stringify(lost, null, 2) + "\n"),
          stderr,
          status,
          project,
        ),
      );
    }
  }
  assert.equal(
    expectedReport("original").files[0].diagnostics[0],
    "error:3:32 [TS2339] Property 'focus' does not exist on type '{}'.",
  );
  assert.equal(
    expectedReport("missing-global").files[0].diagnostics[0],
    "error:2:16 [TS2304] Cannot find name 'useTemplateRef'.",
  );
  assert.deepEqual(expectedReport("original"), expectedReport("restored"));
  assert.equal(expectedReport("missing-global").fileCount, 1);
});

test("native load custody is joined to the independently captured child PID and exact command", () => {
  const identity = {
    root: "/installed",
    cli: "/installed/node_modules/vize/bin/vize",
    binary: { path: "/installed/native.node", sha256: "d".repeat(64) },
    checker: "/installed/beta/tsc",
  };
  const argv = [
    "/node24",
    "--require",
    hook,
    identity.cli,
    "check",
    "--no-config",
    "--tsconfig",
    "tsconfig.json",
    "--format",
    "json",
    "--corsa-path",
    identity.checker,
  ];
  const event = { schema: "vize-public-native-custody-event-v1", pid: 123 };
  const initialized = {
    ...event,
    event: "initialized",
    installRoot: identity.root,
    nativePath: identity.binary.path,
    sha256: identity.binary.sha256,
    node: argv[0],
    argv: [argv[0], ...argv.slice(3)],
  };
  const attempt = {
    ...event,
    event: "attempt",
    actualPath: identity.binary.path,
    sha256: identity.binary.sha256,
    expectedNative: true,
    corsaPath: "",
  };
  const returned = { ...attempt, event: "returned" };
  const raw = (events: unknown[]) =>
    Buffer.from(events.map((row) => JSON.stringify(row)).join("\n") + "\n");
  for (const status of [0, 1]) {
    const events =
      status === 0
        ? [initialized, attempt, returned, { ...event, event: "exit", code: 0 }]
        : [initialized, attempt, returned];
    const child = { argv, pid: 123, status, signal: null, error: null };
    assertJournal(raw(events), identity, child);
    assert.throws(() =>
      assertJournal(raw([{ ...initialized, injected: true }, ...events.slice(1)]), identity, child),
    );
    assert.throws(() => assertJournal(raw(events), identity, { ...child, pid: 124 }));
    assert.throws(() =>
      assertJournal(raw(events), { ...identity, checker: "/foreign/tsc" }, child),
    );
    assert.throws(() =>
      assertJournal(raw(events), identity, { ...child, status: status === 0 ? 1 : 0 }),
    );
    assert.throws(() =>
      assertJournal(
        raw([initialized, { ...attempt, sha256: "e".repeat(64) }, returned]),
        identity,
        child,
      ),
    );
    assert.throws(() =>
      assertJournal(raw([initialized, attempt, returned, returned]), identity, child),
    );
    assert.throws(() =>
      assertJournal(raw(events).subarray(0, raw(events).length - 1), identity, child),
    );
  }
});
