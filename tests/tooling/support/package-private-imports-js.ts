import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { diagnostic, save } from "./editor-jsconfig.ts";
import { original } from "./package-private-imports.ts";

export function vectors(mode: string, native = false, directory = ""): Record<string, unknown[]> {
  const argument = "Argument of type 'number' is not assignable to parameter of type 'string'.";
  const missing = "Cannot find module '#lib/util.js' or its corresponding type declarations.";
  const app =
    mode === "bad-call"
      ? [diagnostic(2345, argument, 3, 22, 24, native)]
      : mode === "missing-target"
        ? [diagnostic(2307, missing, 1, 22, 36, native)]
        : [];
  const main =
    mode === "bad-call"
      ? [diagnostic(2345, argument, 3, 29, 31, native)]
      : mode === "missing-target"
        ? [diagnostic(2307, missing, 0, 22, 36, native)]
        : [];
  if (mode === "ordinary-package") {
    // Native JS retains a suggestion; the existing checked SFC emits strict TS.
    const packageFile = path.join(directory, "node_modules/ordinary/index.js");
    const message = `Could not find a declaration file for module 'ordinary'. '${packageFile}' implicitly has an 'any' type.`;
    app.push({ ...diagnostic(7016, message, 1, 22, 32, native), severity: native ? 4 : 1 });
    main.push({ ...diagnostic(7016, message, 0, 22, 32, native), severity: 4 });
  }
  if (native)
    app.push(
      diagnostic(6133, "'message' is declared but its value is never read.", 3, 6, 13, true),
    );
  return { "src/App.vue": app, "src/main.js": main };
}

export function prepare(
  directory: string,
  configName: string,
  mode: string,
  name: string,
): Record<string, string> {
  const inputs: Record<string, string> = {
    "package.json": original("package.json"),
    [configName]: original("jsconfig.json"),
    "src/lib/util.js": original("util.js"),
    "src/App.vue": original("App-js.vue"),
    "src/main.js": original("main.js"),
  };
  if (mode === "bad-call") {
    inputs["src/App.vue"] = inputs["src/App.vue"].replace('greet("x")', "greet(42)");
    inputs["src/main.js"] = inputs["src/main.js"].replace('greet("y")', "greet(42)");
  }
  if (mode === "missing-target")
    inputs["package.json"] = inputs["package.json"].replace("./src/lib/*", "./src/nope/*");
  if (mode === "ordinary-package") {
    for (const file of ["src/App.vue", "src/main.js"])
      inputs[file] = inputs[file].replace("#lib/util.js", "ordinary");
    inputs["node_modules/ordinary/package.json"] =
      '{"name":"ordinary","type":"module","exports":"./index.js"}';
    inputs["node_modules/ordinary/index.js"] = original("util.js");
  }
  for (const [file, bytes] of Object.entries(inputs)) {
    fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
    fs.writeFileSync(path.join(directory, file), bytes);
  }
  const vue = fs.realpathSync(process.env.VIZE_PRIVATE_IMPORT_VUE_ROOT!);
  const provider = JSON.parse(fs.readFileSync(path.join(vue, "package.json"), "utf8"));
  assert.equal(provider.name, "vue");
  assert.equal(provider.version, "3.5.43");
  fs.mkdirSync(path.join(directory, "node_modules"), { recursive: true });
  fs.symlinkSync(vue, path.join(directory, "node_modules/vue"), "dir");
  assert.equal(fs.existsSync(path.join(directory, "vize.config.json")), false);
  save(name + "-inputs", { inputs, vue, provider, sourceSha: process.env.SOURCE_SHA });
  return inputs;
}

export function cliCheck(
  binary: string,
  directory: string,
  inputs: Record<string, string>,
  mode: string,
  name: string,
): void {
  const result = spawnSync(binary, ["check", "--format", "json"], {
    cwd: directory,
    maxBuffer: 16 * 1024 * 1024,
  });
  save(name + "-cli", {
    argv: ["check", "--format", "json"],
    cwd: directory,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdoutBase64: result.stdout.toString("base64"),
    stderrBase64: result.stderr.toString("base64"),
    inputs,
  });
  const expected = vectors(mode, false, directory);
  const files = ["src/App.vue", "src/lib/util.js", "src/main.js"];
  const rendered = (file: string) =>
    (expected[file] ?? []).map((value) => {
      const item = value as ReturnType<typeof diagnostic>;
      const severity = item.severity === 4 ? "hint" : "error";
      return `${severity}:${item.range.start.line + 1}:${item.range.start.character + 1} [TS${item.code}] ${item.message}`;
    });
  const errors =
    mode === "bad-call" || mode === "missing-target" ? 2 : mode === "ordinary-package" ? 1 : 0;
  assert.equal(result.status, errors ? 1 : 0);
  assert.equal(result.signal, null);
  assert.equal(result.error, undefined);
  assert.deepEqual(JSON.parse(result.stdout.toString()), {
    files: files.map((file) => ({ file, diagnostics: rendered(file) })),
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: JSON.parse(inputs["tsconfig.json"]).compilerOptions,
        files,
      },
    ],
    errorCount: errors,
    warningCount: 0,
    fileCount: 3,
  });
}
