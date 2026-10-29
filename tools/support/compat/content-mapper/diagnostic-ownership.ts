/** Actual published-host/source-CLI observations for component-input ownership. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { resolveOfficialNativeImage } from "./official-fixtures.ts";
import { callbackOwnerProjects, callbackSetupReference } from "./callback-owner-projects.ts";
import {
  directiveControls,
  formDirectiveControls,
  relocationControls,
  type Diagnostic,
  type Project,
} from "./diagnostic-ownership-projects.ts";
type Observation = {
  argv: string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: Record<string, unknown> | null;
  stdout: string;
  stderr: string;
};
const repository = fileURLToPath(new URL("../../../../", import.meta.url));
const packRoot = path.join(
  repository,
  "tests/_fixtures/differential/typechecker/component-input-diagnostic-owners-10607",
);
const packBytes = fs.readFileSync(path.join(packRoot, "cases.json"));
const pack = JSON.parse(packBytes.toString()) as { cases: Project[] };
const slotBase = pack.cases.find((project) => project.id === "slot-prop-named-child-invalid");
assert.ok(slotBase);
const formBase = pack.cases.find((project) => project.id === "art-generic-form-invalid");
assert.ok(formBase);
const ownershipProjects = [
  ...pack.cases,
  ...directiveControls(slotBase),
  ...formDirectiveControls(formBase),
];
const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const args = process.argv.slice(2),
  options = new Map<string, string>();
let baselineOnly = false;
let relocationOnly = false;
let callbackOnly = false;
for (let index = 0; index < args.length; index++) {
  const key = args.at(index);
  assert.ok(key);
  if (key === "--baseline-only") {
    assert.equal(baselineOnly, false);
    baselineOnly = true;
    continue;
  }
  if (key === "--relocation-controls") {
    assert.equal(relocationOnly, false);
    relocationOnly = true;
    continue;
  }
  if (key === "--callback-cfa-controls") {
    assert.equal(callbackOnly, false);
    callbackOnly = true;
    continue;
  }
  assert.ok(
    [
      "--consumer",
      "--candidate-cli",
      "--output",
      "--source-sha",
      "--mapper-host",
      "--mapper-version",
      "--comparison-cli",
      "--comparison-source-sha",
    ].includes(key),
  );
  const value = args.at(++index);
  assert.ok(value && !value.startsWith("--"));
  assert.equal(options.has(key), false);
  options.set(key, value);
}
const required = (key: string) => {
  const value = options.get(key);
  assert.ok(value, `${key} required`);
  return value;
};
const consumer = fs.realpathSync(required("--consumer"));
const output = path.resolve(required("--output"));
assert.equal(fs.existsSync(output), false, "never overwrite an earlier capture");
fs.mkdirSync(output, { recursive: true });
const candidate = baselineOnly ? undefined : fs.realpathSync(required("--candidate-cli"));
const sourceSha = options.get("--source-sha");
if (!baselineOnly) assert.match(sourceSha ?? "", /^[0-9a-f]{40}$/u);
assert.equal(options.has("--comparison-cli"), options.has("--comparison-source-sha"));
if (options.has("--comparison-cli")) {
  assert.ok((relocationOnly || callbackOnly) && !baselineOnly);
  assert.equal(
    required("--comparison-source-sha"),
    callbackOnly
      ? "78719553cf550bcffbf1d0fb891fda642f42de3a"
      : "5597001751a477402537a1cc61cae487feefef9c",
  );
}
const comparisonCli = options.has("--comparison-cli")
  ? fs.realpathSync(required("--comparison-cli"))
  : undefined;
assert.ok(!(relocationOnly && callbackOnly));
if (callbackOnly) assert.ok(comparisonCli && !baselineOnly);
const callbackProjects = callbackOnly ? callbackOwnerProjects() : [];
const projects: Project[] = callbackOnly
  ? callbackProjects.map((project) => ({
      id: project.id,
      inputs: [],
      inlineInputs: project.inlineInputs,
      diagnostics: [],
    }))
  : relocationOnly
    ? relocationControls()
    : ownershipProjects;
const environment = Object.fromEntries(
  Object.entries(process.env).filter(
    ([key]) =>
      !/^(VIZE_|CORSA_|TSGO_|TSRS_|NAPI_RS_)/u.test(key) &&
      !["NODE_OPTIONS", "NODE_PATH", "CARGO_BIN_EXE_vize"].includes(key),
  ),
);
environment.PATH = `${path.dirname(process.execPath)}${path.delimiter}${environment.PATH ?? ""}`;
const write = (name: string, value: unknown) => {
  fs.mkdirSync(path.dirname(name), { recursive: true });
  fs.writeFileSync(name, JSON.stringify(value, null, 2) + "\n");
};
function run(argv: string[], cwd: string, destination: string, input?: Buffer): Observation {
  const command = argv.at(0);
  assert.ok(command);
  const result = spawnSync(command, argv.slice(1), {
    cwd,
    env: environment,
    encoding: "utf8",
    input,
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  const observation: Observation = {
    argv,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error
      ? Object.fromEntries(
          Object.getOwnPropertyNames(result.error).map((key) => [
            key,
            Reflect.get(result.error!, key),
          ]),
        )
      : null,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
  fs.mkdirSync(destination, { recursive: true });
  fs.writeFileSync(path.join(destination, "stdout.raw"), observation.stdout);
  fs.writeFileSync(path.join(destination, "stderr.raw"), observation.stderr);
  if (input) fs.writeFileSync(path.join(destination, "stdin.raw"), input);
  write(path.join(destination, "process.json"), observation);
  return observation;
}
function successful(observation: Observation) {
  assert.equal(observation.status, 0);
  assert.equal(observation.signal, null);
  assert.equal(observation.error, null);
  assert.equal(observation.stderr, "");
}
const readManifest = (name: string) =>
  JSON.parse(
    fs.readFileSync(path.join(consumer, "node_modules", name, "package.json"), "utf8"),
  ) as { name: string; version: string };
assert.equal(readManifest("vize").version, "0.566.0");
assert.equal(readManifest("typescript").version, "7.1.0-dev.20261024.1");
assert.equal(readManifest("vue").version, "3.5.42");
const typescriptRoot = path.join(consumer, "node_modules/typescript");
const installedOfficial = fs.realpathSync(
  (await import(pathToFileURL(path.join(typescriptRoot, "lib/getExePath.js")).href)).default(),
);
assert.equal(options.has("--mapper-host"), options.has("--mapper-version"));
const official = options.has("--mapper-host")
  ? fs.realpathSync(required("--mapper-host"))
  : installedOfficial;
const officialVersion = options.has("--mapper-version")
  ? `${required("--mapper-version")}\n`
  : "Version 7.1.0-dev.20261024.1\n";
const nativePackage = `@typescript/typescript-${process.platform}-${process.arch}`;
const nativeRoot = path.dirname(
  createRequire(path.join(consumer, "node_modules/vize/package.json")).resolve(
    `${nativePackage}/package.json`,
  ),
);
assert.equal(
  JSON.parse(fs.readFileSync(path.join(nativeRoot, "package.json"), "utf8")).version,
  "7.0.2",
);
const bundled = fs.realpathSync(path.join(nativeRoot, "lib/tsc"));
const baselineCli = fs.realpathSync(path.join(consumer, "node_modules/vize/bin/vize"));
const nativeImage = resolveOfficialNativeImage(consumer);
for (const file of [installedOfficial, bundled, baselineCli, nativeImage])
  assert.ok(file.startsWith(consumer + path.sep));
const versions = [
  ["official", official, officialVersion],
  ["bundled", bundled, "Version 7.0.2\n"],
] as const;
for (const [id, file, expected] of versions) {
  const v = run([file, "--version"], consumer, path.join(output, "providers", id));
  successful(v);
  assert.equal(v.stdout, expected);
}
write(path.join(output, "providers.json"), {
  sourceSha,
  comparisonSourceSha: options.get("--comparison-source-sha"),
  mode: callbackOnly
    ? "callback-cfa-control-observations-not-qualified"
    : relocationOnly
      ? "relocation-control-observations-not-qualified"
      : baselineOnly
        ? "registry-baseline-only"
        : "source-cli-qualification",
  notPublicationAcceptance: true,
  explicitMapperHost: options.has("--mapper-host"),
  mapperVersion: officialVersion,
  packageLockSha256: digest(fs.readFileSync(path.join(consumer, "package-lock.json"))),
  packSha256: digest(packBytes),
  packages: ["vize", "typescript", "vue", "@vizejs/native"].map(readManifest),
  binaries: [
    official,
    bundled,
    baselineCli,
    nativeImage,
    ...(candidate ? [candidate] : []),
    ...(comparisonCli ? [comparisonCli] : []),
  ].map((file) => ({ path: file, sha256: digest(fs.readFileSync(file)) })),
  mapperIdentityCustody:
    "installed manifest/resolution and separate native-image probe; not a raw provider RPC witness",
});
const header = "<!-- 日本 🧭 -->";
const rows: {
  id: string;
  files: Record<string, string>;
  diagnostics: Diagnostic[];
  mapperStdout?: string;
  observations: Record<string, Observation>;
}[] = [];
for (const project of projects)
  for (const newline of ["\n", "\r\n"])
    for (const prefix of ["", header]) {
      const id = `${project.id}-${newline === "\n" ? "lf" : "crlf"}-${prefix ? "unicode" : "plain"}`;
      const files: Record<string, string> = {};
      for (const entry of project.inlineInputs
        ? Object.entries(project.inlineInputs).map(([file, text]) => ({ file, text }))
        : project.inputs.map(({ file, source }) => ({
            file,
            text: fs.readFileSync(path.join(packRoot, source), "utf8"),
          }))) {
        let text = entry.text;
        if (
          (entry.file === "src/Min.vue" || entry.file === "src/Form.art.vue") &&
          project.replaceParent
        )
          text = project.replaceParent(text);
        if (
          entry.file.endsWith(".vue") &&
          !entry.file.endsWith("GenericForm.vue") &&
          !entry.file.endsWith("Child3.vue")
        )
          text = text
            .replaceAll("  <GenericForm", `  ${prefix}<GenericForm`)
            .replaceAll("  <Child3", `  ${prefix}<Child3`);
        if ((relocationOnly || callbackOnly) && entry.file === "src/Min.vue")
          text = text.replaceAll("  <Comp", `  ${prefix}<Comp`);
        const file =
          project.ordinaryForm && entry.file === "src/Form.art.vue" ? "src/Form.vue" : entry.file;
        files[file] = text.replaceAll("\n", newline);
      }
      const config = JSON.parse(files["tsconfig.json"]);
      if (project.ordinaryForm)
        config.files = config.files.map((file: string) =>
          file === "src/Form.art.vue" ? "src/Form.vue" : file,
        );
      config.contentMappers = [{ package: "vize", extensions: [".vue"] }];
      files["tsconfig.json"] = JSON.stringify(config, null, 2) + "\n";
      const diagnostics = project.diagnostics.map((row) => {
        const line = files[row.file].split(/\r?\n/u).at(row.line - 1);
        assert.ok(line);
        return { ...row, column: row.column + (line.includes(header) ? prefix.length : 0) };
      });
      const observations: Record<string, Observation> = {};
      if (callbackOnly) {
        const control = callbackProjects.find((control) => control.id === project.id);
        assert.ok(control);
        const directory = path.join(output, "projects", id, "independent-reference");
        fs.mkdirSync(directory, { recursive: true });
        const source =
          `${prefix ? "// 日本 🧭\n" : ""}${callbackSetupReference(control.reference.source)}`.replaceAll(
            "\n",
            newline,
          );
        fs.writeFileSync(path.join(directory, control.reference.file), source);
        write(path.join(directory, "tsconfig.json"), {
          compilerOptions: config.compilerOptions,
          files: [control.reference.file],
        });
        write(path.join(directory, "control.json"), {
          control,
          source,
          sourceSha256: digest(source),
          originalModuleSource: control.reference.source,
          originalModuleSourceSha256: digest(control.reference.source),
          lexicalContext: "handwritten setup function, preserving original module input",
          newline,
          prefix,
          qualification:
            "independent same-physical-SDK TypeScript; complete source observations still require audit",
        });
        observations.reference = run(
          [official, "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
          directory,
          path.join(directory, "compiler"),
        );
        observations["native-reference"] = run(
          [bundled, "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
          directory,
          path.join(directory, "native-compiler"),
        );
      }
      for (const side of candidate ? ["baseline", "candidate"] : ["baseline"]) {
        const directory = path.join(output, "projects", id, side);
        fs.mkdirSync(directory, { recursive: true });
        for (const [file, text] of Object.entries(files)) {
          const destination = path.join(directory, file);
          fs.mkdirSync(path.dirname(destination), { recursive: true });
          fs.writeFileSync(destination, text);
        }
        const nodeModules = path.join(directory, "node_modules");
        if (side === "baseline" && !comparisonCli)
          fs.symlinkSync(path.join(consumer, "node_modules"), nodeModules, "dir");
        else {
          fs.mkdirSync(path.join(nodeModules, "vize"), { recursive: true });
          fs.symlinkSync(
            path.join(consumer, "node_modules/vue"),
            path.join(nodeModules, "vue"),
            "dir",
          );
          write(path.join(nodeModules, "vize/package.json"), {
            name: "vize",
            version: `0.0.0-source-${side === "baseline" ? options.get("--comparison-source-sha") : sourceSha}`,
            private: true,
            typescript: {
              contentMapper: {
                exec: [side === "baseline" ? comparisonCli : candidate, "content-mapper"],
                dynamicConfig: true,
                compilerOptions: ["checkJs", "noUnusedLocals"],
              },
            },
          });
        }
        write(path.join(directory, "inputs.json"), {
          files: Object.fromEntries(
            Object.entries(files).map(([file, text]) => [
              file,
              { bytes: [...Buffer.from(text)], sha256: digest(text) },
            ]),
          ),
        });
        const mapperCommand =
          side === "baseline"
            ? comparisonCli
              ? [comparisonCli]
              : [process.execPath, baselineCli]
            : [candidate!];
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
              projectHandle: "ownership",
              options: null,
              compilerOptions: config.compilerOptions,
            },
          },
          ...Object.entries(files)
            .filter(([file]) => file.endsWith(".vue"))
            .map(([file, content], index) => ({
              jsonrpc: "2.0",
              id: index + 3,
              method: "transform",
              params: { fileName: path.join(directory, file), content, projectHandle: "ownership" },
            })),
          {
            jsonrpc: "2.0",
            id: 100,
            method: "closeProject",
            params: { projectHandle: "ownership" },
          },
        ];
        const frames = Buffer.concat(
          requests.map((request) => {
            const bytes = Buffer.from(JSON.stringify(request));
            return Buffer.concat([Buffer.from(`Content-Length: ${bytes.length}\r\n\r\n`), bytes]);
          }),
        );
        observations[`${side}-protocol`] = run(
          [...mapperCommand, "content-mapper"],
          directory,
          path.join(directory, "mapper-protocol"),
          frames,
        );
        observations[`${side}-mapper`] = run(
          [official, "--runExternalCode", "--noEmit", "-p", "tsconfig.json", "--pretty", "false"],
          directory,
          path.join(directory, "official-mapper"),
        );
        const cli = mapperCommand;
        observations[`${side}-check`] = run(
          [
            ...cli,
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "src",
            "--format",
            "json",
            "--quiet",
            "--corsa-path",
            bundled,
          ],
          directory,
          path.join(directory, "check"),
        );
        write(path.join(directory, "mapper-startup-configuration.json"), {
          manifest: JSON.parse(
            fs.readFileSync(path.join(nodeModules, "vize/package.json"), "utf8"),
          ),
          node: process.execPath,
          nodeSha256: digest(fs.readFileSync(process.execPath)),
          environmentPolicy:
            "explicit Node executable directory first; no Vize/native or Node module overrides",
          custody: "startup configuration; not an external host RPC witness",
        });
        // All children have exited. Retain exact manifests/packets above and
        // remove only this fresh project's dependency link/tree, so artifact
        // collection cannot follow it and duplicate installed SDK binaries.
        fs.rmSync(nodeModules, { recursive: true });
      }
      rows.push({ id, files, diagnostics, observations, mapperStdout: project.mapperStdout });
    }
write(path.join(output, "inventory.json"), {
  cases: rows.length,
  originalCases: relocationOnly || callbackOnly ? 0 : 60,
  additionalEdgeCases: relocationOnly || callbackOnly ? rows.length : 8,
  executedProcesses: rows.reduce((total, row) => total + Object.keys(row.observations).length, 0),
  ids: rows.map((row) => row.id),
  mode: baselineOnly ? "baseline-only" : "candidate-and-baseline",
});
if (relocationOnly) {
  const prelude =
    "declare const Comp: { pick: (x: number) => boolean };\nconst state: { value?: number } = {};\nstate.value = 1;\n";
  const pick = "const pick: typeof Comp.pick = (field: typeof state.value) => !!field;\n";
  for (const [name, source] of [
    ["before-delete", prelude + pick + "delete state.value;\n"],
    ["after-delete", prelude + "delete state.value;\n" + pick],
  ]) {
    const directory = path.join(output, "independent-typescript-control", name!);
    fs.mkdirSync(directory, { recursive: true });
    fs.writeFileSync(path.join(directory, "control.ts"), source!);
    write(path.join(directory, "tsconfig.json"), {
      compilerOptions: { strict: true, noEmit: true, types: [] },
      files: ["control.ts"],
    });
    run(
      [official, "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
      directory,
      path.join(directory, "compiler"),
    );
    write(path.join(directory, "input.json"), {
      source,
      sha256: digest(source!),
      kind: "independent-plain-typescript-not-a-projection",
    });
  }
}
assert.equal(rows.length, callbackOnly ? 92 : relocationOnly ? 56 : 68);
assert.equal(
  rows.reduce((total, row) => total + Object.keys(row.observations).length, 0),
  rows.length * (callbackOnly ? 8 : baselineOnly ? 3 : 6),
);
if (callbackOnly) write(path.join(output, "callback-cfa-observations.json"), rows);
// Every full protocol/check/official-host observation is durable before comparisons.
for (const row of relocationOnly || callbackOnly ? [] : rows) {
  const config = JSON.parse(row.files["tsconfig.json"]);
  const projectFiles = Object.keys(row.files)
    .filter((file) => file !== "tsconfig.json")
    .sort();
  const expectedNative = {
    files: projectFiles.map((file) => ({
      file,
      diagnostics: row.diagnostics
        .filter((d) => d.file === file)
        .map((d) => {
          const base = `error:${d.line}:${d.column} [TS${d.code}] ${d.message}`;
          if (file.endsWith(".art.vue")) return base;
          const line = row.files[file].split(/\r?\n/u).at(d.line - 1);
          assert.ok(line);
          const prefix = line.includes(header);
          const binding =
            d.code === 2578
              ? "; binding: @vue-expect-error"
              : d.code === 2532 && !prefix
                ? "; binding: 'count'"
                : "";
          return `${base} (source: ${line.trim()}${binding})`;
        }),
    })),
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: config.compilerOptions,
        files: projectFiles,
      },
    ],
    errorCount: row.diagnostics.length,
    warningCount: 0,
    fileCount: projectFiles.length,
  };
  const expectedMapper =
    row.mapperStdout ??
    row.diagnostics
      .map((d) => `${d.file}(${d.line},${d.column}): error TS${d.code}: ${d.message}\n`)
      .join("");
  for (const side of candidate ? ["baseline", "candidate"] : ["baseline"]) {
    successful(row.observations[`${side}-protocol`]);
    const check = row.observations[`${side}-check`];
    assert.equal(check.error, null, row.id);
    assert.equal(check.signal, null, row.id);
    assert.equal(check.stderr, "", row.id);
    assert.equal(check.status, row.diagnostics.length ? 1 : 0, row.id);
    assert.deepEqual(
      JSON.parse(check.stdout),
      expectedNative,
      `${row.id}/${side}: complete CLI packet`,
    );
    const mapper = row.observations[`${side}-mapper`];
    assert.equal(mapper.error, null, row.id);
    assert.equal(mapper.signal, null, row.id);
    assert.equal(mapper.stderr, "", row.id);
    const baselineHasEchoes =
      side === "baseline" && /^(?:form-(?:callback-)?)?expect-used-/u.test(row.id);
    assert.equal(mapper.status, row.diagnostics.length || baselineHasEchoes ? 2 : 0, row.id);
    if (side === "candidate" || (row.diagnostics.length === 0 && !baselineHasEchoes))
      assert.equal(
        mapper.stdout,
        expectedMapper,
        `${row.id}/${side}: complete official-host stdout`,
      );
  }
}
write(path.join(output, "result.json"), {
  sourceSha,
  cases: rows.length,
  sourceQualified: !baselineOnly && !relocationOnly && !callbackOnly,
  relocationDiscovery: relocationOnly,
  callbackControlDiscovery: callbackOnly,
  registryBaselineOnly: baselineOnly,
  fullNativeVectors: true,
  fullCandidateMapperVectors: !baselineOnly,
  scope: "actual source CLI and pinned published host; not a release/public candidate claim",
});
