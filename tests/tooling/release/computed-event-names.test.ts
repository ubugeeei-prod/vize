import assert from "node:assert/strict";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  assertCustody,
  assertCheckerVersion,
  assertNativeIdentity,
  assertObservations,
  assertPublicPackage,
  assertSourcePins,
  beforeOutputSha256,
  captureProcess,
  checkerVersion,
  checkerVersionOutput,
  fileImage,
  findNpmCli,
  main,
  negativeOutput,
  negativeProbeSha256,
  normalizeOutput,
  parseArguments,
  sha256,
  sourcePins,
  type Observation,
} from "../../../tools/support/release/computed_event_names.ts";

const fixture = fileURLToPath(
  new URL(
    "../../_fixtures/differential/typechecker/dynamic-component-event-names-11140/",
    import.meta.url,
  ),
);
const original = () =>
  Object.fromEntries(
    Object.keys(sourcePins).map((name) => [
      name,
      readFileSync(path.join(fixture, `${path.basename(name)}.txt`)),
    ]),
  );
const clean = (id: string): Observation => ({
  id,
  status: 0,
  signal: null,
  error: null,
  stdout: "",
  stderr: "",
});
const accepted = (): Observation[] => [
  clean("valid"),
  { ...clean("invalid"), status: 2, stdout: negativeOutput },
  clean("restored"),
];

test("original reporter inputs and independent invalid-input hash are frozen", () => {
  assertSourcePins(original());
  assert.equal(
    sha256(original()["src/Probe.vue"].toString("utf8").replace("pressed = true", "pressed = 1")),
    negativeProbeSha256,
  );
  for (const name of Object.keys(sourcePins)) {
    const changed = original();
    changed[name] = Buffer.concat([changed[name], Buffer.from("\n")]);
    assert.throws(() => assertSourcePins(changed), /frozen reporter input/u);
  }
  const missing = original();
  delete missing["src/Child.vue"];
  assert.throws(() => assertSourcePins(missing));
  assert.throws(() => assertSourcePins({ ...original(), "extra.vue": Buffer.from("") }));
});

test("full valid, authored TS2322, and restored packets must match exactly", () => {
  assertObservations("after", accepted());
  const windows = accepted();
  windows[1].stdout = negativeOutput.replace("src/", "src\\").replace(/\n/gu, "\r\n");
  assertObservations("after", windows);
  assert.equal(normalizeOutput(windows[1].stdout), negativeOutput);
  for (const mutation of [
    (rows: Observation[]) => rows.pop(),
    (rows: Observation[]) => rows.push(clean("extra")),
    (rows: Observation[]) => rows.reverse(),
    (rows: Observation[]) => {
      rows[0].stdout = "unexpected\n";
    },
    (rows: Observation[]) => {
      rows[2].stdout = negativeOutput;
    },
    (rows: Observation[]) => {
      rows[1].stdout = "";
    },
    (rows: Observation[]) => {
      rows[1].stdout += negativeOutput;
    },
    (rows: Observation[]) => {
      rows[1].stdout = negativeOutput.replace("(11,34)", "(11,35)");
    },
    (rows: Observation[]) => {
      rows[1].stdout = negativeOutput.replace("TS2322", "TS7006");
    },
    (rows: Observation[]) => {
      rows[1].stdout = negativeOutput.replace("number", "any");
    },
    (rows: Observation[]) => {
      rows[1].stdout = negativeOutput.trimEnd();
    },
    (rows: Observation[]) => {
      rows[0].stderr = "warning\n";
    },
    (rows: Observation[]) => {
      rows[1].status = 0;
    },
    (rows: Observation[]) => {
      rows[2].status = null;
    },
    (rows: Observation[]) => {
      rows[0].signal = "SIGTERM";
    },
    (rows: Observation[]) => {
      rows[1].error = { message: "failed", code: "ENOENT" };
    },
  ]) {
    const rows = accepted();
    mutation(rows);
    assert.throws(() => assertObservations("after", rows));
  }
});

test("historical observation cannot accept counts alone or an after packet", () => {
  assert.equal(
    beforeOutputSha256,
    "8339daf18f35398693452fc4faf2950dffde11eafa14fb12053a642efb08d1df",
  );
  const counterfeit = {
    ...clean("baseline"),
    status: 2,
    stdout: "src/Probe.vue(1,1): error TS1005: fake\n".repeat(189),
  };
  assert.throws(() => assertObservations("before", [counterfeit]), /complete historical output/u);
  assert.throws(() => assertObservations("before", accepted()));
  assert.throws(() => assertObservations("after", [counterfeit]));
});

test("actual checker version and loaded native binding identities must agree exactly", () => {
  const version = { ...clean("checker-version"), stdout: checkerVersionOutput };
  assertCheckerVersion(version);
  for (const changed of [
    { ...version, stdout: version.stdout.replace("3e9236361fa9", "000000000000") },
    { ...version, stdout: version.stdout + "extra\n" },
    { ...version, stderr: "warning\n" },
    { ...version, status: 2 },
    { ...version, signal: "SIGTERM" },
    { ...version, error: { message: "cannot execute" } },
  ])
    assert.throws(() => assertCheckerVersion(changed));
  const token = "0123456789abcdef".repeat(2);
  const identity = { kind: "native", token };
  assertNativeIdentity(token, token, identity);
  for (const args of [
    [null, null, identity],
    ["not-a-native-image-token", "not-a-native-image-token", identity],
    [token, "0".repeat(32), identity],
    [token, token, { kind: "process", nonce: token }],
    [token, token, { ...identity, token: "0".repeat(32) }],
  ])
    assert.throws(() => assertNativeIdentity(args[0], args[1], args[2]));
});

