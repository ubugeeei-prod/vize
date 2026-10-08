import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../differential/build-receipt.ts";

const repository = fileURLToPath(new URL("../../", import.meta.url));
const corpus = new URL("../_fixtures/differential/compiler/cli-whitespace-7880/", import.meta.url);
const manifest = JSON.parse(fs.readFileSync(new URL("manifest.json", corpus), "utf8"));
const source = fs.readFileSync(new URL("App.vue.txt", corpus), "utf8");
const ui = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const vue = ui("vue");
const compiler = ui("vue/compiler-sfc");
const server = ui("vue/server-renderer");
const binary = path.join(repository, binaryRelativePath());
const receipt = JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8"));
validateBuildReceipt(receipt, expectedBuildIdentity(repository));

void test("#7880 original CLI source/config bytes and official Vue version stay pinned", () => {
  assert.equal(manifest.issue, 7880);
  assert.equal(compiler.version, manifest.referenceVersion);
  assert.equal(vue.version, manifest.referenceVersion);
  for (const input of manifest.inputs) {
    const bytes = fs.readFileSync(new URL(input.path, corpus));
    assert.equal(bytes.length, input.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), input.sha256);
  }
});

void test("#7880 source-built CLI renders complete original DOM/SSR output like pinned Vue", async () => {
  const captures: any[] = [];
  const evidence = path.join(repository, "target/differential/cli-whitespace-7880");
  fs.mkdirSync(evidence, { recursive: true });
  const variants = [
    {
      name: "json-auto",
      file: "vize.config.json",
      explicit: false,
      noConfig: false,
      preserve: true,
    },
    { name: "ts-auto", file: "vize.config.ts", explicit: false, noConfig: false, preserve: true },
    {
      name: "json-explicit",
      file: "explicit.json",
      explicit: true,
      noConfig: false,
      preserve: true,
    },
    { name: "ts-explicit", file: "explicit.ts", explicit: true, noConfig: false, preserve: true },
    {
      name: "no-config",
      file: "vize.config.json",
      explicit: false,
      noConfig: true,
      preserve: false,
    },
    {
      name: "condense",
      file: "vize.config.json",
      explicit: false,
      noConfig: false,
      preserve: false,
    },
  ];
  try {
    for (const variant of variants) {
      for (const ssr of [false, true]) {
        const project = fs.mkdtempSync(path.join(os.tmpdir(), "vize-whitespace-7880-"));
        try {
          fs.writeFileSync(path.join(project, "App.vue"), source);
          const config =
            variant.name === "condense"
              ? '{"compiler":{"whitespace":"condense"}}'
              : fs.readFileSync(
                  new URL(
                    variant.file.endsWith(".ts") ? "vize.config.ts.txt" : "vize.config.json.txt",
                    corpus,
                  ),
                  "utf8",
                );
          fs.writeFileSync(path.join(project, variant.file), config);
          const args = ["build", "--slow-threshold", "600000", "-o", "out", "App.vue"];
          if (variant.explicit) args.push("--config", variant.file);
          if (variant.noConfig) args.push("--no-config");
          if (ssr) args.push("--ssr");
          const processResult = spawnSync(binary, args, {
            cwd: project,
            env: { ...process.env, RAYON_NUM_THREADS: "1" },
            timeout: 60_000,
            maxBuffer: 8 * 1024 * 1024,
          });
          const output = path.join(project, "out/App.js");
          const stdout = processResult.stdout ?? Buffer.alloc(0);
          const stderr = processResult.stderr ?? Buffer.alloc(0);
          const observation: any = {
            variant,
            ssr,
            receipt,
            args,
            cwd: project,
            source,
            config,
            status: processResult.status,
            signal: processResult.signal,
            error: processResult.error?.message ?? null,
            stdoutBase64: stdout.toString("base64"),
            stderrBase64: stderr.toString("base64"),
            code: fs.existsSync(output) ? fs.readFileSync(output, "utf8") : null,
          };
          captures.push(observation);
          fs.writeFileSync(
            path.join(evidence, "whole-observations.json"),
            JSON.stringify(captures, null, 2),
          );
          assert.equal(processResult.error, undefined);
          assert.equal(processResult.signal, null);
          assert.equal(processResult.status, 0, stderr.toString());
          assert.deepEqual(stdout, Buffer.alloc(0));
          const pieces = stderr.toString().split(" compiled in ");
          assert.equal(pieces.length, 2);
          const duration = pieces[1].slice(0, pieces[1].indexOf("s"));
          assert.match(duration, /^\d+\.\d{4}$/);
          assert.ok(Number.isFinite(Number(duration)) && Number(duration) >= 0);
          assert.equal(
            `${pieces[0]} compiled in <TIME>${pieces[1].slice(duration.length)}`,
            `Built: App.vue -> ${path.join("out", "App.js")}\n\x1b[32m✓ 1 file compiled in <TIME>s\x1b[0m\n`,
          );
          assert.equal(fs.readFileSync(path.join(project, "App.vue"), "utf8"), source);
          assert.equal(fs.readFileSync(path.join(project, variant.file), "utf8"), config);
          const parsed = compiler.parse(source, {
            filename: "App.vue",
            templateParseOptions: {
              whitespace: variant.preserve ? "preserve" : "condense",
            },
          });
          assert.deepEqual(parsed.errors, []);
          const reference = compiler.compileScript(parsed.descriptor, {
            id: "whitespace-7880",
            inlineTemplate: true,
            templateOptions: {
              ssr,
              compilerOptions: { whitespace: variant.preserve ? "preserve" : "condense" },
            },
          });
          observation.referenceCode = reference.content;
          observation.actual = await render(observation.code);
          observation.reference = await render(reference.content);
          fs.writeFileSync(
            path.join(evidence, "whole-observations.json"),
            JSON.stringify(captures, null, 2),
          );
          assert.deepEqual(observation.actual, observation.reference);
          assert.deepEqual(observation.actual.warnings, []);
        } finally {
          fs.rmSync(project, { recursive: true, force: true });
        }
      }
    }
    assert.equal(captures.length, 12);
    assert.notEqual(captures[0].actual.html, captures[8].actual.html);
  } finally {
    fs.writeFileSync(
      path.join(evidence, "whole-observations.json"),
      JSON.stringify(captures, null, 2),
    );
  }
});

let sequence = 0;
async function render(code: string) {
  (globalThis as any).__vizeWhitespace7880 = { vue, server };
  const linked = code.replace(
    /import \{([^}]*)\} from ["'](vue|@vue\/server-renderer|vue\/server-renderer)["'];?/g,
    (_, names, owner) =>
      `const {${names.replace(/ as /g, ": ")}} = globalThis.__vizeWhitespace7880.${owner === "vue" ? "vue" : "server"};`,
  );
  assert.ok(!/^import /m.test(linked), "every runtime import must resolve to the one pinned graph");
  const component = (
    await import(`data:text/javascript,${encodeURIComponent(linked)}#${sequence++}`)
  ).default;
  const application = vue.createSSRApp(component);
  const warnings: string[] = [];
  application.config.warnHandler = (message: string) => warnings.push(message);
  return { html: await server.renderToString(application), warnings };
}
