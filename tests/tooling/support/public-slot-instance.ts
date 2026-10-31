import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import ts from "../../../npm/cli/node_modules/typescript/lib/typescript.js";

const preimageSource = "1a65bbd5e95da036bd7e6a0fee7f8b214023a6b0";
const preimages: Record<string, string> = {
  "virtual-ts.d.ts": "0aadc144db9c92c203751ecf77f86ba84217ec3241e88bc0b0b8168265b94b28",
  "virtual-ts.d.mts": "0aadc144db9c92c203751ecf77f86ba84217ec3241e88bc0b0b8168265b94b28",
  "virtual-ts.d.cts": "0aadc144db9c92c203751ecf77f86ba84217ec3241e88bc0b0b8168265b94b28",
  "content-mapper-helpers.d.ts": "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
  "content-mapper-helpers.d.mts":
    "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
  "content-mapper-helpers.d.cts":
    "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
  "generated/content-mapper-types.d.ts":
    "7457fd86aafb4b2a5b1edbddedf538a45a072517645dd0ece2fd4c074a0f632e",
  "virtual-ts-globals.d.ts": "d647737ea035f166c94c131013af3265a3a33783dbad307b2eb3c6d558fc476a",
  "virtual-ts-patterns.d.ts": "e42c2c431553563663e28f75b4e68f8728d928a2867c857c76ef8cf30fe0f545",
};
const failures = [
  "TwoFirstAny",
  "NineFirstAny",
  "NineLastAny",
  "MixedGenericNeverFirst",
  "MixedGenericNeverLast",
];
const unknownControls = ["TwoLastUnknown", "SeventeenFirstUnknown"];
const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
type Row = { name: string; display: string; flags: number; any: boolean; intrinsic: string | null };
function typeRows(file: string) {
  const program = ts.createProgram([file], {
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.Node16,
    moduleResolution: ts.ModuleResolutionKind.Node16,
    types: [],
  });
  const source = program.getSourceFile(file)!;
  const checker = program.getTypeChecker();
  const rows: Row[] = [];
  function visit(node: ts.Node) {
    if (
      (ts.isVariableDeclaration(node) || ts.isTypeAliasDeclaration(node)) &&
      ts.isIdentifier(node.name) &&
      /^(Result_|WholeSlots_|WholeNoAny_|WholePayload_|WholePayloadAny_)/u.test(node.name.text)
    ) {
      const value = checker.getTypeAtLocation(node.name) as ts.Type & { intrinsicName?: string };
      rows.push({
        name: node.name.text,
        display: checker.typeToString(value, node, ts.TypeFormatFlags.NoTruncation),
        flags: value.flags,
        any: (value.flags & ts.TypeFlags.Any) !== 0,
        intrinsic: value.intrinsicName ?? null,
      });
    }
    ts.forEachChild(node, visit);
  }
  visit(source);
  assert.equal(rows.length, 40, "all five actual public observations for all eight shapes");
  return rows;
}
function expected(file: string, before: boolean, repaired = false) {
  const messages = before
    ? [
        [12, 44, 2344, "Type 'true' does not satisfy the constraint 'false'."],
        [24, 45, 2344, "Type 'true' does not satisfy the constraint 'false'."],
        [36, 44, 2344, "Type 'true' does not satisfy the constraint 'false'."],
        [48, 55, 2344, "Type 'true' does not satisfy the constraint 'false'."],
        [54, 54, 2344, "Type 'true' does not satisfy the constraint 'false'."],
      ]
    : [];
  if (!repaired) messages.push([55, 36, 2322, "Type 'number' is not assignable to type 'string'."]);
  return messages
    .map(
      ([line, column, code, message]) =>
        `${file}(${line},${column}): error TS${code}: ${message}\n`,
    )
    .join("");
}

