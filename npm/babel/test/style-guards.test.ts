import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { test } from "node:test";
import {
  exists,
  hash,
  hostSpecs,
  runtimeSpecs,
  runtimeIdentity,
  prepareCompiler,
  json,
} from "./runtime/environment.ts";

let configured: ReturnType<typeof settings> | undefined;
const compilers = new Map<number, ReturnType<typeof prepareCompiler>>();
function settings() {
  assert.match(process.env.SOURCE_SHA ?? "", /^[a-f0-9]{40}$/);
  assert(process.env.VIZE_BABEL_PACKED_PLUGIN && process.env.VIZE_BABEL_NATIVE_ROOT);
  assert(process.env.VIZE_BABEL_STYLE_GUARD_OUTPUT, "fresh independent guard output required");
  const output = path.resolve(process.env.VIZE_BABEL_STYLE_GUARD_OUTPUT);
  assert(!exists(output));
  fs.mkdirSync(output, { recursive: true });
  const tarball = fs.realpathSync(process.env.VIZE_BABEL_PACKED_PLUGIN);
  return {
    source: process.env.SOURCE_SHA!,
    output,
    tarball,
    tarballSha256: hash(fs.readFileSync(tarball)),
    nativeRoot: fs.realpathSync(process.env.VIZE_BABEL_NATIVE_ROOT),
  };
}
for (const spec of hostSpecs)
  for (const format of ["esm", "cjs"] as const) {
    void test(`actual packed Babel ${spec.major}/${format} independent literal-style guards`, () => {
      configured ??= settings();
      if (!compilers.has(spec.major)) compilers.set(spec.major, prepareCompiler(spec, configured));
      const compiler = compilers.get(spec.major)!,
        runtime = runtimeIdentity(runtimeSpecs.find((row) => row.id === "rc11")!);
      const id = `babel${spec.major}-${format}`,
        report = path.join(configured.output, id + ".json");
      const input = {
        ...configured,
        compiler,
        runtime,
        id,
        format,
        report,
        qualificationRole: "additive-csr-style-guards",
      };
      const file = path.join(configured.output, id + ".input.json");
      json(file, input);
      const runner = pathToFileURL(path.resolve(import.meta.dirname, "style-guards/run.ts")).href;
      const loading =
        "{requireHost,core:()=>import('@babel/core'),typescript:()=>import('@babel/plugin-transform-typescript'),plugin:()=>" +
        (format === "esm"
          ? "import('@vizejs/babel-plugin')"
          : "Promise.resolve(requireHost('@vizejs/babel-plugin'))") +
        "}";
      const body = `const {run}=await import(${JSON.stringify(runner)});await run(JSON.parse(requireHost('node:fs').readFileSync(process.argv[2])),${loading});`;
      const entry = path.join(
        compiler.root,
        "style-guard-" + format + (format === "esm" ? ".mjs" : ".cjs"),
      );
      fs.writeFileSync(
        entry,
        format === "esm"
          ? "import {createRequire} from 'node:module';const requireHost=createRequire(import.meta.url);" +
              body
          : "const requireHost=require;(async()=>{" +
              body +
              "})().catch(error=>{console.error(error);process.exitCode=1});",
      );
      const child = spawnSync(process.execPath, [entry, file], {
        cwd: runtime.root,
        env: process.env,
        timeout: 180000,
        maxBuffer: 64 * 1024 * 1024,
      });
      fs.writeFileSync(
        path.join(configured.output, id + ".stdout.raw"),
        child.stdout ?? Buffer.alloc(0),
      );
      fs.writeFileSync(
        path.join(configured.output, id + ".stderr.raw"),
        child.stderr ?? Buffer.alloc(0),
      );
      json(path.join(configured.output, id + ".process.json"), {
        status: child.status,
        signal: child.signal,
        error: child.error ? String(child.error) : null,
      });
      assert.equal(child.status, 0, report + "\n" + String(child.stderr));
      assert.equal(child.signal, null);
      assert.equal(child.stderr.length, 0);
      assert.equal(JSON.parse(fs.readFileSync(report, "utf8")).error, null);
    });
  }
