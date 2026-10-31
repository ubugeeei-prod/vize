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
const inputBytes = (utf8: string) => ({ utf8, bytes: Buffer.byteLength(utf8), sha256: hash(utf8) });
type Row = { name: string; display: string; flags: number; any: boolean; intrinsic: string | null };
function typeRows(file: string, preservation = false) {
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
      (preservation
        ? /^Preserve_/u
        : /^(Result_|WholeSlots_|WholeNoAny_|WholePayload_|WholePayloadAny_)/u
      ).test(node.name.text)
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
  assert.equal(rows.length, preservation ? 60 : 40, "complete public type observation roster");
  return rows;
}

function preservationSource(entry: string) {
  const lines = [
    `import type { $SlotConstructor, $SlotConstructorWithoutArguments, $SlotConstructorWithArguments, $SlotConstructorArgs, $SlotConstructorInstance } from ${JSON.stringify(entry)};`,
    "type Typed<T> = { readonly $props: { value: T }; $slots: { default(props: { item: T }): void } };",
    "declare function instance<C>(): <Z, P>(zero: Z, props: P) => $SlotConstructorInstance<C, Z, P>;",
  ];
  for (const count of [2, 8, 9, 17])
    for (const order of ["first", "last"]) {
      const name = `${count}_${order}`;
      const signatures = Array.from(
        { length: count - 1 },
        (_, i) => `new(props: { kind: 'brand${i}'; value: number }): Typed<number>;`,
      );
      signatures.splice(order === "first" ? 0 : signatures.length, 0, "new(): Typed<string>;");
      lines.push(
        `declare const Original_${name}: { ${signatures.join(" ")} };`,
        `type Preserve_C_${name} = $SlotConstructor<typeof Original_${name}>;`,
        `type Preserve_Without_${name} = $SlotConstructorWithoutArguments<Preserve_C_${name}>;`,
        `type Preserve_With_${name} = $SlotConstructorWithArguments<Preserve_C_${name}>;`,
        `type Preserve_Args_${name} = $SlotConstructorArgs<Preserve_C_${name}>;`,
        `const zero_${name} = new(undefined as unknown as Preserve_Without_${name})();`,
        `const props_${name} = new(undefined as unknown as Preserve_With_${name})({ kind: 'brand0' as const, value: 123 }, ...([] as unknown as Preserve_Args_${name}));`,
        `const Preserve_Result_${name} = instance<Preserve_C_${name}>()(zero_${name}, props_${name});`,
      );
    }
  for (const poison of ["any", "unknown", "never", "string"]) {
    lines.push(
      `declare const Poison_${poison}: { new(): never; new(props: { value: number }): { $slots: ${poison} } };`,
      `type Preserve_PoisonC_${poison} = $SlotConstructor<typeof Poison_${poison}>;`,
      `type Preserve_PoisonWithout_${poison} = $SlotConstructorWithoutArguments<Preserve_PoisonC_${poison}>;`,
      `type Preserve_PoisonWith_${poison} = $SlotConstructorWithArguments<Preserve_PoisonC_${poison}>;`,
      `type Preserve_PoisonArgs_${poison} = $SlotConstructorArgs<Preserve_PoisonC_${poison}>;`,
      `type Preserve_PoisonHost_${poison} = typeof Poison_${poison};`,
    );
  }
  lines.push('const wrong: number = "owned-negative";');
  return lines.join("\n") + "\n";
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
  const hosts = [
    {
      name: "typescript-6.0.3",
      executable: process.execPath,
      prefix: [compiler],
      sha256: hash(readFileSync(compiler)),
      version: null as object | null,
    },
  ];
  const native = process.env.VIZE_PUBLIC_SLOT_NATIVE;
  if (native !== undefined) {
    assert.ok(path.isAbsolute(native));
    const version = spawnSync(native, ["--version"], { encoding: "utf8", timeout: 30_000 });
    assert.equal(version.error, undefined);
    assert.equal(version.signal, null);
    assert.equal(version.status, 0);
    assert.equal(version.stderr, "");
    assert.equal(version.stdout, "Version 7.1.0-beta\n");
    hosts.push({
      name: "native-7.1.0-beta",
      executable: native,
      prefix: [],
      sha256: hash(readFileSync(native)),
      version: {
        argv: ["--version"],
        status: version.status,
        stdout: version.stdout,
        stderr: version.stderr,
      },
    });
  }
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
    consumerPackage: object;
    sdkPackage: object;
    processes: object[];
    repair: object | null;
    preservation: { sourceUtf8: string; rows: Row[]; processes: object[]; repair: object[] };
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
        const consumerPackage = inputBytes(JSON.stringify({ private: true, type: "module" }));
        writeFileSync(path.join(cwd, "package.json"), consumerPackage.utf8);
        const currentManifest = readFileSync(path.join(root, "npm/cli/package.json"), "utf8");
        const oldManifest = currentManifest.replace('\n    "virtual-ts-slots.d.ts",', "");
        assert.notEqual(oldManifest, currentManifest);
        assert.equal(
          hash(oldManifest),
          "6d4cddbdcd51ccb049a695688b5896294ad6aa23abb812f867e6bf6bd0b50d36",
        );
        const sdkPackage = inputBytes(role === "before" ? oldManifest : currentManifest);
        writeFileSync(path.join(pkg, "package.json"), sdkPackage.utf8);
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
        const run = (repaired: boolean) =>
          hosts.map((host) => {
            const argv = [...host.prefix, "--noEmit", "-p", "tsconfig.json", "--pretty", "false"];
            const output = spawnSync(host.executable, argv, {
              cwd,
              encoding: "utf8",
              timeout: 60_000,
            });
            const wanted = expected(file, role === "before", repaired);
            const result = {
              host,
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
              consumerPackage,
              sdkPackage,
            };
            assert.equal(output.error, undefined);
            assert.equal(output.signal, null);
            assert.equal(output.status, wanted === "" ? 0 : 2, JSON.stringify(result));
            assert.equal(output.stderr, "", JSON.stringify(result));
            assert.equal(output.stdout, wanted, JSON.stringify(result));
            return result;
          });
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
        const preservation = preservationSource(
          mode === "direct" ? "./node_modules/vize/virtual-ts.d.ts" : "vize/virtual-ts",
        );
        writeFileSync(target, preservation);
        const preservationRows = typeRows(target, true);
        const preserveRun = (repaired: boolean) =>
          hosts.map((host) => {
            const argv = [...host.prefix, "--noEmit", "-p", "tsconfig.json", "--pretty", "false"];
            const output = spawnSync(host.executable, argv, {
              cwd,
              encoding: "utf8",
              timeout: 60_000,
            });
            const result = {
              host,
              argv,
              cwd,
              sourceUtf8: readFileSync(target, "utf8"),
              status: output.status,
              signal: output.signal,
              error: output.error?.message ?? null,
              stdout: output.stdout,
              stderr: output.stderr,
            };
            assert.equal(output.error, undefined, JSON.stringify(result));
            assert.equal(output.signal, null, JSON.stringify(result));
            assert.equal(output.stderr, "", JSON.stringify(result));
            assert.equal(output.status, repaired ? 0 : 2, JSON.stringify(result));
            assert.equal(
              output.stdout,
              repaired
                ? ""
                : `${file}(${preservation.split("\n").length - 1},7): error TS2322: Type 'string' is not assignable to type 'number'.\n`,
              JSON.stringify(result),
            );
            return result;
          });
        const preserved = preserveRun(false);
        writeFileSync(
          target,
          preservation.replace(
            'const wrong: number = "owned-negative";',
            "const wrong: number = 123;",
          ),
        );
        const preservationRepair = preserveRun(true);
        observations.push({
          role,
          mode,
          sdk,
          consumerPackage,
          sdkPackage,
          processes: processResult,
          repair,
          rows,
          preservation: {
            sourceUtf8: preservation,
            rows: preservationRows,
            processes: preserved,
            repair: preservationRepair,
          },
        });
        if (process.env.VIZE_PUBLIC_SLOT_RECEIPT) {
          writeFileSync(
            process.env.VIZE_PUBLIC_SLOT_RECEIPT,
            JSON.stringify(
              { passed: false, preimageSource, fixtureSha256: hash(input), hosts, observations },
              null,
              2,
            ) + "\n",
          );
        }
      }
    for (const role of ["before", "head"]) {
      const modes = observations.filter((item) => item.role === role);
      for (const item of modes.slice(1)) assert.deepEqual(item.rows, modes[0]!.rows);
    }
    const before = observations.find((item) => item.role === "before")!.rows;
    const head = observations.find((item) => item.role === "head")!.rows;
    for (const mode of ["esm", "commonjs", "direct"]) {
      const before = observations.find(
        (item) => item.role === "before" && item.mode === mode,
      )!.preservation;
      const head = observations.find(
        (item) => item.role === "head" && item.mode === mode,
      )!.preservation;
      assert.equal(head.sourceUtf8, before.sourceUtf8);
      assert.deepEqual(
        head.rows,
        before.rows,
        "2/8/9/17 signatures in both orders and four poison adapters remain exact; no rare-correlation repair claimed",
      );
    }
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
      passed: true,
      preimageSource,
      fixtureSha256: hash(input),
      typescript: ts.version,
      compilerSha256: hash(readFileSync(compiler)),
      hosts,
      observations,
    };
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}
