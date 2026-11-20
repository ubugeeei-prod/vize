// The focused entry packages under npm/entry each expose one slice of the
// `vize` release. Every command, value and type they name must exist in the
// package they forward to, so a slice can never drift from its source.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const entryRoot = path.join(root, "npm/entry");

type Manifest = {
  name: string;
  version: string;
  private?: boolean;
  bin?: Record<string, string>;
  files?: string[];
  exports?: Record<string, unknown>;
  dependencies?: Record<string, string>;
  repository?: { directory?: string };
  typescript?: { contentMapper?: { exec?: string[] } & Record<string, unknown> };
};

function read(relative: string): string {
  return fs.readFileSync(path.join(root, relative), "utf8");
}

function manifest(relative: string): Manifest {
  return JSON.parse(read(relative)) as Manifest;
}

type NamedReexport = {
  source: string;
  imported: string;
  exported: string;
  typeOnly: boolean;
};

function withoutComments(source: string): string {
  return source.replace(/\/\*[\s\S]*?\*\/|\/\/[^\r\n]*/g, "");
}

function namedReexports(source: string): NamedReexport[] {
  const code = withoutComments(source);
  const declaration = /^export\s+(type\s+)?\{([^}]*)\}\s+from\s+(["'])([^"']+)\3\s*;?/gm;
  const exports: NamedReexport[] = [];
  let cursor = 0;
  for (const match of code.matchAll(declaration)) {
    assert.equal(code.slice(cursor, match.index).trim(), "", "unexpected declaration text");
    const outerTypeOnly = match[1] !== undefined;
    for (const item of match[2]
      .split(",")
      .map((name) => name.trim())
      .filter(Boolean)) {
      const parsed = item.match(/^(type\s+)?(\w+)(?:\s+as\s+(\w+))?$/);
      assert.ok(parsed, `invalid re-export item: ${item}`);
      exports.push({
        source: match[4],
        imported: parsed[2],
        exported: parsed[3] ?? parsed[2],
        typeOnly: outerTypeOnly || parsed[1] !== undefined,
      });
    }
    cursor = (match.index ?? 0) + match[0].length;
  }
  assert.equal(code.slice(cursor).trim(), "", "unexpected declaration text");
  return exports;
}

function starReexports(source: string): string[] {
  const code = withoutComments(source);
  const declaration = /^export\s+\*\s+from\s+(["'])([^"']+)\1\s*;?/gm;
  const exports: string[] = [];
  let cursor = 0;
  for (const match of code.matchAll(declaration)) {
    assert.equal(code.slice(cursor, match.index).trim(), "", "unexpected forwarding text");
    exports.push(match[2]);
    cursor = (match.index ?? 0) + match[0].length;
  }
  assert.equal(code.slice(cursor).trim(), "", "unexpected forwarding text");
  return exports;
}

const entries = fs
  .readdirSync(entryRoot)
  .filter((name) => fs.existsSync(path.join(entryRoot, name, "package.json")))
  .toSorted();
const cli = manifest("npm/cli/package.json");
const nativeTypes = read("npm/native/index.d.ts");

/** `vize` subcommands, as clap names the `Commands` variants in kebab case. */
const commands = new Set(
  [...read("crates/vize/src/cli.rs").matchAll(/^ {4}([A-Z][A-Za-z]+)\(crate::commands::/gm)].map(
    ([, variant]) => variant.replace(/(?<!^)[A-Z]/g, (c) => `-${c}`).toLowerCase(),
  ),
);

test("entry re-exports compare declarations rather than formatting", () => {
  assert.deepEqual(
    namedReexports(
      "// retained context\nexport type { Input as PublicInput } from '@vizejs/native';\n",
    ),
    [
      {
        source: "@vizejs/native",
        imported: "Input",
        exported: "PublicInput",
        typeOnly: true,
      },
    ],
  );
  assert.deepEqual(starReexports("/* forwarding entry */\nexport * from '@vizejs/fresco'\n"), [
    "@vizejs/fresco",
  ]);
});

test("every requested entry package exists once under npm/entry", () => {
  assert.deepEqual(entries, [
    "compile",
    "compile-jsx",
    "compile-tsrx",
    "content-mapper",
    "curator",
    "doctor",
    "fmt",
    "lint",
    "lsp",
    "tui",
    "typecheck",
  ]);
  const workspace = read("pnpm-workspace.yaml");
  assert.match(workspace, /^ {2}- "npm\/entry\/\*"$/m);
});

test("entry manifests are public, versioned with the CLI and ship every listed file", () => {
  for (const entry of entries) {
    const dir = `npm/entry/${entry}`;
    const pkg = manifest(`${dir}/package.json`);
    assert.equal(pkg.name, `@vizejs/${entry}`);
    assert.equal(pkg.private, undefined, `${pkg.name} must publish`);
    assert.equal(pkg.version, cli.version, `${pkg.name} must release with vize`);
    assert.equal(pkg.repository?.directory, dir);
    for (const file of pkg.files ?? []) {
      assert.ok(fs.existsSync(path.join(root, dir, file)), `${pkg.name} lists missing ${file}`);
    }
    for (const spec of Object.values(pkg.dependencies ?? {})) {
      assert.equal(spec, "workspace:*", `${pkg.name} must pin the same release`);
    }
  }
});

test("entry commands forward to an existing vize subcommand", () => {
  assert.deepEqual(cli.exports?.["./cli"], { import: "./dist/cli.mjs", default: "./dist/cli.mjs" });
  assert.deepEqual(cli.exports?.["./content-mapper-cli"], {
    import: "./dist/content-mapper-cli.mjs",
    default: "./dist/content-mapper-cli.mjs",
  });
  const pack = read("npm/cli/vite.config.ts");
  assert.match(pack, /"src\/cli\.ts"/);
  assert.match(pack, /"src\/content-mapper-cli\.ts"/);

  for (const entry of entries) {
    const dir = `npm/entry/${entry}`;
    const pkg = manifest(`${dir}/package.json`);
    for (const [bin, target] of Object.entries(pkg.bin ?? {})) {
      assert.equal(target, `bin/${bin}`);
      assert.equal(pkg.dependencies?.vize, "workspace:*", `${bin} runs the vize CLI`);
      const source = read(`${dir}/${target}`);
      assert.ok(source.startsWith("#!/usr/bin/env node\n"), `${bin} must be a node script`);
      const mode = fs.statSync(path.join(root, dir, target)).mode;
      assert.ok(mode & 0o111, `${bin} must be executable`);
      if (entry === "content-mapper") {
        assert.match(source, /^import\("vize\/content-mapper-cli"\);$/m);
        continue;
      }
      const forwarded = source.match(/^process\.argv\.splice\(2, 0, "([a-z-]+)"\);$/m)?.[1];
      assert.ok(forwarded && commands.has(forwarded), `${bin} must forward to a vize subcommand`);
      assert.match(source, /^import\("vize\/cli"\);$/m);
    }
  }
});

test("entry Node APIs re-export native values and types that exist", () => {
  for (const entry of entries) {
    const dir = `npm/entry/${entry}`;
    if (entry === "tui" || !fs.existsSync(path.join(root, dir, "index.js"))) continue;
    const pkg = manifest(`${dir}/package.json`);
    assert.equal(pkg.dependencies?.["@vizejs/native"], "workspace:*");
    const values = [...read(`${dir}/index.js`).matchAll(/^export const (\w+) = native\.(\w+);$/gm)];
    assert.ok(values.length > 0, `${pkg.name} exports nothing`);
    for (const [, local, imported] of values) assert.equal(local, imported);
    const dts = read(`${dir}/index.d.ts`);
    const reexports = namedReexports(dts);
    assert.ok(reexports.every(({ source }) => source === "@vizejs/native"));
    const declared = reexports.filter(({ typeOnly }) => !typeOnly);
    assert.deepEqual(
      declared.map(({ exported }) => exported),
      values.map(([, name]) => name),
      `${pkg.name} types and values diverge`,
    );
    for (const { imported, exported } of declared) {
      assert.equal(imported, exported, `${pkg.name} must not rename native value ${imported}`);
      assert.match(
        nativeTypes,
        new RegExp(`^export declare (?:function|class) ${imported}\\b`, "m"),
        `${pkg.name} re-exports missing native value ${imported}`,
      );
    }
    for (const { imported } of reexports.filter(({ typeOnly }) => typeOnly)) {
      assert.match(
        nativeTypes,
        new RegExp(
          `^export (?:interface|type|declare const enum|declare enum) ${imported}\\b`,
          "m",
        ),
        `${pkg.name} re-exports missing native type ${imported}`,
      );
    }
  }
});

test("the content-mapper entry declares the same mapper as vize", () => {
  const pkg = manifest("npm/entry/content-mapper/package.json");
  const { exec, ...declared } = pkg.typescript?.contentMapper ?? {};
  const { exec: cliExec, ...expected } = cli.typescript?.contentMapper ?? {};
  assert.deepEqual(declared, expected);
  assert.deepEqual(cliExec, ["node", "./bin/vize", "content-mapper"]);
  assert.deepEqual(exec, ["node", "./bin/vize-content-mapper"]);
});

test("@vizejs/tui mirrors every @vizejs/fresco entry point", () => {
  const fresco = manifest("npm/fresco/package.json");
  const tui = manifest("npm/entry/tui/package.json");
  assert.equal(tui.dependencies?.["@vizejs/fresco"], "workspace:*");
  const frescoPaths = Object.keys(fresco.exports ?? {});
  assert.deepEqual(
    Object.keys(tui.exports ?? {}).filter((subpath) => subpath !== "./package.json"),
    frescoPaths,
  );
  for (const subpath of frescoPaths) {
    const stem = subpath === "." ? "index" : subpath.slice(2);
    const target = subpath === "." ? "@vizejs/fresco" : `@vizejs/fresco/${stem}`;
    for (const extension of ["js", "d.ts"]) {
      assert.deepEqual(starReexports(read(`npm/entry/tui/${stem}.${extension}`)), [target]);
    }
  }
});
