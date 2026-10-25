import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { asRecord, parseJson, readJson } from "../../json-value.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

/** Run the current source CLI over the consumer's actual Vue 2 type graph. */
export function verifyNuxt2Typecheck(fixture: string, artifacts: string): void {
  const project = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt2-types-"));
  const requireRoot = createRequire(path.join(root, "package.json"));
  const corsa =
    process.env.CORSA_PATH ??
    path.join(
      path.dirname(
        requireRoot.resolve(
          `@typescript/typescript-${process.platform}-${process.arch}/package.json`,
        ),
      ),
      "lib",
      process.platform === "win32" ? "tsc.exe" : "tsc",
    );
  assert.equal(fs.existsSync(corsa), true);
  const providerVersion = spawnSync(corsa, ["--version"], { encoding: "utf8" });
  assert.equal(providerVersion.status, 0);
  const provider = {
    path: corsa,
    sha256: hash(fs.readFileSync(corsa)),
    version: providerVersion.stdout.trim(),
  };
  const native = path.join(root, "npm/native/index.js");
  const sourceHead = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
  }).stdout.trim();
  const identity = path.join(artifacts, "typecheck-source-binding.json");
  const script = [
    `const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');`,
    `const native=require(${JSON.stringify(native)});`,
    `const selected=process.report.getReport().sharedObjects.filter(file=>file.endsWith('.node')&&path.basename(file).includes('vize-vitrine'));`,
    `if(selected.length!==1)throw new Error('source Vize binding identity is ambiguous');`,
    `const file=fs.realpathSync(selected[0]);`,
    `if(!file.startsWith(${JSON.stringify(root)}))throw new Error('typecheck must load the current workspace binding');`,
    `fs.writeFileSync(${JSON.stringify(identity)},JSON.stringify({path:file,sha256:crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex'),source:${JSON.stringify(sourceHead)}},null,2));`,
    `native.runCli(JSON.parse(process.argv[1]));`,
  ].join("\n");
  const valid = fs.readFileSync(
    path.join(root, "tools/support/compat/nuxt/fixtures/nuxt2-typecheck/App.vue.txt"),
    "utf8",
  );
  const rows: unknown[] = [];
  try {
    fs.symlinkSync(
      path.join(fixture, "node_modules"),
      path.join(project, "node_modules"),
      process.platform === "win32" ? "junction" : "dir",
    );
    fs.writeFileSync(path.join(project, "nuxt.config.ts"), "export default {}\n");
    fs.writeFileSync(
      path.join(project, "vite.config.mjs"),
      `export default ${JSON.stringify({ vue: { version: "2.7", typeChecker: { legacyVue2: true, optionsApi: true, corsaPath: corsa } } })};\n`,
    );
    fs.writeFileSync(
      path.join(project, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          skipLibCheck: true,
        },
        include: ["App.vue"],
        vueCompilerOptions: { target: 2.7 },
      }),
    );
    for (const [name, source, errors] of [
      ["valid-vue2-context", valid, 0],
      ["invalid-string-method", valid.replace("message.toUpperCase()", "message.toFixed(2)"), 1],
    ] as const) {
      fs.writeFileSync(path.join(project, "App.vue"), source);
      const args = ["check", "App.vue", "--format", "json", "--corsa-path", corsa];
      const started = performance.now();
      const result = spawnSync(process.execPath, ["--eval", script, JSON.stringify(args)], {
        cwd: project,
        encoding: "utf8",
        timeout: 60_000,
        env: { ...process.env, NO_COLOR: "1" },
      });
      fs.writeFileSync(path.join(artifacts, `${name}.stdout.json`), result.stdout);
      fs.writeFileSync(path.join(artifacts, `${name}.stderr.log`), result.stderr);
      const packet = asRecord(parseJson(result.stdout), "Nuxt 2 typecheck report");
      const row = {
        name,
        args,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        milliseconds: performance.now() - started,
        source,
        sourceSha256: hash(Buffer.from(source)),
        packet,
      };
      rows.push(row);
      fs.writeFileSync(
        path.join(artifacts, "typecheck-packets.json"),
        JSON.stringify(
          {
            sourceBinding: readJson(identity),
            provider,
            vue: readJson(path.join(fixture, "node_modules/vue/package.json")),
            rows,
          },
          null,
          2,
        ) + "\n",
      );
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, errors === 0 ? 0 : 1, result.stderr + result.stdout);
      assert.equal(packet.errorCount, errors);
      assert.deepEqual(
        packet,
        readJson(
          path.join(
            root,
            "tools/support/compat/nuxt/fixtures/nuxt2-typecheck",
            `${name}.expected.json`,
          ),
        ),
      );
    }
  } finally {
    fs.rmSync(project, { recursive: true, force: true });
  }
}
