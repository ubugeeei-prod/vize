import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("vize package exposes an executable TypeScript content mapper", () => {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/package.json"), "utf-8"),
  ) as {
    bin?: Record<string, string>;
    exports?: Record<
      string,
      {
        import?: { types?: string };
        require?: { types?: string };
        types?: string;
      }
    >;
    files?: string[];
    tsContentMapper?: unknown;
    typescript?: {
      contentMapper?: {
        dynamicConfig?: boolean;
        compilerOptions?: string[];
        exec?: string[];
      };
    };
  };
  const binPath = path.join(root, "npm/cli/bin/vize");

  assert.deepEqual(packageJson.typescript?.contentMapper, {
    dynamicConfig: true,
    exec: ["node", "./bin/vize", "content-mapper"],
    compilerOptions: [
      "checkJs",
      "noUnusedLocals",
      "rootDir",
      "module",
      "moduleResolution",
      "resolvePackageJsonExports",
      "customConditions",
      "baseUrl",
      "paths",
      "moduleSuffixes",
      "noDtsResolution",
      "preserveSymlinks",
    ],
  });
  assert.equal(
    packageJson.tsContentMapper,
    undefined,
    "the retired tsContentMapper key must not ship alongside typescript.contentMapper",
  );
  assert.equal(packageJson.bin?.vize, "bin/vize");
  assert.ok(packageJson.files?.includes("bin"));
  assert.ok(packageJson.files?.includes("content-mapper-helpers.d.cts"));
  assert.ok(packageJson.files?.includes("content-mapper-helpers.d.mts"));
  assert.ok(packageJson.files?.includes("content-mapper-helpers.d.ts"));
  assert.deepEqual(packageJson.exports?.["./content-mapper-helpers"], {
    import: { types: "./content-mapper-helpers.d.mts" },
    require: { types: "./content-mapper-helpers.d.cts" },
    types: "./content-mapper-helpers.d.ts",
  });
  const helperPaths = [
    "content-mapper-helpers.d.cts",
    "content-mapper-helpers.d.mts",
    "content-mapper-helpers.d.ts",
  ];
  const helpers = fs.readFileSync(path.join(root, "npm/cli", helperPaths[0]), "utf-8");
  for (const helperPath of helperPaths.slice(1)) {
    assert.equal(
      fs.readFileSync(path.join(root, "npm/cli", helperPath), "utf-8"),
      helpers,
      `${helperPath} must stay identical across module formats`,
    );
  }
  assert.match(helpers, /export type VizeComponentAttrCamel</u);
  assert.match(helpers, /export type VizeComponentGlobalHtmlAttrs/u);
  assert.notEqual(
    fs.statSync(binPath).mode & 0o111,
    0,
    "content mapper entrypoint must be executable",
  );
  assert.match(fs.readFileSync(binPath, "utf-8"), /^#!\/usr\/bin\/env node\n/);
});

test("content mapper helpers resolve for Node16 ESM and CommonJS consumers", () => {
  const result = spawnSync(
    process.execPath,
    [
      path.join(root, "npm/cli/node_modules/typescript/bin/tsc"),
      "--ignoreConfig",
      "--noEmit",
      "--strict",
      "--target",
      "ES2022",
      "--module",
      "Node16",
      "--moduleResolution",
      "Node16",
      "npm/cli/tests/content-mapper-helpers.test-d.ts",
      "npm/cli/tests/content-mapper-helpers.test-d.cts",
    ],
    { cwd: root, encoding: "utf8" },
  );
  assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
});

test("VS Code discovers Vize through the trusted native preview command", () => {
  const discovery = fs.readFileSync(
    path.join(root, "editors/vscode/src/content-mapper-discovery.ts"),
    "utf-8",
  );

  assert.match(discovery, /extensions\.getExtension\("TypeScriptTeam\.native-preview"\)/);
  assert.match(
    discovery,
    /commands\.executeCommand\(\s*"typescript\.native-preview\.discoverContentMappers"/,
  );
  assert.match(discovery, /workspace\.isTrusted/);
  assert.match(discovery, /extensions: \["\.vue"\]/);
});

test("published recursive emit helper matches the canonical native declaration", () => {
  const ts = createRequire(path.join(root, "npm/cli/package.json"))("typescript");
  const canonical = fs.readFileSync(
    path.join(
      root,
      "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt",
    ),
    "utf8",
  );
  const published = fs.readFileSync(
    path.join(root, "npm/cli/virtual-ts-emit-overloads.d.ts"),
    "utf8",
  );
  function signature(source: string): string[][] {
    const file = ts.createSourceFile("helpers.d.ts", source, ts.ScriptTarget.Latest, true);
    assert.deepEqual(file.parseDiagnostics, []);
    const pending = [file];
    const aliases = [];
    while (pending.length > 0) {
      const node = pending.pop()!;
      if (ts.isTypeAliasDeclaration(node) && node.name.text === "$OverloadUnionRecursive") {
        aliases.push(node);
      }
      pending.push(...node.getChildren(file));
    }
    assert.equal(aliases.length, 1);
    const alias = aliases[0];
    const parts = [...alias.typeParameters, alias.type];
    const result: string[][] = [];
    for (const part of parts) {
      const scanner = ts.createScanner(
        ts.ScriptTarget.Latest,
        true,
        ts.LanguageVariant.Standard,
        part.getText(file),
      );
      const tokens: string[] = [];
      for (
        let kind = scanner.scan();
        kind !== ts.SyntaxKind.EndOfFileToken;
        kind = scanner.scan()
      ) {
        tokens.push(
          `${kind}:${kind === ts.SyntaxKind.StringLiteral ? scanner.getTokenValue() : scanner.getTokenText()}`,
        );
      }
      result.push(tokens);
    }
    return result;
  }
  assert.deepEqual(signature(published), signature(canonical));
  // These regressions must be visible even when formatting and quote style differ.
  assert.notDeepEqual(
    signature(published.replace("extends 120", "extends 32")),
    signature(canonical),
  );
  assert.notDeepEqual(
    signature(published.replace("TAccumulated = never", "TAccumulated = unknown")),
    signature(canonical),
  );
});
