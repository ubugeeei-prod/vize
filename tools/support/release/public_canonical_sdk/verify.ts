/** Genuine published mapper/helper qualification; the original CPP consumer stays untouched. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  authenticate,
  array,
  image,
  read,
  record,
  run,
  save,
  sha,
  string,
  type Image,
} from "./io.ts";
import { transform } from "./protocol.ts";
type Phase = {
  id: string;
  source: string;
  code: number | null;
  snippet: string;
  token: string;
  message: string;
};
function point(
  text: string,
  snippet: string,
  token: string,
): { line: number; column: number; byte: number } {
  const start = text.indexOf(snippet);
  assert.ok(start >= 0 && text.indexOf(snippet, start + 1) < 0);
  const at = start + snippet.indexOf(token);
  assert.ok(snippet.includes(token));
  const rows = text.slice(0, at).split(/\r\n|[\r\n\u2028\u2029]/u),
    tail = rows.at(-1);
  assert.ok(tail !== undefined);
  return { line: rows.length, column: tail.length + 1, byte: Buffer.byteLength(text.slice(0, at)) };
}
function snapshot(workspace: string): Image[] {
  return [
    "package.json",
    "package-lock.json",
    "tsconfig.json",
    "acceptance.json",
    "installed-image-manifest.json",
    ...fs
      .readdirSync(path.join(workspace, "src"), { withFileTypes: true })
      .filter((entry) => entry.isFile())
      .map((entry) => "src/" + entry.name),
  ].map((file) => image(path.join(workspace, file)));
}
export async function main(args: string[]): Promise<void> {
  const [workspaceArg, version, outputArg] = args;
  assert.ok(workspaceArg && version && outputArg && args.length === 3);
  assert.match(version, /^\d+\.\d+\.\d+$/u);
  const workspace = fs.realpathSync(workspaceArg),
    output = path.resolve(outputArg);
  assert.equal(fs.existsSync(output), false);
  const protectedBefore = snapshot(workspace),
    installed = authenticate(workspace, version);
  const fixtures = fileURLToPath(new URL("./fixtures/", import.meta.url));
  const manifest = record(read(path.join(fixtures, "manifest.json.txt"))),
    sources: Record<string, string> = {};
  for (const entry of array(manifest.files)) {
    const file = record(entry),
      name = string(file.file),
      bytes = fs.readFileSync(path.join(fixtures, name));
    assert.equal(bytes.length, file.bytes);
    assert.equal(sha(bytes), file.sha256);
    sources[name] = bytes.toString("utf8");
  }
  const source = sources["Probe.vue.txt"],
    expectedImport = sources["core-import.txt"],
    mts = sources["Witness.mts.txt"],
    cts = sources["Witness.cts.txt"];
  assert.ok(source && expectedImport && mts && cts);
  assert.equal([...expectedImport.matchAll(/^\s+__[A-Za-z_0-9]+,$/gmu)].length, 53);
  assert.equal(
    image(path.join(installed.root, "generated/content-mapper-types.d.ts")).sha256,
    manifest.publicCoreSha256,
  );
  assert.equal(
    image(path.join(installed.root, "content-mapper-helpers.d.ts")).sha256,
    manifest.publicEntrySha256,
  );
  const helper = fs.readFileSync(path.join(installed.root, "content-mapper-helpers.d.ts"), "utf8");
  assert.ok(
    helper
      .split("\n")
      .includes(
        'export type * from "./generated/content-mapper-types.js" with { "resolution-mode": "import" };',
      ),
  );
  const project = fs.mkdtempSync(path.join(path.dirname(workspace), "public-canonical-sdk-")),
    capture = output.replace(/\.json$/u, "") + "-evidence";
  assert.equal(fs.existsSync(capture), false);
  fs.mkdirSync(capture, { recursive: true });
  fs.mkdirSync(path.join(project, "node_modules"));
  fs.symlinkSync(
    path.join(workspace, "node_modules/vize"),
    path.join(project, "node_modules/vize"),
    "dir",
  );
  fs.symlinkSync(
    path.join(workspace, "node_modules/vue"),
    path.join(project, "node_modules/vue"),
    "dir",
  );
  fs.symlinkSync(
    path.join(workspace, "node_modules/@vue"),
    path.join(project, "node_modules/@vue"),
    "dir",
  );
  const compilerOptions = {
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    target: "ESNext",
    module: "NodeNext",
    moduleResolution: "NodeNext",
    jsx: "preserve",
    lib: ["esnext", "dom"],
    types: [],
  };
  save(path.join(project, "package.json"), {
    name: "public-canonical-sdk-probe",
    private: true,
    type: "module",
  });
  save(path.join(project, "tsconfig.json"), {
    compilerOptions,
    files: ["Probe.vue"],
    contentMappers: [{ package: "vize", extensions: [".vue"] }],
  });
  save(path.join(project, "tsconfig.generated.json"), {
    compilerOptions,
    files: ["Probe.virtual.tsx", "Witness.mts", "Witness.cts"],
  });
  fs.writeFileSync(path.join(project, "Witness.mts"), mts);
  fs.writeFileSync(path.join(project, "Witness.cts"), cts);
  for (const file of [
    "package.json",
    "tsconfig.json",
    "tsconfig.generated.json",
    "Witness.mts",
    "Witness.cts",
  ])
    fs.copyFileSync(path.join(project, file), path.join(capture, file + ".txt"));
  save(path.join(capture, "immutable-input-manifest.json"), manifest);
  const cliVersion = run(
    capture,
    "version/cli",
    process.execPath,
    [installed.cli, "--version"],
    project,
  );
  assert.equal(cliVersion.status, 0);
  assert.equal(cliVersion.stdout.toString().trim(), "vize " + version);
  const nativeVersion = run(
    capture,
    "version/typescript",
    installed.provider,
    ["--version"],
    project,
  );
  assert.equal(nativeVersion.status, 0);
  assert.equal(nativeVersion.stdout.toString().trim(), "Version 7.0.2");
  const phases: Phase[] = [
    { id: "valid", source, code: null, snippet: "", token: "", message: "" },
    {
      id: "wrong-value",
      source: source.replace("const exact: number", "const exact: string"),
      code: 2322,
      snippet: "const exact: string = props.value;",
      token: "exact",
      message: "Type 'number' is not assignable to type 'string'.",
    },
    { id: "value-repaired", source, code: null, snippet: "", token: "", message: "" },
    {
      id: "readonly-violation",
      source: source.replace("const exact:", "props.value = 2;\nconst exact:"),
      code: 2540,
      snippet: "props.value = 2;",
      token: "value",
      message: "Cannot assign to 'value' because it is a read-only property.",
    },
    { id: "readonly-repaired", source, code: null, snippet: "", token: "", message: "" },
  ];
  const observations = [];
  for (const phase of phases) {
    fs.writeFileSync(path.join(project, "Probe.vue"), phase.source);
    const transformed = transform(
      installed.cli,
      project,
      capture,
      phase.id,
      phase.source,
      compilerOptions,
      expectedImport,
    );
    fs.writeFileSync(path.join(project, "Probe.virtual" + transformed.extension), transformed.text);
    const result = run(
      capture,
      phase.id + "/compiler",
      installed.provider,
      [
        "--checkers",
        "1",
        "--pretty",
        "false",
        "--project",
        path.join(project, "tsconfig.generated.json"),
      ],
      project,
    );
    const diagnostics = [
      ...result.stdout
        .toString("utf8")
        .matchAll(/^(.+)\((\d+),(\d+)\): error TS(\d+): ([^\r\n]*)/gmu),
    ].map((match) => {
      const file = string(match[1]),
        line = Number(string(match[2])),
        column = Number(string(match[3])),
        code = Number(string(match[4])),
        message = string(match[5]);
      return {
        file: path.relative(project, path.resolve(project, file)),
        line,
        column,
        code,
        message,
      };
    });
    if (phase.code === null) {
      assert.equal(result.status, 0, result.stdout.toString() + result.stderr.toString());
      assert.equal(diagnostics.length, 0);
      assert.equal(result.stdout.length, 0);
    } else {
      assert.notEqual(result.status, 0);
      const location = point(transformed.text, phase.snippet, phase.token),
        authored = point(phase.source, phase.snippet, phase.token);
      assert.deepEqual(diagnostics, [
        {
          file: "Probe.virtual.tsx",
          line: location.line,
          column: location.column,
          code: phase.code,
          message: phase.message,
        },
      ]);
      assert.ok(
        transformed.mappings.some((span) => {
          const [g, gl, a, al, kind] = span;
          return (
            g !== undefined &&
            gl !== undefined &&
            a !== undefined &&
            al !== undefined &&
            kind === 0 &&
            location.byte >= g &&
            location.byte < g + gl &&
            authored.byte >= a &&
            authored.byte < a + al &&
            location.byte - g === authored.byte - a
          );
        }),
        "the real error must map exactly to the owned authored token",
      );
    }
    observations.push({
      id: phase.id,
      sourceSha256: sha(phase.source),
      virtualTsSha256: sha(transformed.text),
      diagnostics,
      completePacket: transformed.packet,
    });
  }
  assert.equal(observations.length, 5);
  assert.deepEqual(
    snapshot(workspace),
    protectedBefore,
    "the CPP consumer/source/lock must remain unchanged",
  );
  save(output, {
    schema: "vize.publicCanonicalSdk.v1",
    version,
    status: "pass",
    qualification:
      "genuine published mapper RPC and unchanged returned VTS checked by public bundled native TypeScript; not an external TypeScript 7.1 host qualification",
    aliases: 53,
    native: installed.native,
    installed: installed.metadata,
    cppReleaseIdentity: installed.cpp.releaseIdentity,
    originalCppProtected: protectedBefore,
    project,
    capture,
    immutableInputManifest: manifest,
    observations,
    scope: "correctness and actual public capability; no timing/cache/performance claim",
  });
}
await main(process.argv.slice(2));
