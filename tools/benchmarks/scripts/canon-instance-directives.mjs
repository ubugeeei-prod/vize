import { cases, referenceSource } from "./canon-instance-directives-cases.mjs";
/** Actual source CLI/native directive receipts; no performance claim. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const save = (file, value) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
};
const identify = (file) => {
  const actual = fs.realpathSync(file),
    bytes = fs.readFileSync(actual);
  return { path: actual, bytes: bytes.length, sha256: hash(bytes) };
};
const point = (source, token, last = false) => {
  const at = last ? source.lastIndexOf(token) : source.indexOf(token);
  assert.ok(at >= 0, "missing owned token: " + token);
  const lines = source.slice(0, at).split(/\r\n|[\r\n\u2028\u2029]/u);
  return { line: lines.length, column: lines.at(-1).length + 1 };
};
export function packet(stdout, status, directory) {
  const result = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(stdout));
  assert.ok(Array.isArray(result.files));
  const diagnostics = result.files.flatMap((file) => {
    assert.ok(Array.isArray(file.diagnostics));
    return file.diagnostics.map((text) => {
      const m = /^(error|warning):(\d+):(\d+) (?:\[TS(\d+)\] )?([\s\S]+)$/u.exec(text);
      assert.ok(m, "unrecognized complete diagnostic: " + text);
      return {
        file: path.relative(directory, path.resolve(directory, file.file)),
        severity: m[1],
        line: +m[2],
        column: +m[3],
        code: m[4] ? +m[4] : null,
        message: m[5],
        raw: text,
      };
    });
  });
  assert.equal(result.errorCount, diagnostics.filter((d) => d.severity === "error").length);
  assert.ok(status === 0 || status === 1);
  assert.equal(status === 0, result.errorCount === 0);
  return { result, diagnostics };
}
function run(output, name, executable, argv, cwd, env = process.env) {
  const r = spawnSync(executable, argv, { cwd, env, timeout: 90_000, maxBuffer: 64 * 1024 * 1024 });
  const prefix = path.join(output, name),
    stdout = r.stdout ?? Buffer.alloc(0),
    stderr = r.stderr ?? Buffer.alloc(0);
  fs.mkdirSync(path.dirname(prefix), { recursive: true });
  fs.writeFileSync(prefix + ".stdout.bin", stdout);
  fs.writeFileSync(prefix + ".stderr.bin", stderr);
  save(prefix + ".process.json", {
    executable: identify(executable),
    argv,
    cwd,
    status: r.status,
    signal: r.signal,
    error: r.error?.message ?? null,
    stdoutSha256: hash(stdout),
    stderrSha256: hash(stderr),
    environment: Object.fromEntries(
      ["CORSA_PATH", "PATH", "GOMAXPROCS", "RAYON_NUM_THREADS", "NODE_OPTIONS", "NODE_PATH"].map(
        (key) => [key, env[key] ?? null],
      ),
    ),
  });
  assert.equal(r.error, undefined, r.error?.message);
  assert.equal(r.signal, null);
  return { ...r, stdout, stderr };
}
const augmentation =
  'import "vue";\ndeclare module "vue" {\n  interface ComponentCustomProperties {\n    x: number;\n    pluginMethod(value: string): string;\n  }\n}\nexport {};\n';
const wrapperSource = [
  "#!/usr/bin/env node",
  "import fs from 'node:fs'; import path from 'node:path'; import {spawnSync} from 'node:child_process';",
  "const root=process.env.VIZE_DIRECTIVE_NATIVE_CAPTURE; fs.mkdirSync(root,{recursive:true});",
  "const out=fs.mkdtempSync(path.join(root,'call-')),argv=process.argv.slice(2);",
  "fs.writeFileSync(path.join(out,'request.json'),JSON.stringify({executable:process.env.VIZE_DIRECTIVE_NATIVE,argv,cwd:process.cwd()}));",
  "const pi=argv.indexOf('--project'); if(pi>=0){const cfg=path.resolve(process.cwd(),argv[pi+1]);fs.copyFileSync(cfg,path.join(out,'tsconfig.json'));}",
  "const r=spawnSync(process.env.VIZE_DIRECTIVE_NATIVE,argv,{env:process.env,cwd:process.cwd(),maxBuffer:64*1024*1024,timeout:85000});",
  "fs.writeFileSync(path.join(out,'stdout.bin'),r.stdout??Buffer.alloc(0));fs.writeFileSync(path.join(out,'stderr.bin'),r.stderr??Buffer.alloc(0));",
  "fs.writeFileSync(path.join(out,'response.json'),JSON.stringify({status:r.status,signal:r.signal,error:r.error?.message??null}));",
  "if(r.stdout?.length)await new Promise(resolve=>process.stdout.write(r.stdout,resolve));",
  "if(r.stderr?.length)await new Promise(resolve=>process.stderr.write(r.stderr,resolve));",
  "if(r.signal)process.kill(process.pid,r.signal);else process.exitCode=r.status??1;",
  "",
].join("\n");
function assertErrors(row, observed) {
  const expected = row.errors.map(({ code, token, last }) => ({
    file: "main.vue",
    severity: "error",
    code,
    ...point(row.source, token, last),
  }));
  const order = (a, b) => a.line - b.line || a.column - b.column || a.code - b.code;
  assert.deepEqual(
    observed.map(({ message, raw, ...rest }) => rest).sort(order),
    expected.sort(order),
    row.id,
  );
  for (const d of observed) {
    if (d.code === 2578) assert.match(d.message, /^Unused '@ts-expect-error' directive\./u);
    if (d.code === 2339)
      assert.match(
        d.message,
        /^Property '[^']+' does not exist on (?:type |the component instance\.)/u,
      );
    if (d.code === 2345) assert.match(d.message, /^Argument of type /u);
  }
}
export async function main(argv) {
  assert.equal(argv.length, 5, "sourceRoot providerRoot baseCli headCli captureRoot");
  const [sourceRoot, providerRoot, baseCli, headCli, output] = argv.map((file) =>
    path.resolve(file),
  );
  fs.mkdirSync(output, { recursive: true });
  const fixture = path.join(
    sourceRoot,
    "tests/_fixtures/vue-language-tools/upstream/test-workspace",
  );
  const original = fs.readFileSync(path.join(fixture, "tsc/#3488/main.vue"), "utf8");
  const baseConfig = fs.readFileSync(path.join(fixture, "tsconfig.base.json")),
    config = fs.readFileSync(path.join(fixture, "tsc/#3488/tsconfig.json"));
  const providers = {};
  for (const [name, version] of [
    ["vue", "3.6.0-rc.6"],
    ["typescript", "6.0.3"],
    ["native-typescript", "7.1.0-beta"],
  ]) {
    const file = path.join(providerRoot, "node_modules", name, "package.json"),
      manifest = JSON.parse(fs.readFileSync(file));
    assert.equal(manifest.version, version);
    providers[name] = { version, manifest: identify(file) };
    save(path.join(output, "providers", name + ".manifest.json"), manifest);
  }
  const getExe = (
    await import(
      pathToFileURL(path.join(providerRoot, "node_modules/native-typescript/lib/getExePath.js"))
        .href
    )
  ).default;
  const native = fs.realpathSync(getExe());
  assert.ok(native.startsWith(fs.realpathSync(providerRoot) + path.sep));
  const platformManifest = path.resolve(path.dirname(native), "../package.json");
  assert.equal(JSON.parse(fs.readFileSync(platformManifest)).version, "7.1.0-beta");
  const jsCompiler = path.join(providerRoot, "node_modules/typescript/bin/tsc");
  const version = run(output, "providers/native-version", native, ["--version"], providerRoot);
  assert.equal(version.status, 0);
  assert.equal(version.stdout.toString(), "Version 7.1.0-beta\n");
  const jsVersion = run(
    output,
    "providers/js-version",
    process.execPath,
    [jsCompiler, "--version"],
    providerRoot,
  );
  assert.equal(jsVersion.status, 0);
  assert.equal(jsVersion.stdout.toString(), "Version 6.0.3\n");
  for (const name of ["package.json", "package-lock.json"])
    fs.copyFileSync(path.join(providerRoot, name), path.join(output, "providers", name));
  const binaries = {
    base: identify(baseCli),
    head: identify(headCli),
    native: identify(native),
    node: identify(process.execPath),
    jsCompiler: identify(jsCompiler),
  };
  assert.notEqual(binaries.base.sha256, binaries.head.sha256, "different source products required");
  save(path.join(output, "providers.json"), {
    providers,
    binaries,
    platform: identify(platformManifest),
  });
  const wrapper = path.join(output, "native-observer.mjs");
  fs.writeFileSync(wrapper, wrapperSource, { mode: 0o755 });
  const observations = [];
  for (const row of cases(original)) {
    const directory = path.join(output, "projects", row.id, "test-workspace/tsc/#3488"),
      workspace = path.resolve(directory, "../..");
    fs.mkdirSync(directory, { recursive: true });
    fs.symlinkSync(
      path.join(providerRoot, "node_modules"),
      path.join(workspace, "node_modules"),
      "dir",
    );
    fs.writeFileSync(path.join(workspace, "tsconfig.base.json"), baseConfig);
    fs.writeFileSync(path.join(directory, "tsconfig.json"), config);
    fs.writeFileSync(path.join(directory, "main.vue"), row.source);
    if (row.augmented) fs.writeFileSync(path.join(directory, "augmentation.d.ts"), augmentation);
    const inputPaths = [
      path.join(directory, "main.vue"),
      path.join(directory, "tsconfig.json"),
      path.join(workspace, "tsconfig.base.json"),
      ...(row.augmented ? [path.join(directory, "augmentation.d.ts")] : []),
    ];
    const inputs = inputPaths.map(identify);
    save(path.join(output, "cases", row.id, "input.json"), { ...row, inputs });
    const roles = {};
    for (const [role, binary] of [
      ["base", baseCli],
      ["head", headCli],
    ]) {
      const env = {
        ...process.env,
        NO_COLOR: "1",
        CORSA_PATH: wrapper,
        VIZE_DIRECTIVE_NATIVE: native,
        VIZE_DIRECTIVE_NATIVE_CAPTURE: path.join(output, "cases", row.id, role, "native"),
      };
      const result = run(
        output,
        "cases/" + row.id + "/" + role + "/cli",
        binary,
        [
          "check",
          ".",
          "--no-config",
          "--format",
          "json",
          "--show-virtual-ts",
          "--quiet",
          "--corsa-path",
          wrapper,
        ],
        directory,
        env,
      );
      roles[role] = packet(result.stdout, result.status, directory);
      assert.ok(
        roles[role].result.files.some(
          (file) => path.basename(file.file) === "main.vue" && typeof file.virtualTs === "string",
        ),
      );
      save(path.join(output, "cases", row.id, role, "packet.json"), roles[role]);
      assert.deepEqual(
        inputPaths.map(identify),
        inputs,
        "source/config must remain byte-identical",
      );
      const nativeCalls = fs
        .readdirSync(env.VIZE_DIRECTIVE_NATIVE_CAPTURE)
        .filter((name) => name.startsWith("call-"));
      assert.ok(
        nativeCalls.some((name) =>
          JSON.parse(
            fs.readFileSync(path.join(env.VIZE_DIRECTIVE_NATIVE_CAPTURE, name, "request.json")),
          ).argv.includes("--project"),
        ),
        "real native checking must occur",
      );
    }
    const virtual = (role) =>
      roles[role].result.files
        .map(({ file, virtualTs }) => ({ file, virtualTs }))
        .sort((a, b) => a.file.localeCompare(b.file));
    assert.deepEqual(
      virtual("base"),
      virtual("head"),
      row.id + ": complete production VTS unchanged",
    );
    assertErrors(row, roles.head.diagnostics);
    for (const d of roles.head.diagnostics)
      assert.ok(
        roles.base.diagnostics.some((before) => JSON.stringify(before) === JSON.stringify(d)),
        row.id + ": retained complete diagnostic changed",
      );
    if (row.original)
      assert.deepEqual(roles.base.diagnostics.map((d) => d.code).sort(), [2339, 2578]);
    const reference = path.join(output, "references", row.id);
    fs.mkdirSync(reference, { recursive: true });
    fs.symlinkSync(
      path.join(providerRoot, "node_modules"),
      path.join(reference, "node_modules"),
      "dir",
    );
    fs.writeFileSync(path.join(reference, "reference.ts"), referenceSource(row));
    fs.writeFileSync(path.join(reference, "tsconfig.base.json"), baseConfig);
    if (row.augmented) fs.writeFileSync(path.join(reference, "augmentation.d.ts"), augmentation);
    save(path.join(reference, "tsconfig.json"), {
      extends: "./tsconfig.base.json",
      compilerOptions: {
        module: "ESNext",
        moduleResolution: "Bundler",
        target: "ESNext",
        types: [],
      },
      files: ["reference.ts", ...(row.augmented ? ["augmentation.d.ts"] : [])],
      include: [],
    });
    for (const kind of ["js", "native"]) {
      const args = [
        "-p",
        "tsconfig.json",
        "--pretty",
        "false",
        ...(kind === "native" ? ["--checkers", "1"] : []),
      ];
      const result = run(
        output,
        "cases/" + row.id + "/reference-" + kind,
        kind === "js" ? process.execPath : native,
        kind === "js" ? [jsCompiler, ...args] : args,
        reference,
      );
      const codes = [...result.stdout.toString().matchAll(/^.+\(\d+,\d+\): error TS(\d+):/gmu)]
        .map((m) => +m[1])
        .sort();
      assert.deepEqual(
        codes,
        row.errors.map((d) => d.code).sort(),
        row.id + ": independent " + kind + " reference",
      );
      assert.equal(result.status === 0, codes.length === 0);
    }
    observations.push({
      id: row.id,
      sourceSha256: hash(row.source),
      base: roles.base.diagnostics,
      head: roles.head.diagnostics,
      virtualTsUnchanged: true,
    });
  }
  save(path.join(output, "acceptance.json"), {
    schema: "vize.canonInstanceDirectives.v1",
    cases: observations,
    binaries,
    original: { file: "tsc/#3488/main.vue", sha256: hash(original) },
    scope:
      "actual source CLI+native correctness; complete VTS equality; independent TS6/native7 references, no speed claim",
    knownSeparateScope:
      "triple-slash and last-line block generator retention gaps are not qualified by this matrix",
  });
}
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href)
  await main(process.argv.slice(2));
