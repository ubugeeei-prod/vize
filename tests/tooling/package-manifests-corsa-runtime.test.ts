import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const TYPESCRIPT_PLATFORM_PACKAGES = [
  "@typescript/typescript-darwin-arm64",
  "@typescript/typescript-darwin-x64",
  "@typescript/typescript-linux-arm64",
  "@typescript/typescript-linux-x64",
  "@typescript/typescript-win32-arm64",
  "@typescript/typescript-win32-x64",
];

const USER_GUIDANCE_FILES = [
  "docs/content/guide/content-mapper.md",
  "docs/content/fr/guide/content-mapper.md",
  "docs/content/ja/guide/content-mapper.md",
  "docs/content/pt-BR/guide/content-mapper.md",
  "docs/content/zh-CN/guide/content-mapper.md",
  "npm/cli/README.md",
];

test("Corsa runtime is declared for vize check users", () => {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/package.json"), "utf-8"),
  ) as {
    dependencies?: Record<string, string>;
    optionalDependencies?: Record<string, string>;
    peerDependencies?: Record<string, string>;
    peerDependenciesMeta?: Record<string, { optional?: boolean }>;
  };

  for (const name of ["@typescript/native-preview", "typescript"]) {
    for (const section of [
      "dependencies",
      "optionalDependencies",
      "peerDependencies",
      "peerDependenciesMeta",
    ] as const) {
      assert.equal(packageJson[section]?.[name], undefined);
    }
  }
  for (const name of TYPESCRIPT_PLATFORM_PACKAGES) {
    assert.equal(packageJson.optionalDependencies?.[name], "catalog:corsa-runtime");
  }
});

test("workspace tooling does not depend on the retired native preview package", () => {
  for (const relative of [
    "package.json",
    "editors/vscode/package.json",
    "npm/fresco/package.json",
    "npm/marquette/package.json",
  ]) {
    const packageJson = JSON.parse(fs.readFileSync(path.join(root, relative), "utf-8")) as Record<
      string,
      Record<string, unknown> | undefined
    >;

    for (const section of [
      "dependencies",
      "devDependencies",
      "optionalDependencies",
      "peerDependencies",
      "peerDependenciesMeta",
    ]) {
      assert.equal(
        packageJson[section]?.["@typescript/native-preview"],
        undefined,
        `${relative} ${section} must use TypeScript 7 stable instead of native-preview`,
      );
    }
  }

  assert.doesNotMatch(
    fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf-8"),
    /@typescript\/native-preview/u,
  );
});

test("workspace tooling installs the stable TypeScript 7 native runtime", () => {
  for (const [relative, version] of [
    ["package.json", "catalog:corsa-runtime"],
    ["editors/vscode/package.json", "7.0.2"],
  ] as const) {
    const packageJson = JSON.parse(fs.readFileSync(path.join(root, relative), "utf-8")) as {
      optionalDependencies?: Record<string, string>;
    };

    for (const name of TYPESCRIPT_PLATFORM_PACKAGES) {
      assert.equal(packageJson.optionalDependencies?.[name], version, relative);
    }
  }
});

test("published user guidance does not mention the retired native preview runtime", () => {
  for (const relative of USER_GUIDANCE_FILES) {
    const contents = fs.readFileSync(path.join(root, relative), "utf-8");
    assert.doesNotMatch(contents, /@typescript\/native-preview/u, relative);
    assert.doesNotMatch(contents, /\bnative[- ]preview\b/iu, relative);
  }
});