/** Actual public import/constructor calls; no private Instance substituted into the consumer. */
export function verifyPublicSlotInstance(root: string) {
  assert.equal(ts.version, "6.0.3");
  const directory = mkdtempSync(path.join(tmpdir(), "vize-public-slot-instance-"));
  const compiler = path.join(root, "npm/cli/node_modules/typescript/bin/tsc");
  const historicalBytes = readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/slot-provider-payload-update-11027.json",
    ),
  );
  assert.equal(
    hash(historicalBytes),
    "0a71c5c21535e50d02df527e6c00267dfd8b44bfc0d63c37c49a55d67ab1166c",
  );
  const historical = JSON.parse(historicalBytes.toString("utf8")) as {
    files: Array<{ path: string; afterWholeUtf8: string }>;
  };
  const input = readFileSync(
    path.join(root, "tests/fixtures/typechecker/public-slot-instance/proof.ts.txt"),
    "utf8",
  );
  const observations: Array<{
    role: string;
    mode: string;
    rows: Row[];
    sdk: object[];
    process: object;
    repair: object | null;
  }> = [];
  try {
    for (const role of ["before", "head"] as const)
      for (const [mode, extension] of [
        ["esm", "mts"],
        ["commonjs", "cts"],
        ["direct", "ts"],
      ] as const) {
        const cwd = path.join(directory, role, mode),
          pkg = path.join(cwd, "node_modules/vize");
        mkdirSync(pkg, { recursive: true });
        const currentManifest = readFileSync(path.join(root, "npm/cli/package.json"), "utf8");
        const oldManifest = currentManifest.replace('\n    "virtual-ts-slots.d.ts",', "");
        assert.notEqual(oldManifest, currentManifest);
        assert.equal(
          hash(oldManifest),
          "6d4cddbdcd51ccb049a695688b5896294ad6aa23abb812f867e6bf6bd0b50d36",
        );
        writeFileSync(
          path.join(pkg, "package.json"),
          role === "before" ? oldManifest : currentManifest,
        );
        symlinkSync(path.join(root, "npm/cli/node_modules"), path.join(pkg, "node_modules"), "dir");
        const sdk = Object.entries(preimages).map(([file, sha256]) => {
          const current = readFileSync(path.join(root, "npm/cli", file), "utf8");
          const old = historical.files.find(
            (item) => item.path === "npm/cli/" + file,
          )!.afterWholeUtf8;
          assert.equal(hash(old), sha256, `exact actual-main preimage ${file}`);
          if (!file.startsWith("virtual-ts.d.")) assert.equal(current, old, `unchanged ${file}`);
          const body = role === "before" ? old : current;
          mkdirSync(path.dirname(path.join(pkg, file)), { recursive: true });
          writeFileSync(path.join(pkg, file), body);
          return { file, bytes: Buffer.byteLength(body), sha256: hash(body) };
        });
        if (role === "head") {
          const file = "virtual-ts-slots.d.ts",
            body = readFileSync(path.join(root, "npm/cli", file));
          writeFileSync(path.join(pkg, file), body);
          sdk.push({ file, bytes: body.length, sha256: hash(body) });
        }
        assert.equal(sdk.length, role === "before" ? 9 : 10);
        const file = `proof.${extension}`,
          target = path.join(cwd, file);
        const source =
          mode === "direct"
            ? input.replace('"vize/virtual-ts"', '"./node_modules/vize/virtual-ts.d.ts"')
            : input;
        writeFileSync(target, source);
        writeFileSync(
          path.join(cwd, "tsconfig.json"),
          JSON.stringify({
            compilerOptions: {
              strict: true,
              noEmit: true,
              skipLibCheck: true,
              module: "Node16",
              moduleResolution: "Node16",
              target: "ESNext",
              types: [],
            },
            files: [file],
          }),
        );
        const run = (repaired: boolean) => {
          const argv = [compiler, "--noEmit", "-p", "tsconfig.json", "--pretty", "false"];
          const output = spawnSync(process.execPath, argv, {
            cwd,
            encoding: "utf8",
            timeout: 60_000,
          });
          const wanted = expected(file, role === "before", repaired);
          const result = {
            executable: process.execPath,
            argv,
            cwd,
            status: output.status,
            signal: output.signal,
            error: output.error?.message ?? null,
            stdout: output.stdout,
            stderr: output.stderr,
            expectedStdout: wanted,
            sourceSha256: hash(readFileSync(target)),
            sourceUtf8: readFileSync(target, "utf8"),
            sdk,
          };
          assert.equal(output.error, undefined);
          assert.equal(output.signal, null);
          assert.equal(output.status, wanted === "" ? 0 : 2, JSON.stringify(result));
          assert.equal(output.stderr, "", JSON.stringify(result));
          assert.equal(output.stdout, wanted, JSON.stringify(result));
          return result;
        };
        const processResult = run(false),
          rows = typeRows(target);
        let repair: object | null = null;
        if (role === "head") {
          const repaired = source.replace(
            "Result_DefaultZero.$slots.default({value:123});",
            "Result_DefaultZero.$slots.default({value:'repaired'});",
          );
          assert.notEqual(repaired, source);
          writeFileSync(target, repaired);
          repair = run(true);
        }
        observations.push({ role, mode, sdk, process: processResult, repair, rows });
      }
    for (const role of ["before", "head"]) {
      const modes = observations.filter((item) => item.role === role);
      for (const item of modes.slice(1)) assert.deepEqual(item.rows, modes[0]!.rows);
    }
    const before = observations.find((item) => item.role === "before")!.rows;
    const head = observations.find((item) => item.role === "head")!.rows;
    assert.deepEqual(
      head.filter((row) => row.name.endsWith("_DefaultZero")),
      before.filter((row) => row.name.endsWith("_DefaultZero")),
      "ordinary default-zero Result, Slots, Payload and both any flags remain exact",
    );
    for (const name of unknownControls) {
      for (const rows of [before, head])
        assert.equal(rows.find((row) => row.name === "WholeNoAny_" + name)!.display, "false");
    }
    for (const name of failures) {
      assert.equal(before.find((row) => row.name === "WholeNoAny_" + name)!.display, "true");
      assert.equal(head.find((row) => row.name === "WholeNoAny_" + name)!.display, "false");
      for (const prefix of ["Result_", "WholeSlots_"]) {
        const row = head.find((row) => row.name === prefix + name)!;
        assert.equal(row.flags, ts.TypeFlags.Never, JSON.stringify(row));
        assert.equal(row.any, false);
      }
    }
    // Payload rows are complete observations. Do not invent a fix for the
    // independent unknown/payload-any boundary or arbitrary overload correlation.
    return {
      preimageSource,
      fixtureSha256: hash(input),
      typescript: ts.version,
      compilerSha256: hash(readFileSync(compiler)),
      observations,
    };
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}
