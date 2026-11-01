/** Literal inherited-Options INPUTS and public packet controls; no portable wording rewrite. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

type State = "invalid" | "typed";
type Fixture = {
  schema: string;
  sourceSha: string;
  args: string[];
  files: {
    file: string;
    sourcePath: string;
    bytes: number;
    sha256: string;
    gitBlob: string;
    base64: string;
  }[];
  repair: [string, string][];
  absent: string[];
};
const digest = (body: Buffer) => createHash("sha256").update(body).digest("hex");
export function loadInheritedOptionsInputs() {
  const bytes = fs.readFileSync(
    new URL(
      "../../../tests/_fixtures/tooling/paired-inherited-options-inputs.json",
      import.meta.url,
    ),
  );
  assert.equal(digest(bytes), "220f58faa383588ee412c0c97028a139059ec2740b0b7b402d0c7227ec62be0c");
  const fixture = JSON.parse(bytes.toString("utf8")) as Fixture;
  assert.equal(fixture.schema, "vize.pairedInheritedOptionsInputs.v1");
  assert.equal(fixture.sourceSha, "84c4fe5b8fcf5479817a9744683b00fe3841b478");
  assert.deepEqual(fixture.args, ["check", "--format", "json"]);
  assert.deepEqual(
    fixture.files.map((row) => row.file),
    ["src/shared.ts", "src/base.ts", "src/Invalid.vue", "tsconfig.json"],
  );
  for (const row of fixture.files) {
    const body = Buffer.from(row.base64, "base64");
    assert.equal(body.toString("base64"), row.base64);
    assert.equal(body.length, row.bytes);
    assert.equal(digest(body), row.sha256);
    assert.equal(
      createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
      row.gitBlob,
    );
  }
  return fixture;
}

export function inheritedOptionsState(fixture: Fixture, state: State) {
  assert(state === "invalid" || state === "typed");
  return fixture.files.map((row) => {
    let body = Buffer.from(row.base64, "base64");
    if (row.file === "src/Invalid.vue" && state === "typed") {
      let text = body.toString("utf8");
      for (const [before, after] of fixture.repair) {
        assert.equal(text.split(before).length, 2, "literal repair must be unique");
        text = text.replace(before, after);
      }
      body = Buffer.from(text);
    }
    return { file: row.file, body };
  });
}

export function verifyInheritedInputSources(base: string, head: string, fixture: Fixture) {
  for (const root of [base, head])
    for (const row of fixture.files)
      assert(
        fs.readFileSync(path.join(root, row.sourcePath)).equals(Buffer.from(row.base64, "base64")),
        "actual containing source changed the literal original fixture",
      );
}

function ownedInventory(root: string, provider: string) {
  assert.equal(fs.realpathSync(root), root);
  assert.deepEqual(fs.readdirSync(root).sort(), ["node_modules", "src", "tsconfig.json"]);
  const modules = path.join(root, "node_modules");
  assert(fs.lstatSync(modules).isSymbolicLink());
  assert.equal(fs.realpathSync(modules), fs.realpathSync(path.join(provider, "node_modules")));
  assert(fs.lstatSync(path.join(root, "src")).isDirectory());
  assert(!fs.lstatSync(path.join(root, "src")).isSymbolicLink());
  assert.deepEqual(fs.readdirSync(path.join(root, "src")).sort(), [
    "Invalid.vue",
    "base.ts",
    "shared.ts",
  ]);
}

export function materializeInheritedOptions(
  root: string,
  provider: string,
  fixture: Fixture,
  state: State,
) {
  assert(path.isAbsolute(root) && fs.realpathSync(path.dirname(root)) === path.dirname(root));
  if (fs.lstatSync(root, { throwIfNoEntry: false }) === undefined) {
    fs.mkdirSync(root);
    fs.mkdirSync(path.join(root, "src"));
    fs.symlinkSync(
      fs.realpathSync(path.join(provider, "node_modules")),
      path.join(root, "node_modules"),
      "dir",
    );
  } else ownedInventory(root, provider);
  const files = inheritedOptionsState(fixture, state);
  for (const row of files) {
    const file = path.join(root, row.file);
    const previous = fs.lstatSync(file, { throwIfNoEntry: false });
    assert(previous === undefined || (previous.isFile() && !previous.isSymbolicLink()));
    fs.writeFileSync(file, row.body);
  }
  return verifyInheritedOwnedInputs(root, provider, fixture, state);
}

export function verifyInheritedOwnedInputs(
  root: string,
  provider: string,
  fixture: Fixture,
  state: State,
) {
  ownedInventory(root, provider);
  const files = inheritedOptionsState(fixture, state);
  for (const row of files) assert(fs.readFileSync(path.join(root, row.file)).equals(row.body));
  for (const absent of fixture.absent)
    assert(fs.lstatSync(path.join(root, absent), { throwIfNoEntry: false }) === undefined);
  return {
    state,
    files: files.map(({ file, body }) => ({
      file,
      bytes: body.length,
      sha256: digest(body),
      base64: body.toString("base64"),
    })),
    absent: fixture.absent,
    nativeLoadedFiles: "unobserved",
  };
}

/** Baseline functional law only. Preserve the actual full receiver text, then compare all H bytes. */
export function verifyInheritedOptionsOutput(
  stdout: Buffer,
  stderr: Buffer,
  status: number | null,
  root: string,
  state: State,
  fixture: Fixture,
) {
  assert.equal(status, state === "typed" ? 0 : 1);
  const text = new TextDecoder("utf-8", { fatal: true }).decode(stdout);
  const packet = JSON.parse(text);
  assert.deepEqual(Object.keys(packet).sort(), [
    "errorCount",
    "fileCount",
    "files",
    "programs",
    "warningCount",
  ]);
  assert.equal(packet.errorCount, state === "typed" ? 0 : 5);
  assert.equal(packet.warningCount, 0);
  assert.equal(packet.fileCount, 3);
  const names = ["src/Invalid.vue", "src/base.ts", "src/shared.ts"];
  const config = JSON.parse(
    inheritedOptionsState(fixture, state)
      .find((row) => row.file === "tsconfig.json")!
      .body.toString("utf8"),
  );
  assert.deepEqual(packet.programs, [
    { root: ".", tsconfig: "tsconfig.json", compilerOptions: config.compilerOptions, files: names },
  ]);
  assert.deepEqual(
    packet.files.map((row: { file: string }) => row.file),
    names,
  );
  for (const row of packet.files) {
    assert.deepEqual(Object.keys(row).sort(), ["diagnostics", "file"]);
    if (state === "typed" || row.file !== "src/Invalid.vue") assert.deepEqual(row.diagnostics, []);
  }
  if (state === "invalid") {
    const diagnostics = packet.files[0].diagnostics;
    assert.equal(diagnostics.length, 5);
    const source = Buffer.from(
      fixture.files.find((row) => row.file === "src/Invalid.vue")!.base64,
      "base64",
    )
      .toString("utf8")
      .split("\n")[1]
      .trim();
    const firstFour = [
      `error:2:20 [TS2345] Argument of type 'number' is not assignable to parameter of type 'string'. (source: ${source}; binding: 'title')`,
      `error:2:32 [TS2339] Property 'toFixed' does not exist on type 'boolean'. (source: ${source})`,
      `error:2:62 [TS2551] Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'? (source: ${source})`,
      `error:2:87 [TS2339] Property 'toUpperCase' does not exist on type 'number'. (source: ${source})`,
    ];
    assert.deepEqual(diagnostics.slice(0, 4), firstFour);
    assert.equal(typeof diagnostics[4], "string");
    const prefix = "error:2:107 [TS2339] Property 'missing' does not exist on type '",
      suffix = `'. (source: ${source})`;
    assert(diagnostics[4].startsWith(prefix) && diagnostics[4].endsWith(suffix));
    assert(diagnostics[4].length > prefix.length + suffix.length);
  }
  assert(
    stderr.equals(
      Buffer.from(
        `Building Corsa virtual project for 3 files under ${root}...\nRunning Corsa diagnostics for 3 files...\n`,
      ),
    ),
    "full original progress stderr changed",
  );
  return packet;
}

export function compareInheritedOptionsRaw(
  before: { stdout: Buffer; stderr: Buffer; status: number | null },
  after: typeof before,
) {
  assert.equal(after.status, before.status);
  assert(
    after.stdout.equals(before.stdout),
    "whole original diagnostic packet changed; no receiver normalization",
  );
  assert(after.stderr.equals(before.stderr), "whole original stderr changed");
}
