import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { test } from "node:test";

const require = createRequire(import.meta.url);
const root = process.cwd();

function toPosixPath(value: string): string {
  return value.split(path.sep).join("/");
}

test("defineConfig accepts the documented vue.version section", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-types-"));

  try {
    const sourceIndex = toPosixPath(
      path.relative(tempDir, path.join(root, "npm", "cli", "src", "index.ts")),
    );
    fs.writeFileSync(
      path.join(tempDir, "tsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            allowImportingTsExtensions: true,
            baseUrl: ".",
            ignoreDeprecations: "6.0",
            module: "NodeNext",
            moduleResolution: "NodeNext",
            noEmit: true,
            paths: {
              vize: [sourceIndex],
            },
            skipLibCheck: true,
            strict: true,
            target: "ES2022",
            typeRoots: [path.join(root, "node_modules", "@types")],
            types: ["node"],
          },
          include: ["vize.config.ts"],
        },
        null,
        2,
      ),
    );
    fs.writeFileSync(
      path.join(tempDir, "vize.config.ts"),
      [
        'import { defineConfig, type UserConfigInput } from "vize";',
        "",
        "const scopedEntries: UserConfigInput = [",
        "  {",
        '    files: ["legacy/**/*.vue"],',
        "    vue: { version: '2.7' },",
        "  },",
        "];",
        "",
        "void scopedEntries;",
        "",
        "// @ts-expect-error unsupported Vue versions must stay rejected.",
        "const invalidRootVersion: UserConfigInput = { vue: { version: '4' } };",
        "// @ts-expect-error unsupported scoped Vue versions must stay rejected.",
        "const invalidEntryVersion: UserConfigInput = [{ vue: { version: '4' } }];",
        "void invalidRootVersion;",
        "void invalidEntryVersion;",
        "",
        "export default defineConfig(({ command }) => ({",
        "  vue: {",
        "    version: command === 'build' ? '2' : '3',",
        "  },",
        "  entries: [",
        "    {",
        '      files: ["src/**/*.vue"],',
        "      vue: { version: '2' },",
        "    },",
        "  ],",
        "}));",
        "",
      ].join("\n"),
    );

    const result = spawnSync(
      process.execPath,
      [require.resolve("typescript/bin/tsc"), "-p", tempDir, "--pretty", "false"],
      { encoding: "utf8" },
    );

    assert.equal(
      result.status,
      0,
      `${result.error?.message ?? ""}\n${result.stderr}\n${result.stdout}`.trim(),
    );
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});
