import assert from "node:assert/strict";
import { lstatSync, readFileSync, readdirSync, realpathSync, rmSync } from "node:fs";
import { join } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { a65Data, type A65State } from "./a-data.ts";
import { sha256 } from "./contract.ts";
import { healthy, type Image, type Packet, type Recorder } from "./execute.ts";
export interface A65Output {
  name: string;
  raw: Buffer;
  image: Image;
}
export interface A65Snapshot {
  names: string[];
  files: A65Output[];
  safe: boolean;
  present: boolean;
}
function directory(path: string) {
  const stat = lstatSync(path);
  assert(
    stat.isDirectory() && !stat.isSymbolicLink() && realpathSync(path) === path,
    "physical owned output directory required",
  );
}
export function retainA65Outputs(root: string, record: Recorder, label: string): A65Snapshot {
  const path = join(root, "out"),
    stat = lstatSync(path, { throwIfNoEntry: false });
  if (!stat) {
    record.json(label + ".outputs.json", {
      directoryPresent: false,
      roster: [],
      files: [],
      fullByteGoldenOwned: false,
    });
    return { names: [], files: [], safe: true, present: false };
  }
  directory(path);
  const names = readdirSync(path).sort(),
    files: A65Output[] = [],
    roster: unknown[] = [];
  let safe = true;
  for (const name of names) {
    const file = join(path, name),
      stat = lstatSync(file),
      known = a65Data.outputFiles.includes(name),
      regular = stat.isFile() && !stat.isSymbolicLink(),
      bounded = stat.size > 0 && stat.size <= 4 * 1024 ** 2,
      canonical = regular && realpathSync(file) === file;
    roster.push({
      name: known ? name : "UNRECOGNIZED_OUTPUT_NAME",
      nameSha256: sha256(name),
      bytes: stat.size,
      regular,
      canonical,
      known,
      bounded,
    });
    if (known && regular && canonical && bounded) {
      const raw = readFileSync(file),
        image = record.raw(label + "." + name + ".raw", raw);
      files.push({ name, raw, image });
    } else safe = false;
  }
  record.json(label + ".outputs.json", {
    directoryPresent: true,
    roster,
    files: files.map((file) => ({ name: file.name, image: file.image })),
    fullByteGoldenOwned: false,
    sourceDeclarationsInjected: false,
  });
  return { names, files, safe, present: true };
}
export function verifyA65Outputs(root: string, snapshot: A65Snapshot) {
  const path = join(root, "out");
  if (!snapshot.present) {
    assert.equal(lstatSync(path, { throwIfNoEntry: false }), undefined);
    return;
  }
  directory(path);
  assert(snapshot.safe, "unexpected declaration outputs refuse further execution");
  assert.deepEqual(readdirSync(path).sort(), snapshot.names);
  for (const file of snapshot.files) {
    const p = join(path, file.name),
      stat = lstatSync(p);
    assert(stat.isFile() && !stat.isSymbolicLink() && realpathSync(p) === p);
    assert(readFileSync(p).equals(file.raw), "retained declaration changed");
  }
}
export function resetA65Outputs(root: string, snapshot: A65Snapshot) {
  verifyA65Outputs(root, snapshot);
  if (snapshot.present) rmSync(join(root, "out"), { recursive: true });
  assert.equal(lstatSync(join(root, "out"), { throwIfNoEntry: false }), undefined);
}
export function requireSameA65Declarations(left: A65Snapshot, right: A65Snapshot) {
  assert.deepEqual(
    left.files.map((file) => file.name),
    right.files.map((file) => file.name),
  );
  for (const [index, file] of left.files.entries())
    assert(
      file.raw.equals(right.files[index]!.raw),
      "identical source restoration changed emitted declaration bytes",
    );
}
export function expectedA65Json(state: A65State) {
  const options = JSON.parse(state.files.find((file) => file.path === "tsconfig.json")!.source) as {
    compilerOptions: Record<string, unknown>;
  };
  return {
    files: [
      { file: "src/Foo.vue", diagnostics: [] },
      {
        file: "src/Probe.vue",
        diagnostics: state.expectedDiagnostic === null ? [] : [state.expectedDiagnostic],
      },
    ],
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: options.compilerOptions,
        files: ["src/Foo.vue", "src/Probe.vue"],
      },
    ],
    errorCount: state.status === 0 ? 0 : 1,
    warningCount: 0,
    fileCount: 2,
    ...(state.status === 0
      ? { declarations: a65Data.outputFiles.map((name) => "out/" + name) }
      : {}),
  };
}
function originalText(stdout: string, cwd: string) {
  const escape = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&"),
    duration = "[0-9]+(?:\\.[0-9]+)?(?:ns|µs|ms|s)";
  return [false, true].some((color) => {
    const green = (s: string) => (color ? "\x1b[32m" + s + "\x1b[0m" : s);
    const pattern =
      "^\\n" +
      escape(green("✓")) +
      " Type checked 2 files in " +
      duration +
      " \\(collect: " +
      duration +
      ", imports: " +
      duration +
      ", gen: " +
      duration +
      ", corsa: " +
      duration +
      ", dts: " +
      duration +
      "\\)\\n  " +
      escape(green("No type errors found!")) +
      "\\n  " +
      escape(green("Emitted 3 declaration file(s)")) +
      " to " +
      escape(join(cwd, "out")) +
      "\\n$";
    return new RegExp(pattern, "u").test(stdout);
  });
}
export function assessA65(state: A65State, packet: Packet, snapshot: A65Snapshot) {
  healthy(packet, state.status);
  assert(snapshot.safe, "unsafe output population");
  const clean = state.status === 0;
  assert.deepEqual(snapshot.names, clean ? [...a65Data.outputFiles].sort() : []);
  assert.equal(snapshot.files.length, clean ? 3 : 0);
  for (const file of snapshot.files) {
    const text = new TextDecoder("utf8", { fatal: true }).decode(file.raw);
    assert(!text.includes("vize:virtual-ts"), "private declaration specifier leaked");
  }
  const progress =
    "Building Corsa virtual project for 2 files under " +
    packet.cwd +
    "...\nRunning Corsa diagnostics for 2 files...\n" +
    (clean ? "" : "Skipping declaration emit because type errors were reported.\n");
  assert(packet.stderr === progress, "complete source-owned progress stream differs");
  if (state.json) {
    let actual: unknown;
    try {
      actual = JSON.parse(packet.stdout!);
    } catch {
      assert.fail("source-owned JSON stream malformed");
    }
    const expected = expectedA65Json(state);
    assert(
      isDeepStrictEqual(actual, expected),
      "complete source-owned JSON/program/diagnostic shape differs",
    );
  } else
    assert(originalText(packet.stdout!, packet.cwd), "source-owned original text grammar differs");
  return {
    id: state.id,
    status: packet.status,
    sourceIntentMatches: true,
    wholeStreamsAndDeclarationsRetained: true,
    fullDeclarationByteGoldenOwned: false,
    timingNumbersClaimed: false,
    publicAcceptance: false,
  };
}
