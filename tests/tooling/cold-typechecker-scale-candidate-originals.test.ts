import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  loadInheritedOptionsInputs,
  inheritedOptionsState,
  materializeInheritedOptions,
  verifyInheritedOwnedInputs,
  verifyInheritedOptionsOutput,
  compareInheritedOptionsRaw,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-originals.ts";

test("literal original inputs have one exact five-change repair, same config/roster and explicit absences", (t) => {
  const fixture = loadInheritedOptionsInputs(),
    invalid = inheritedOptionsState(fixture, "invalid"),
    typed = inheritedOptionsState(fixture, "typed");
  assert.equal(invalid.length, 4);
  assert.deepEqual(fixture.args, ["check", "--format", "json"]);
  assert.deepEqual(
    invalid.filter((row, index) => !row.body.equals(typed[index].body)).map((row) => row.file),
    ["src/Invalid.vue"],
  );
  assert(
    invalid[2].body.includes(Buffer.from("greet(1)")) &&
      typed[2].body.includes(Buffer.from("greet('x')")),
  );
  assert(!typed[2].body.includes(Buffer.from("{{ missing }}")));
  assert.equal(JSON.parse(invalid[3].body.toString()).compilerOptions.skipLibCheck, undefined);
  const parent = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "inherited-options-")));
  t.after(() => fs.rmSync(parent, { recursive: true, force: true }));
  const provider = path.join(parent, "provider"),
    root = path.join(parent, "project");
  fs.mkdirSync(provider);
  fs.mkdirSync(path.join(provider, "node_modules"));
  const first = materializeInheritedOptions(root, provider, fixture, "invalid");
  const repaired = materializeInheritedOptions(root, provider, fixture, "typed");
  assert.equal(first.files.length, 4);
  assert.equal(repaired.files.length, 4);
  const restored = materializeInheritedOptions(root, provider, fixture, "invalid");
  assert.deepEqual(restored, first);
  fs.writeFileSync(path.join(root, "src/Wrapped.vue"), "unowned extra input");
  assert.throws(() => verifyInheritedOwnedInputs(root, provider, fixture, "invalid"));
  assert.throws(() => materializeInheritedOptions(root, provider, fixture, "typed"));
});

function packets() {
  const fixture = loadInheritedOptionsInputs(),
    root = "/owned/inherited-project";
  const source = inheritedOptionsState(fixture, "invalid")[2].body.toString().split("\n")[1].trim();
  const messages = [
    `error:2:20 [TS2345] Argument of type 'number' is not assignable to parameter of type 'string'. (source: ${source}; binding: 'title')`,
    `error:2:32 [TS2339] Property 'toFixed' does not exist on type 'boolean'. (source: ${source})`,
    `error:2:62 [TS2551] Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'? (source: ${source})`,
    `error:2:87 [TS2339] Property 'toUpperCase' does not exist on type 'number'. (source: ${source})`,
    `error:2:107 [TS2339] Property 'missing' does not exist on type 'ActualCompleteReceiver<unknown>'. (source: ${source})`,
  ];
  const names = ["src/Invalid.vue", "src/base.ts", "src/shared.ts"];
  const config = JSON.parse(inheritedOptionsState(fixture, "invalid")[3].body.toString());
  const packet = {
    files: names.map((file, index) => ({ file, diagnostics: index === 0 ? messages : [] })),
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: config.compilerOptions,
        files: names,
      },
    ],
    errorCount: 5,
    warningCount: 0,
    fileCount: 3,
  };
  const stderr = Buffer.from(
    `Building Corsa virtual project for 3 files under ${root}...\nRunning Corsa diagnostics for 3 files...\n`,
  );
  return { fixture, root, packet, stderr };
}
test("original functional guard preserves complete actual receiver wording and whole B/H packets", () => {
  // Synthetic public packets exercise the reader; no native oracle or provider execution is claimed.
  const x = packets(),
    stdout = Buffer.from(JSON.stringify(x.packet));
  const parsed = verifyInheritedOptionsOutput(stdout, x.stderr, 1, x.root, "invalid", x.fixture);
  assert(parsed.files[0].diagnostics[4].includes("ActualCompleteReceiver<unknown>"));
  compareInheritedOptionsRaw(
    { stdout, stderr: x.stderr, status: 1 },
    { stdout: Buffer.from(stdout), stderr: Buffer.from(x.stderr), status: 1 },
  );
  assert.throws(() =>
    compareInheritedOptionsRaw(
      { stdout, stderr: x.stderr, status: 1 },
      {
        stdout: Buffer.from(stdout.toString().replace("ActualCompleteReceiver", "ChangedReceiver")),
        stderr: x.stderr,
        status: 1,
      },
    ),
  );
  const typed = structuredClone(x.packet);
  typed.errorCount = 0;
  typed.files.forEach((row) => {
    row.diagnostics = [];
  });
  assert.doesNotThrow(() =>
    verifyInheritedOptionsOutput(
      Buffer.from(JSON.stringify(typed)),
      x.stderr,
      0,
      x.root,
      "typed",
      x.fixture,
    ),
  );
});
test("original guard refuses missing/extra/wrong authored findings, skipped files, changed config and stderr", () => {
  for (const alter of [
    (x: ReturnType<typeof packets>) => {
      x.packet.files[0].diagnostics.pop();
    },
    (x: ReturnType<typeof packets>) => {
      x.packet.files[0].diagnostics.push("extra opaque error");
    },
    (x: ReturnType<typeof packets>) => {
      x.packet.files[0].diagnostics[0] = "wrong binding/message";
    },
    (x: ReturnType<typeof packets>) => {
      x.packet.files.pop();
    },
    (x: ReturnType<typeof packets>) => {
      x.packet.programs[0].compilerOptions.noEmit = false;
    },
    (x: ReturnType<typeof packets>) => {
      x.stderr = Buffer.alloc(0);
    },
  ]) {
    const x = packets();
    alter(x);
    assert.throws(() =>
      verifyInheritedOptionsOutput(
        Buffer.from(JSON.stringify(x.packet)),
        x.stderr,
        1,
        x.root,
        "invalid",
        x.fixture,
      ),
    );
  }
});
