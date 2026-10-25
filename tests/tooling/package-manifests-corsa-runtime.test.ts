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

test("published CLI and editor do not install or constrain a user's compiler", () => {
  for (const relative of ["npm/cli/package.json", "editors/vscode/package.json"]) {
    const packageJson = JSON.parse(fs.readFileSync(path.join(root, relative), "utf-8")) as Record<
      string,
      Record<string, unknown> | undefined
    >;
    for (const name of [
      "@typescript/native-preview",
      "typescript",
      ...TYPESCRIPT_PLATFORM_PACKAGES,
    ]) {
      for (const section of [
        "dependencies",
        "optionalDependencies",
        "peerDependencies",
        "peerDependenciesMeta",
      ]) {
        assert.equal(packageJson[section]?.[name], undefined, `${relative}: ${section}.${name}`);
      }
    }
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

test("the private workspace retains its immutable comparison runtime", () => {
  const packageJson = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf-8")) as {
    private?: boolean;
    optionalDependencies?: Record<string, string>;
  };
  assert.equal(packageJson.private, true);
  for (const name of TYPESCRIPT_PLATFORM_PACKAGES) {
    assert.equal(packageJson.optionalDependencies?.[name], "catalog:corsa-runtime");
  }
});

test("published user guidance does not mention the retired native preview runtime", () => {
  for (const relative of USER_GUIDANCE_FILES) {
    const contents = fs.readFileSync(path.join(root, relative), "utf-8");
    assert.doesNotMatch(contents, /@typescript\/native-preview/u, relative);
    assert.doesNotMatch(contents, /\bnative[- ]preview\b/iu, relative);
  }
});
