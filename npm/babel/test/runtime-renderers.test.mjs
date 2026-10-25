import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  settings,
  hostSpecs,
  runtimeSpecs,
  runtimeIdentity,
  prepareCompiler,
  json,
  here,
} from "./runtime/environment.mjs";

let configured;
const compilers = new Map();
for (const runtimeSpec of runtimeSpecs)
  for (const hostSpec of hostSpecs)
    for (const format of ["esm", "cjs"]) {
      const id = runtimeSpec.id + "-babel" + hostSpec.major + "-" + format;
      void test("actual packed Babel " + id + " mount, SSR and hydration", () => {
        configured ??= settings();
        if (!compilers.has(hostSpec.major))
          compilers.set(hostSpec.major, prepareCompiler(hostSpec, configured));
        const compiler = compilers.get(hostSpec.major),
          runtime = runtimeIdentity(runtimeSpec);
        const input = {
          ...configured,
          id,
          compiler,
          runtime,
          format,
          report: path.join(configured.output, id + ".json"),
        };
        const inputFile = path.join(configured.output, id + ".input.json");
        json(inputFile, input);
        const runner = pathToFileURL(path.join(here, "run.mjs")).href;
        const loading =
          "{requireHost,core:()=>import('@babel/core'),typescript:()=>import('@babel/plugin-transform-typescript'),plugin:()=>" +
          (format === "esm"
            ? "import('@vizejs/babel-plugin')"
            : "Promise.resolve(requireHost('@vizejs/babel-plugin'))") +
          "}";
        const body =
          "const fs=requireHost('node:fs'); const {runRuntime}=await import(" +
          JSON.stringify(runner) +
          "); await runRuntime(JSON.parse(fs.readFileSync(process.argv[2]))," +
          loading +
          ");";
        const entry = path.join(
          compiler.root,
          "consumer-" + format + (format === "esm" ? ".mjs" : ".cjs"),
        );
        fs.writeFileSync(
          entry,
          format === "esm"
            ? "import {createRequire} from 'node:module'; const requireHost=createRequire(import.meta.url); " +
                body
            : "const requireHost=require; (async()=>{" +
                body +
                "})().catch(error=>{console.error(error);process.exitCode=1;});",
        );
        const result = spawnSync(process.execPath, [entry, inputFile], {
          cwd: runtime.root,
          env: process.env,
          timeout: 180000,
          maxBuffer: 64 * 1024 * 1024,
        });
        fs.writeFileSync(
          path.join(configured.output, id + ".stdout.raw"),
          result.stdout ?? Buffer.alloc(0),
        );
        fs.writeFileSync(
          path.join(configured.output, id + ".stderr.raw"),
          result.stderr ?? Buffer.alloc(0),
        );
        json(path.join(configured.output, id + ".process.json"), {
          argv: [process.execPath, entry, inputFile],
          status: result.status,
          signal: result.signal,
          error: result.error ? String(result.error) : null,
        });
        assert.equal(result.status, 0, input.report + "\n" + String(result.stderr));
        assert.equal(result.signal, null);
        assert.equal(result.stderr.length, 0);
        const report = JSON.parse(fs.readFileSync(input.report));
        assert.equal(report.error, null);
      });
    }