test("custody detects byte changes, missing native executables, and duplicate images", () => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-computed-custody-"));
  try {
    const executable = path.join(directory, "native");
    writeFileSync(executable, "real binary bytes");
    const image = fileImage(executable);
    assertCustody([image]);
    writeFileSync(executable, "fake binary bytes"); // Equal length; size alone is insufficient.
    assert.throws(() => assertCustody([image]), /installed file changed/u);
    writeFileSync(executable, "real binary bytes");
    assert.throws(() => assertCustody([image, image]));
    rmSync(executable);
    assert.throws(() => assertCustody([image]));
    assert.throws(() => assertCustody([]));
  } finally {
    rmSync(directory, { recursive: true });
  }
});

test("installed package identity rejects registry, version, link, and integrity substitutions", () => {
  const manifest = { name: "@tsrs/linux-x64-gnu", version: checkerVersion };
  const locked = {
    version: checkerVersion,
    resolved: "https://registry.npmjs.org/@tsrs/linux-x64-gnu/-/linux-x64-gnu.tgz",
    integrity: "sha512-YWJjZA==",
  };
  assertPublicPackage(manifest, locked, manifest.name, checkerVersion);
  for (const changed of [
    { ...locked, version: "0.0.0" },
    { ...locked, resolved: "https://example.com/fixture.tgz" },
    { ...locked, resolved: "file:../source" },
    { ...locked, integrity: "" },
    { ...locked, link: true },
  ])
    assert.throws(() => assertPublicPackage(manifest, changed, manifest.name, checkerVersion));
  assert.throws(() =>
    assertPublicPackage(
      { ...manifest, name: "source-checker" },
      locked,
      manifest.name,
      checkerVersion,
    ),
  );
  assert.throws(() =>
    assertPublicPackage({ ...manifest, version: "0.0.0" }, locked, manifest.name, checkerVersion),
  );
});

test("raw streams and failed process metadata survive oracle rejection", () => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-computed-output-"));
  try {
    const observation = captureProcess(
      directory,
      "valid",
      process.execPath,
      [
        "-e",
        "process.stdout.write('unexpected\\n');process.stderr.write('warning\\n');process.exitCode=2",
      ],
      directory,
    );
    assert.throws(() => assertObservations("after", [observation, ...accepted().slice(1)]));
    assert.equal(readFileSync(path.join(directory, "valid.stdout.txt"), "utf8"), "unexpected\n");
    assert.equal(readFileSync(path.join(directory, "valid.stderr.txt"), "utf8"), "warning\n");
    assert.equal(
      JSON.parse(readFileSync(path.join(directory, "valid.process.json"), "utf8")).status,
      2,
    );
    const absent = captureProcess(
      directory,
      "missing",
      path.join(directory, "does-not-exist"),
      [],
      directory,
    );
    assert.equal(absent.status, null);
    assert.equal(absent.error?.code, "ENOENT");
    assert.equal(
      JSON.parse(readFileSync(path.join(directory, "missing.process.json"), "utf8")).error.code,
      "ENOENT",
    );
  } finally {
    rmSync(directory, { recursive: true });
  }
});

test("the real CLI retains a failed receipt when a source override is refused", () => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-computed-refusal-"));
  try {
    const output = path.join(directory, "refused");
    const harness = fileURLToPath(
      new URL("../../../tools/support/release/computed_event_names.ts", import.meta.url),
    );
    const result = captureProcess(
      directory,
      "refusal",
      process.execPath,
      [harness, "--version", "0.608.0", "--out", output],
      directory,
      { ...process.env, TSRS_BINARY: path.join(directory, "source-checker") },
    );
    assert.equal(result.status, 1);
    const receipt = JSON.parse(readFileSync(path.join(output, "receipt.json"), "utf8"));
    assert.equal(receipt.status, "FAILED");
    assert.equal(receipt.accepted, false);
    assert.match(receipt.error.message, /runtime override: TSRS_BINARY/u);
    assert.equal(existsSync(path.join(output, "consumer")), false, "refuse before installation");
  } finally {
    rmSync(directory, { recursive: true });
  }
});

test("argument validation freezes before version and requires exclusive absolute outputs", async () => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-computed-existing-"));
  try {
    const args = ["--version", "0.608.0", "--out", directory];
    assert.equal(parseArguments(args).phase, "after");
    assert.equal(parseArguments([...args, "--phase", "before"]).phase, "before");
    for (const invalid of [
      ["--version", "latest", "--out", directory],
      ["--version", "0.609.0", "--out", directory, "--phase", "before"],
      ["--version", "0.608.0", "--out", "relative"],
      [...args, "--version", "0.609.0"],
      [...args, "--phase", "anything"],
      [...args, "--extra", "value"],
      [...args, "--phase"],
    ])
      assert.throws(() => parseArguments(invalid));
    const marker = path.join(directory, "receipt.json");
    writeFileSync(marker, "original evidence\n");
    await assert.rejects(main(args), /EEXIST/u);
    assert.equal(readFileSync(marker, "utf8"), "original evidence\n");
    assert.equal(path.basename(findNpmCli()), "npm-cli.js");
  } finally {
    rmSync(directory, { recursive: true });
  }
});
