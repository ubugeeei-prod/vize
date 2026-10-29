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
  "docs/content/guide/cli.md",
  "docs/content/guide/content-mapper.md",
  "docs/content/guide/type-checking.md",
  "docs/content/fr/guide/content-mapper.md",
  "docs/content/ja/guide/content-mapper.md",
  "docs/content/pt-BR/guide/content-mapper.md",
  "docs/content/zh-CN/guide/content-mapper.md",
  "npm/cli/README.md",
];

const checkerDeclaration = (name: string, value: unknown): boolean =>
  name === "typescript" ||
  name === "tsrs" ||
  name.startsWith("@typescript/typescript-") ||
  name.startsWith("@typescript/native-preview") ||
  name.startsWith("@tsrs/") ||
  (typeof value === "string" &&
    /^npm:(?:typescript|tsrs|@typescript\/(?:typescript-[^@]+|native-preview)|@tsrs\/[^@]+)(?:@|$)/u.test(
      value,
    ));

test("published CLI and editor do not install or constrain a user's compiler", () => {
  for (const relative of ["npm/cli/package.json", "editors/vscode/package.json"]) {
    const packageJson = JSON.parse(fs.readFileSync(path.join(root, relative), "utf-8")) as Record<
      string,
      Record<string, unknown> | undefined
    >;
    for (const section of ["dependencies", "optionalDependencies", "peerDependencies"]) {
      for (const [name, value] of Object.entries(packageJson[section] ?? {})) {
        assert.equal(checkerDeclaration(name, value), false, `${relative}: ${section}.${name}`);
      }
    }
    for (const name of [
      "@typescript/native-preview",
      "typescript",
      ...TYPESCRIPT_PLATFORM_PACKAGES,
    ]) {
      for (const section of ["dependencies", "optionalDependencies", "peerDependenciesMeta"]) {
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

test("the lockfile keeps comparison runtimes only in the private root importer", () => {
  const lock = fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf-8");
  const importer = (name: string, last = false) => {
    const marker = `\n  ${name}:\n`;
    const start = last ? lock.lastIndexOf(marker) : lock.indexOf(marker);
    assert.notEqual(start, -1, `missing ${name} lockfile importer`);
    const bodyStart = start + marker.length;
    const relativeEnd = lock.slice(bodyStart).search(/\n {2}\S/gu);
    return lock.slice(bodyStart, relativeEnd === -1 ? undefined : bodyStart + relativeEnd);
  };
  const workspace = importer(".", true);
  const cli = importer("npm/cli");
  for (const name of TYPESCRIPT_PLATFORM_PACKAGES) {
    const declaration = `      '${name}':\n        specifier: catalog:corsa-runtime\n`;
    assert.ok(workspace.includes(declaration), `private root lock missing ${name}`);
    assert.ok(!cli.includes(declaration), `published CLI lock still declares ${name}`);
  }
});

test("published user guidance does not mention the retired native preview runtime", () => {
  for (const relative of USER_GUIDANCE_FILES) {
    const contents = fs.readFileSync(path.join(root, relative), "utf-8");
    assert.doesNotMatch(contents, /@typescript\/native-preview/u, relative);
    assert.doesNotMatch(contents, /\bnative[- ]preview\b/iu, relative);
  }
  for (const relative of ["docs/content/guide/cli.md", "docs/content/guide/type-checking.md"]) {
    const contents = fs.readFileSync(path.join(root, relative), "utf-8");
    assert.match(contents, /typescript-native@npm:typescript@7\.1\.0-beta/u, relative);
    assert.match(contents, /does not bundle or download/u, relative);
  }
});
