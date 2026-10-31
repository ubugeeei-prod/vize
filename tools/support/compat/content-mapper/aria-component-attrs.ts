/** Real packaged-declaration acceptance for #11144; source and installed releases. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { FrameDecoder } from "../../../../tests/tooling/support/lsp/installed-alias-replay/frames.ts";
import { verifyRawAriaContext } from "./aria-component-raw-context.ts";
import {
  ariaComponentProjects,
  ariaPropReference,
  ariaPropReferenceStdout,
  reportedAriaConfig,
} from "./aria-component-projects.ts";

const packageFiles = [
  "virtual-ts-globals.d.ts",
  "virtual-ts-patterns.d.ts",
  "virtual-ts.d.ts",
  "virtual-ts.d.mts",
  "virtual-ts.d.cts",
  "content-mapper-helpers.d.ts",
  "content-mapper-helpers.d.mts",
  "content-mapper-helpers.d.cts",
  "generated/content-mapper-types.d.ts",
];
const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const repo = fileURLToPath(new URL("../../../../", import.meta.url));
const options = new Map<string, string>();
for (let index = 2; index < process.argv.length; index += 2) {
  const name = process.argv[index],
    value = process.argv[index + 1];
  assert.ok(
    [
      "--consumer",
      "--output",
      "--mapper-host",
      "--mapper-version",
      "--candidate-cli",
      "--source-sha",
      "--source-package",
    ].includes(name),
  );
  assert.ok(value && !value.startsWith("--") && !options.has(name));
  options.set(name, value);
}
const required = (name: string) => {
  const value = options.get(name);
  assert.ok(value, `${name} required`);
  return value;
};
const consumer = fs.realpathSync(required("--consumer")),
  output = path.resolve(required("--output")),
  host = fs.realpathSync(required("--mapper-host")),
  candidate = options.has("--candidate-cli")
    ? fs.realpathSync(required("--candidate-cli"))
    : undefined;
assert.equal(options.has("--source-sha"), !!candidate);
assert.ok(
  !options.has("--source-package") || candidate,
  "installed mode cannot override package files",
);
if (candidate) assert.match(required("--source-sha"), /^[0-9a-f]{40}$/u);
assert.equal(fs.existsSync(output), false, "never overwrite an earlier observation");
fs.mkdirSync(output, { recursive: true });
const sourcePackage = candidate
  ? fs.realpathSync(options.get("--source-package") ?? path.join(repo, "npm/cli"))
  : path.join(consumer, "node_modules/vize");
const manifest = JSON.parse(fs.readFileSync(path.join(sourcePackage, "package.json"), "utf8"));
assert.equal(manifest.name, "vize");
for (const name of ["./virtual-ts", "./content-mapper-helpers"]) assert.ok(manifest.exports[name]);
const environment = Object.fromEntries(
  Object.entries(process.env).filter(
    ([name]) =>
      !/^(VIZE_|CORSA_|TSGO_|TSRS_|NAPI_RS_)/u.test(name) &&
      !["NODE_OPTIONS", "NODE_PATH", "CARGO_BIN_EXE_vize"].includes(name),
  ),
);
environment.PATH = `${path.dirname(process.execPath)}${path.delimiter}${environment.PATH ?? ""}`;
const write = (file: string, value: unknown) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
};
function run(argv: string[], cwd: string, directory: string, input?: Buffer) {
  const result = spawnSync(argv[0], argv.slice(1), {
    cwd,
    env: environment,
    input,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  const observed = {
    argv,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error
      ? Object.fromEntries(
          Object.getOwnPropertyNames(result.error).map((name) => [
            name,
            Reflect.get(result.error!, name),
          ]),
        )
      : null,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
  write(path.join(directory, "process.json"), observed);
  fs.writeFileSync(path.join(directory, "stdout.raw"), observed.stdout);
  fs.writeFileSync(path.join(directory, "stderr.raw"), observed.stderr);
  if (input) fs.writeFileSync(path.join(directory, "stdin.raw"), input);
  assert.equal(observed.error, null);
  assert.equal(observed.signal, null);
  return observed;
}
const version = run([host, "--version"], consumer, path.join(output, "host"));
assert.equal(version.status, 0);
assert.equal(version.stderr, "");
assert.equal(version.stdout, `${required("--mapper-version")}\n`);
if (candidate) {
  const source = run(
    ["git", "-C", sourcePackage, "rev-parse", "HEAD"],
    sourcePackage,
    path.join(output, "source-package"),
  );
  assert.equal(source.status, 0);
  assert.equal(source.stderr, "");
  assert.equal(
    source.stdout,
    `${required("--source-sha")}\n`,
    "declarations and source CLI must name the same exact revision",
  );
}
const declarations = packageFiles.map((file) => ({
  file,
  sha256: digest(fs.readFileSync(path.join(sourcePackage, file))),
}));
write(path.join(output, "providers.json"), {
  issue: 11144,
  mode: candidate ? "source-cli-with-shipped-declarations" : "unmodified-installed-package",
  source: options.get("--source-sha"),
  sourceCli: candidate,
  sourcePackage,
  sourceCliSha256: candidate ? digest(fs.readFileSync(candidate)) : undefined,
  package: {
    name: manifest.name,
    version: manifest.version,
    manifestSha256: digest(fs.readFileSync(path.join(sourcePackage, "package.json"))),
  },
  declarations,
  installedRuntimes: ["vue", "typescript", "tsrs"].flatMap((name) => {
    const file = path.join(consumer, "node_modules", name, "package.json");
    if (!fs.existsSync(file)) return [];
    const bytes = fs.readFileSync(file),
      installed = JSON.parse(bytes.toString());
    return [{ name: installed.name, version: installed.version, manifestSha256: digest(bytes) }];
  }),
  packageLockSha256: digest(fs.readFileSync(path.join(consumer, "package-lock.json"))),
  node: { path: process.execPath, sha256: digest(fs.readFileSync(process.execPath)) },
  host: { path: host, sha256: digest(fs.readFileSync(host)), version: version.stdout },
});
const results: { id: string; phase: string; passed: boolean; error?: string }[] = [];
const referenceDirectory = path.join(output, "independent-reference");
fs.mkdirSync(referenceDirectory);
fs.writeFileSync(path.join(referenceDirectory, "reference.ts"), ariaPropReference);
write(path.join(referenceDirectory, "tsconfig.json"), {
  compilerOptions: reportedAriaConfig.compilerOptions,
  files: ["reference.ts"],
});
write(path.join(referenceDirectory, "inputs.json"), {
  source: ariaPropReference,
  sourceSha256: digest(ariaPropReference),
  expectedStdout: ariaPropReferenceStdout,
  expectedStatus: 2,
});
const reference = run(
  [host, "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
  referenceDirectory,
  path.join(referenceDirectory, "compiler"),
);
assert.equal(reference.stderr, "");
assert.equal(
  reference.stdout,
  ariaPropReferenceStdout,
  "the same physical SDK must reject the handwritten owned prop",
);
assert.equal(reference.status, 2);
for (const project of ariaComponentProjects()) {
  const directory = path.join(output, "projects", project.id),
    modules = path.join(directory, "node_modules");
  fs.mkdirSync(path.join(directory, "src"), { recursive: true });
  fs.writeFileSync(path.join(directory, "src/child.vue"), project.child);
  write(path.join(directory, "tsconfig.json"), reportedAriaConfig);
  let packageRoot: string;
  if (candidate) {
    fs.mkdirSync(modules, { recursive: true });
    for (const entry of fs.readdirSync(path.join(consumer, "node_modules"))) {
      if (entry !== "vize" && entry !== ".bin")
        fs.symlinkSync(
          path.join(consumer, "node_modules", entry),
          path.join(modules, entry),
          "dir",
        );
    }
    packageRoot = path.join(modules, "vize");
    for (const file of packageFiles) {
      const target = path.join(packageRoot, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.copyFileSync(path.join(sourcePackage, file), target);
    }
    const sourceManifest = structuredClone(manifest);
    sourceManifest.typescript.contentMapper.exec = [candidate, "content-mapper"];
    write(path.join(packageRoot, "package.json"), sourceManifest);
    assert.deepEqual(
      sourceManifest.exports,
      manifest.exports,
      "retain actual package export conditions",
    );
  } else {
    fs.symlinkSync(path.join(consumer, "node_modules"), modules, "dir");
    packageRoot = fs.realpathSync(path.join(modules, "vize"));
  }
  const installed = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8"));
  const exec = installed.typescript.contentMapper.exec as string[];
  const command = exec.map((part, index) =>
    index === 0 && part === "node" ? process.execPath : part,
  );
  for (const phase of project.phases) {
    const capture = path.join(directory, phase.id);
    fs.writeFileSync(path.join(directory, "src/parent.vue"), phase.parent);
    write(path.join(capture, "inputs.json"), {
      config: reportedAriaConfig,
      files: {
        "src/parent.vue": { text: phase.parent, sha256: digest(phase.parent) },
        "src/child.vue": { text: project.child, sha256: digest(project.child) },
      },
      manifest: installed,
    });
    try {
      const requests = [
        {
          jsonrpc: "2.0",
          id: 1,
          method: "initialize",
          params: { positionEncodings: ["utf-8"], locale: "en-US" },
        },
        {
          jsonrpc: "2.0",
          id: 2,
          method: "openProject",
          params: {
            configFileName: path.join(directory, "tsconfig.json"),
            projectHandle: "aria",
            options: null,
            compilerOptions: reportedAriaConfig.compilerOptions,
          },
        },
        ...[
          ["child.vue", project.child],
          ["parent.vue", phase.parent],
        ].map(([file, content], index) => ({
          jsonrpc: "2.0",
          id: index + 3,
          method: "transform",
          params: { fileName: path.join(directory, "src", file), content, projectHandle: "aria" },
        })),
        { jsonrpc: "2.0", id: 100, method: "closeProject", params: { projectHandle: "aria" } },
      ];
      const frames = Buffer.concat(
        requests.map((request) => {
          const body = Buffer.from(JSON.stringify(request));
          return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
        }),
      );
      const protocol = run(command, packageRoot, path.join(capture, "protocol"), frames);
      const mapper = run(
        [host, "--runExternalCode", "--noEmit", "-p", "tsconfig.json", "--pretty", "false"],
        directory,
        path.join(capture, "mapper"),
      );
      assert.equal(protocol.status, 0);
      assert.equal(protocol.stderr, "");
      const decoder = new FrameDecoder(),
        packets: Record<string, unknown>[] = [];
      decoder.push(Buffer.from(protocol.stdout), (packet) => {
        assert.equal(packet.error, undefined);
        if (packet.id === 3 || packet.id === 4)
          packets.push(packet.result as Record<string, unknown>);
      });
      assert.equal(decoder.remaining, 0);
      assert.equal(packets.length, 2);
      write(path.join(capture, "transforms.json"), packets);
      for (const [index, packet] of packets.entries()) {
        assert.equal(typeof packet.text, "string");
        const text = packet.text as string;
        assert.equal(
          text.split('import type {} from "vize/virtual-ts";').length,
          2,
          "exercise the actual shipped shared declaration module",
        );
        assert.ok(!text.includes("vize:virtual-ts"));
        assert.ok(
          !text.includes("type $PropValue<"),
          "embedded helpers would hide the published regression",
        );
        assert.deepEqual(packet.diagnostics, []);
        assert.ok(Array.isArray(packet.mappings));
        const generated = Buffer.from(text),
          authored = Buffer.from(index === 0 ? project.child : phase.parent),
          utf8 = new TextDecoder("utf8", { fatal: true });
        for (const tuple of packet.mappings as number[][]) {
          assert.equal(tuple.length, 6);
          assert.ok(tuple.every((value) => Number.isSafeInteger(value) && value >= 0));
          assert.ok(
            tuple[0] + tuple[1] <= generated.length && tuple[2] + tuple[3] <= authored.length,
          );
          utf8.decode(generated.subarray(tuple[0], tuple[0] + tuple[1]));
          utf8.decode(authored.subarray(tuple[2], tuple[2] + tuple[3]));
        }
      }
      assert.equal(mapper.stderr, "");
      assert.equal(
        mapper.stdout,
        phase.expectedStdout,
        `${project.id}/${phase.id}: all authored diagnostics and no virtual errors`,
      );
      assert.equal(mapper.status, phase.expectedStatus);
      assert.equal(fs.readFileSync(path.join(directory, "src/parent.vue"), "utf8"), phase.parent);
      assert.equal(fs.readFileSync(path.join(directory, "src/child.vue"), "utf8"), project.child);
      results.push({ id: project.id, phase: phase.id, passed: true });
    } catch (error) {
      results.push({
        id: project.id,
        phase: phase.id,
        passed: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }
  fs.rmSync(modules, { recursive: true });
}
for (const declaration of declarations)
  assert.equal(
    digest(fs.readFileSync(path.join(sourcePackage, declaration.file))),
    declaration.sha256,
    "the installed or shipped declaration bytes must remain unchanged",
  );
let rawContext: { passed: boolean; error?: string };
try {
  rawContext = verifyRawAriaContext(output, host, run);
} catch (error) {
  rawContext = { passed: false, error: error instanceof Error ? error.message : String(error) };
}
const report = {
  issue: 11144,
  completeCases: results.length,
  passed: results.filter((result) => result.passed).length,
  source: options.get("--source-sha"),
  installedVersion: candidate ? undefined : manifest.version,
  independentPropReference: true,
  rawContext,
  results,
};
write(path.join(output, "report.json"), report);
console.log(JSON.stringify(report));
assert.equal(
  report.passed,
  report.completeCases,
  "every original, native and invalid/repair package control must pass",
);
assert.equal(rawContext.passed, true, "raw three-argument context must retain every native error");
