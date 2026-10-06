import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { root } from "./lsp/paths.ts";
import { diagnostic, save, stockOracle as nativeStockOracle } from "./editor-jsconfig.ts";

export const corpus = path.join(root, "tests/_fixtures/differential/lsp/package-private-imports");
export const original = (file: string) => fs.readFileSync(path.join(corpus, file + ".txt"), "utf8");
export {
  captureRoot,
  change,
  diagnostic,
  hash,
  initialize,
  observation,
  open,
  publication,
  save,
  uri,
} from "./editor-jsconfig.ts";

export function expected(mode: string, native = false): Record<string, unknown[]> {
  const message = "Argument of type 'number' is not assignable to parameter of type 'string'.";
  const missing = "Cannot find module '#lib/util.ts' or its corresponding type declarations.";
  const app =
    mode === "bad-call"
      ? [diagnostic(2345, message, 3, 22, 24, native)]
      : mode === "missing-target"
        ? [diagnostic(2307, missing, 1, 22, 36, native)]
        : [];
  const main =
    mode === "bad-call"
      ? [diagnostic(2345, message, 2, 37, 39, native)]
      : mode === "missing-target"
        ? [diagnostic(2307, missing, 0, 22, 36, native)]
        : [];
  if (native)
    app.push(
      diagnostic(6133, "'message' is declared but its value is never read.", 3, 6, 13, true),
    );
  return { "src/App.vue": app, "src/main.ts": main };
}

export function prepare(directory: string, mode: string): Record<string, string> {
  const inputs: Record<string, string> = {
    "package.json": original("package.json"),
    "tsconfig.json": original("tsconfig.json"),
    "src/lib/util.ts": original("util.ts"),
    "src/App.vue": original("App.vue"),
    "src/main.ts": original("main.ts"),
  };
  if (mode === "missing-target")
    inputs["package.json"] = inputs["package.json"].replace("./src/lib/*", "./src/nope/*");
  if (mode === "bad-call") {
    inputs["src/App.vue"] = inputs["src/App.vue"].replace('greet("x")', "greet(42)");
    inputs["src/main.ts"] = inputs["src/main.ts"].replace('greet("y")', "greet(42)");
  }
  if (mode === "ordinary-package") {
    for (const file of ["src/App.vue", "src/main.ts"])
      inputs[file] = inputs[file].replace("#lib/util.ts", "ordinary");
    inputs["node_modules/ordinary/package.json"] =
      '{"name":"ordinary","type":"module","exports":"./index.ts"}';
    inputs["node_modules/ordinary/index.ts"] = original("util.ts");
  }
  for (const [file, content] of Object.entries(inputs)) {
    fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
    fs.writeFileSync(path.join(directory, file), content);
  }
  const vue = fs.realpathSync(process.env.VIZE_PRIVATE_IMPORT_VUE_ROOT!);
  const vuePackage = JSON.parse(fs.readFileSync(path.join(vue, "package.json"), "utf8"));
  assert.equal(vuePackage.name, "vue");
  assert.equal(vuePackage.version, "3.5.43");
  fs.mkdirSync(path.join(directory, "node_modules"), { recursive: true });
  fs.symlinkSync(vue, path.join(directory, "node_modules/vue"), "dir");
  save(mode + "-inputs", { inputs, vue, vuePackage, sourceSha: process.env.SOURCE_SHA });
  assert.equal(fs.existsSync(path.join(directory, "vize.config.json")), false);
  return inputs;
}

export async function stockOracle(
  directory: string,
  inputs: Record<string, string>,
  mode: string,
  runtime: string,
): Promise<void> {
  const source = inputs["src/App.vue"];
  const bare = source.slice(source.indexOf(">") + 1, source.indexOf("</script>"));
  await nativeStockOracle(
    directory,
    "tsconfig.json",
    inputs,
    bare,
    "ts",
    "src/main.ts",
    expected(mode, true),
    mode,
    runtime,
  );
}

export function cliCheck(
  binary: string,
  directory: string,
  inputs: Record<string, string>,
  mode: string,
): void {
  const command = spawnSync(binary, ["check", "--format", "json"], {
    cwd: directory,
    maxBuffer: 16 * 1024 * 1024,
  });
  save(mode + "-cli", {
    argv: ["check", "--format", "json"],
    cwd: directory,
    status: command.status,
    signal: command.signal,
    error: command.error?.message ?? null,
    stdoutBase64: command.stdout.toString("base64"),
    stderrBase64: command.stderr.toString("base64"),
    originalInputs: inputs,
  });
  const vectors = expected(mode);
  const files = ["src/App.vue", "src/lib/util.ts", "src/main.ts"];
  const diagnostics = Object.fromEntries(
    files.map((file) => [
      file,
      (vectors[file] ?? []).map((value) => {
        const item = value as ReturnType<typeof diagnostic>;
        return `error:${item.range.start.line + 1}:${item.range.start.character + 1} [TS${item.code}] ${item.message}`;
      }),
    ]),
  );
  assert.equal(command.status, mode === "bad-call" || mode === "missing-target" ? 1 : 0);
  assert.equal(command.signal, null);
  assert.equal(command.error, undefined);
  assert.deepEqual(JSON.parse(command.stdout.toString()), {
    files: files.map((file) => ({ file, diagnostics: diagnostics[file] })),
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: JSON.parse(inputs["tsconfig.json"]).compilerOptions,
        files,
      },
    ],
    errorCount: mode === "bad-call" || mode === "missing-target" ? 2 : 0,
    warningCount: 0,
    fileCount: 3,
  });
}
