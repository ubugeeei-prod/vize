import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import ts from "../../npm/cli/node_modules/typescript/lib/typescript.js";
import { repoRoot } from "./_helpers/tool-commands.ts";
import { findNpmCli } from "../../tools/support/release/computed_event_names.ts";

const sourcePackage = path.join(repoRoot, "npm/cli");
const moduleName = "virtual-ts-emit-overloads.d.ts";
const valid = [
  "import type { __EmitProps } from 'vize/virtual-ts';",
  "import type { __RuntimePropValue as PublicProp } from 'vize/content-mapper-helpers';",
  "type Events = { (e: 'first', value: number): void; <T>(e: 'gen', value: T): void; (e: 'last', value: string): void };",
  "type Handlers = __EmitProps<Events>;",
  "const first: Handlers = { onFirst: (value: number) => {} };",
  "const generic: Handlers = { onGen: (value: unknown) => {} };",
  "const last: Handlers = { onLast: (value: string) => {} };",
  "type Both = { (): 'call'; new(): 'construct' };",
  "const publicCall: PublicProp<Both> = 'call';",
  "const globalCall: __RuntimePropValue<Both> = 'call';",
  "",
].join("\n");
const wrong = valid + "const wrong: Handlers = { onFirst: (value: string) => {} };\n";
const construct = valid + "const wrong: PublicProp<Both> = 'construct';\n";

function command(bin: string, args: string[], cwd: string, env = process.env): string {
  const result = spawnSync(bin, args, { cwd, env, encoding: "utf8" });
  assert.ifError(result.error);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  return result.stdout;
}

test("packed public emit props retain complete diagnostics and require every declaration", async (t) => {
  const scratch = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize public emit package ")),
  );
  try {
    const packed = JSON.parse(
      command(
        process.execPath,
        [findNpmCli(), "pack", "--ignore-scripts", "--json", "--pack-destination", scratch],
        sourcePackage,
        {
          ...process.env,
          npm_config_cache: path.join(scratch, "npm-cache"),
        },
      ),
    )[0] as { filename: string; files: { path: string }[] };
    const consumer = path.join(scratch, "consumer");
    const packageRoot = path.join(consumer, "node_modules/vize");
    fs.mkdirSync(packageRoot, { recursive: true });
    command(
      "tar",
      ["-xzf", path.join(scratch, packed.filename), "--strip-components=1", "-C", packageRoot],
      scratch,
    );
    fs.symlinkSync(
      fs.realpathSync(path.join(repoRoot, "node_modules/vue")),
      path.join(consumer, "node_modules/vue"),
      process.platform === "win32" ? "junction" : "dir",
    );
    fs.writeFileSync(path.join(consumer, "package.json"), JSON.stringify({ type: "module" }));
    const manifest = JSON.parse(
      fs.readFileSync(path.join(sourcePackage, "package.json"), "utf8"),
    ) as {
      files: string[];
      exports: Record<string, unknown>;
    };
    const declarations = manifest.files.filter((file) => /\.d\.(?:ts|mts|cts)$/.test(file));
    assert.ok(declarations.includes(moduleName));

    function physicalClosure() {
      const installed = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8"));
      assert.deepEqual(installed.exports, manifest.exports);
      for (const file of declarations) {
        assert.ok(
          packed.files.some((entry) => entry.path === file),
          `tarball inventory missing ${file}`,
        );
        const installedPath = path.join(packageRoot, file);
        assert.ok(fs.existsSync(installedPath), `package declaration missing ${file}`);
        assert.deepEqual(
          fs.readFileSync(installedPath),
          fs.readFileSync(path.join(sourcePackage, file)),
          `package declaration changed ${file}`,
        );
      }
    }

    for (const extension of ["mts", "cts"] as const) {
      await t.test(
        `NodeNext .${extension} admission rejects missing and foreign module bytes`,
        () => {
          const file = `index.${extension}`;
          const options = {
            strict: true,
            skipLibCheck: true,
            noEmit: true,
            types: [],
            target: ts.ScriptTarget.ES2022,
            module: ts.ModuleKind.NodeNext,
            moduleResolution: ts.ModuleResolutionKind.NodeNext,
          };
          function diagnostics(source: string, skipLibCheck = true) {
            const input = path.join(consumer, file);
            fs.writeFileSync(input, source);
            const program = ts.createProgram([input], { ...options, skipLibCheck });
            return ts.getPreEmitDiagnostics(program).map((diagnostic) => {
              assert.ok(diagnostic.file && diagnostic.start !== undefined);
              const position = diagnostic.file.getLineAndCharacterOfPosition(diagnostic.start);
              return {
                file: path.relative(consumer, diagnostic.file.fileName).replaceAll("\\", "/"),
                line: position.line + 1,
                column: position.character + 1,
                code: diagnostic.code,
                category: diagnostic.category,
                message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
              };
            });
          }
          const wrongPacket = [
            {
              file,
              line: 11,
              column: 27,
              code: 2322,
              category: ts.DiagnosticCategory.Error,
              message:
                "Type '(value: string) => void' is not assignable to type '(value: number) => any'.\n  Types of parameters 'value' and 'value' are incompatible.\n    Type 'number' is not assignable to type 'string'.",
            },
          ];
          function admit() {
            physicalClosure();
            const mode = extension === "mts" ? ts.ModuleKind.ESNext : ts.ModuleKind.CommonJS;
            const resolved = ts.resolveModuleName(
              "vize/virtual-ts",
              path.join(consumer, file),
              options,
              ts.sys,
              undefined,
              undefined,
              mode,
            ).resolvedModule;
            assert.ok(resolved);
            assert.equal(
              fs.realpathSync(resolved.resolvedFileName),
              fs.realpathSync(path.join(packageRoot, `virtual-ts.d.${extension}`)),
            );
            assert.deepEqual(diagnostics(valid), []);
            assert.deepEqual(diagnostics(wrong), wrongPacket);
            assert.deepEqual(diagnostics(construct), [
              {
                file,
                line: 11,
                column: 7,
                code: 2322,
                category: ts.DiagnosticCategory.Error,
                message: "Type '\"construct\"' is not assignable to type '\"call\"'.",
              },
            ]);
            assert.deepEqual(diagnostics(valid, false), []);
          }
          admit();
          const modulePath = path.join(packageRoot, moduleName);
          const original = fs.readFileSync(modulePath, "utf8");
          try {
            fs.renameSync(modulePath, `${modulePath}.missing`);
            // skipLibCheck can erase the public wrapper's error when its helper is absent.
            assert.deepEqual(diagnostics(wrong), []);
            assert.throws(
              () => assert.deepEqual(diagnostics(wrong), wrongPacket),
              assert.AssertionError,
            );
            assert.throws(admit, /package declaration missing/);
            fs.renameSync(`${modulePath}.missing`, modulePath);
            admit();
            const foreign = original.replace("type __EmitOptions<", "type __ForeignEmitOptions<");
            assert.notEqual(foreign, original);
            fs.writeFileSync(modulePath, foreign);
            assert.deepEqual(diagnostics(wrong), []);
            assert.throws(
              () => assert.deepEqual(diagnostics(wrong), wrongPacket),
              assert.AssertionError,
            );
            assert.throws(admit, /package declaration changed/);
          } finally {
            fs.writeFileSync(modulePath, original);
          }
          admit();
        },
      );
    }
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
});
