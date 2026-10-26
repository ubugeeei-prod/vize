import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { createRequire } from "node:module";
import { test } from "node:test";

import { repoRoot } from "../_helpers/realworld-patch.ts";
import { resolveTsgoBinary } from "../_helpers/realworld-typecheck.ts";
import { prepareExperimentalRouterProject } from "./support/router-experimental-provider.ts";

type ReferenceCompiler = {
  version: string;
  ScriptTarget: { Latest: number };
  createSourceFile(
    name: string,
    source: string,
    target: number,
    parents: boolean,
  ): {
    referencedFiles: Array<{ fileName: string; pos: number; end: number }>;
  };
  resolveTripleslashReference(name: string, containingFile: string): string;
};

type Packet = {
  label: string;
  command: string;
  args: string[];
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
  elapsedMs: number;
};

await test(
  "literal reference pragma targets resolve through both actual compilers",
  {
    skip:
      process.platform === "win32"
        ? "quote and angle path components are not legal on Windows"
        : false,
  },
  async () => {
    const output = path.join(repoRoot, "target/differential/reference-path-attributes");
    const provider = await prepareExperimentalRouterProject(output);
    const project = fs.realpathSync(provider.root);
    const require = createRequire(path.join(project, "package.json"));
    const ts = require("typescript") as ReferenceCompiler;
    assert.equal(ts.version, "5.9.3");
    const native = resolveTsgoBinary();
    const packets: Packet[] = [];
    const run = (label: string, command: string, args: string[], cwd: string) => {
      const start = performance.now();
      const result = spawnSync(command, args, {
        cwd,
        encoding: "utf8",
        timeout: 120_000,
        env: { ...process.env, LANG: "C", LC_ALL: "C" },
      });
      const packet: Packet = {
        label,
        command,
        args,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdout: result.stdout ?? "",
        stderr: result.stderr ?? "",
        elapsedMs: performance.now() - start,
      };
      packets.push(packet);
      fs.writeFileSync(
        path.join(output, "compiler-packets.json"),
        `${JSON.stringify(packets, null, 2)}\n`,
      );
      assert.equal(packet.error, null);
      assert.equal(packet.signal, null);
      return packet;
    };
    const variants = [
      {
        name: "ampersand",
        directory: "types&雪",
        header: '/// <reference path="$DECLARATION" />\n',
      },
      { name: "angles", directory: "types<雪>", header: '/// <reference path="$DECLARATION" />\n' },
      {
        name: "apostrophe",
        directory: "types'雪",
        header: '/// <reference path="$DECLARATION" />\n',
      },
      {
        name: "doublequote",
        directory: 'types"雪',
        header: "/// <reference path='$DECLARATION' />\n",
      },
      {
        name: "bothquotes-relative",
        directory: "types'\"雪",
        header: '/// <reference path="./ambient.d.ts" />\n',
      },
    ];
    try {
      const version = run("native-version", native, ["--version"], project);
      assert.equal(version.status, 0);
      assert.equal(version.stderr, "");
      assert.equal(version.stdout, "Version 7.0.2\n");
      for (const variant of variants) {
        const root = path.join(project, variant.directory);
        fs.mkdirSync(root);
        const declaration = path.join(root, "ambient.d.ts");
        const bytes = fs.readFileSync(
          path.join(
            repoRoot,
            "tests/_fixtures/differential/typechecker/literal-reference-paths-10551/fixture-inputs/ambient.d.ts.txt",
          ),
        );
        fs.writeFileSync(declaration, bytes);
        fs.writeFileSync(
          path.join(root, "tsconfig.json"),
          JSON.stringify({
            compilerOptions: {
              strict: true,
              skipLibCheck: true,
              noEmit: true,
              target: "ESNext",
              module: "ESNext",
              types: [],
            },
            files: ["control.ts"],
            include: [],
          }),
        );
        const header = variant.header.replace("$DECLARATION", declaration.replaceAll("\\", "/"));
        const parsed = ts.createSourceFile("control.ts", header, ts.ScriptTarget.Latest, true);
        assert.deepEqual(
          parsed.referencedFiles.map((reference) => ({
            fileName: reference.fileName,
            pos: reference.pos,
            end: reference.end,
          })),
          [
            {
              fileName:
                variant.name === "bothquotes-relative"
                  ? "./ambient.d.ts"
                  : declaration.replaceAll("\\", "/"),
              pos: header.indexOf("path=") + 6,
              end: header.lastIndexOf(" />") - 1,
            },
          ],
        );
        assert.equal(
          ts.resolveTripleslashReference(
            parsed.referencedFiles[0]!.fileName,
            path.join(root, "control.ts"),
          ),
          declaration.replaceAll("\\", "/"),
        );
        for (const [phase, annotation] of [
          ["valid", "number"],
          ["invalid", "string"],
          ["repaired", "number"],
        ] as const) {
          const source = `${header}const value: ${annotation} = referenceToken;\n`;
          fs.writeFileSync(path.join(root, "control.ts"), source);
          fs.writeFileSync(path.join(output, `${variant.name}-${phase}.ts.txt`), source);
          for (const [name, command, prefix] of [
            [
              "reference",
              process.execPath,
              [path.join(project, "node_modules/typescript/bin/tsc")],
            ],
            ["native", native, []],
          ] as const) {
            const packet = run(
              `${variant.name}/${phase}/${name}`,
              command,
              [...prefix, "--project", "tsconfig.json", "--pretty", "false"],
              root,
            );
            assert.equal(packet.status, phase === "invalid" ? (name === "reference" ? 2 : 1) : 0);
            assert.equal(packet.stderr, "");
            assert.equal(
              packet.stdout,
              phase === "invalid"
                ? "control.ts(2,7): error TS2322: Type 'number' is not assignable to type 'string'.\n"
                : "",
            );
            assert.deepEqual(fs.readFileSync(declaration), bytes);
          }
        }
        if (["ampersand", "angles", "doublequote"].includes(variant.name)) {
          const encoded = declaration
            .replaceAll("&", "&amp;")
            .replaceAll('"', "&quot;")
            .replaceAll("<", "&lt;")
            .replaceAll(">", "&gt;");
          const source = `/// <reference path="${encoded}" />\nconst value: number = referenceToken;\n`;
          fs.writeFileSync(path.join(root, "control.ts"), source);
          fs.writeFileSync(path.join(output, `${variant.name}-legacy-entity.ts.txt`), source);
          for (const [name, command, prefix] of [
            [
              "reference",
              process.execPath,
              [path.join(project, "node_modules/typescript/bin/tsc")],
            ],
            ["native", native, []],
          ] as const) {
            const packet = run(
              `${variant.name}/legacy-entity/${name}`,
              command,
              [...prefix, "--project", "tsconfig.json", "--pretty", "false"],
              root,
            );
            assert.equal(packet.status, name === "reference" ? 2 : 1);
            assert.equal(packet.stderr, "");
            assert.equal(
              packet.stdout,
              `control.ts(1,22): error TS6053: File '${encoded}' not found.\ncontrol.ts(2,23): error TS2304: Cannot find name 'referenceToken'.\n`,
            );
          }
        }
      }
    } finally {
      try {
        // Artifact services reject legal Unix filenames containing quotes or
        // angle brackets. Preserve their original names and bytes inside tar.
        const directories = variants
          .map((variant) => variant.directory)
          .filter((directory) => fs.existsSync(path.join(project, directory)));
        if (directories.length > 0) {
          const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
          const inputs = directories.flatMap((directory) =>
            fs
              .readdirSync(path.join(project, directory))
              .sort()
              .map((file) => {
                const relative = path.join(directory, file);
                const bytes = fs.readFileSync(path.join(project, relative));
                return { path: relative, bytes: bytes.length, sha256: sha256(bytes) };
              }),
          );
          const archive = path.join(output, "literal-fixtures.tar");
          const args = ["-cf", archive, "-C", project, ...directories];
          const start = performance.now();
          const result = spawnSync("tar", args, { encoding: "utf8", timeout: 30_000 });
          const packet = {
            command: "tar",
            args,
            status: result.status,
            signal: result.signal,
            error: result.error?.message ?? null,
            stdout: result.stdout,
            stderr: result.stderr,
            elapsedMs: performance.now() - start,
            inputs,
            archiveSha256: fs.existsSync(archive) ? sha256(fs.readFileSync(archive)) : null,
          };
          fs.writeFileSync(
            path.join(output, "fixture-archive-packet.json"),
            `${JSON.stringify(packet, null, 2)}\n`,
          );
          assert.equal(packet.error, null);
          assert.equal(packet.signal, null);
          assert.equal(packet.status, 0);
          assert.equal(packet.stdout, "");
          assert.equal(packet.stderr, "");
          assert.notEqual(packet.archiveSha256, null);
          for (const directory of directories) {
            fs.rmSync(path.join(project, directory), { recursive: true });
          }
        }
      } finally {
        provider.cleanup();
      }
    }
  },
);
