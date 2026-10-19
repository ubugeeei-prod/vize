// Experimental flag docs are checked against the flag names declared in code, not their wording.
import assert from "node:assert/strict";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";

const root = resolve(import.meta.dirname, "../..");
const content = join(root, "docs/content");
const read = (path: string) => readFileSync(join(root, path), "utf8");
const require = createRequire(join(root, "docs/package.json"));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");

/** Members of an exported interface with their doc comments, read from the Vite plugin types. */
function interfaceMembers(name: string) {
  const types = read("npm/builder/vite/src/experimental-options.ts");
  const body = types.match(new RegExp(`export interface ${name}\\b[^{]*\\{([\\s\\S]*?)^\\}`, "m"));
  assert.ok(body, `${name} is declared`);
  return [...body[1].matchAll(/(\/\*\*[\s\S]*?\*\/)?\s*("[^"]+"|\w+)\?:/g)].map(
    ([, comment = "", key]) => ({ key: key.replaceAll('"', ""), comment }),
  );
}

const switches = interfaceMembers("ExperimentalOptions");
const compileFlags = interfaceMembers("ExperimentalCompileFlags").map(({ key }) => key);
const canonical = switches.filter(({ comment }) => !/\balias\b/i.test(comment));
const rfcFlags = switches.flatMap(({ key, comment }) => {
  const rfc = comment.match(/Vue RFC #(\d+)/)?.[1];
  if (!rfc) return [];
  const field = compileFlags.find((flag) =>
    flag.startsWith(`experimental${key[0].toUpperCase()}${key.slice(1)}`),
  );
  assert.ok(field, `${key} has a direct compiler field`);
  return [{ key, rfc, field }];
});

const page = (locale: string, name: string) =>
  read(`docs/content/${locale === "en" ? "" : `${locale}/`}guide/${name}.md`);
/** The name appears as a whole identifier, in prose or in an example. */
const documents = (source: string, name: string) =>
  new RegExp(`(?<![\\w$])${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}(?![\\w$])`).test(source);

test("experimental switches and compile fields are read from code", () => {
  assert.ok(switches.length > 0 && compileFlags.length > 0 && rfcFlags.length > 0);
  assert.ok(canonical.length < switches.length, "compatibility aliases are recognised");
});

for (const locale of ["en", "ja"]) {
  test(`${locale}: experimentals guide documents every switch and compile field`, () => {
    const guide = page(locale, "experimentals");
    for (const { key } of switches) assert.ok(documents(guide, key), `${key} is documented`);
    for (const field of compileFlags) assert.ok(documents(guide, field), `${field} is documented`);
    for (const { rfc } of rfcFlags)
      assert.ok(guide.includes(`https://github.com/vuejs/rfcs/pull/${rfc}`), `RFC #${rfc} linked`);
  });

  test(`${locale}: Vue RFC details document each RFC flag and its compile field`, () => {
    const details = page(locale, "experimentals-vue-rfcs");
    for (const { key, rfc, field } of rfcFlags) {
      assert.ok(documents(details, key), `${key} is documented`);
      assert.ok(documents(details, field), `${field} is documented`);
      assert.ok(
        details.includes(`https://github.com/vuejs/rfcs/pull/${rfc}`),
        `RFC #${rfc} linked`,
      );
    }
  });

  test(`${locale}: experimentals reference documents every canonical switch and compile field`, () => {
    const reference = page(locale, "experimentals-reference");
    for (const { key } of canonical) assert.ok(documents(reference, key), `${key} is documented`);
    for (const field of compileFlags)
      assert.ok(documents(reference, field), `${field} is documented`);
  });

  test(`${locale}: Vite plugin guide lists the experimentals option and links the guide`, () => {
    const vitePlugin = page(locale, "vite-plugin");
    assert.match(vitePlugin, /^\| `experimentals`/m);
    assert.match(vitePlugin, /\]\(\.\/experimentals\.md(?:#[^)]*)?\)/);
  });
}

test("experimentals tables keep one cell per column", () => {
  for (const locale of ["en", "ja"]) {
    for (const name of ["experimentals", "experimentals-vue-rfcs", "experimentals-reference"]) {
      const source = page(locale, name).replace(/^```[\s\S]*?^```/gm, "");
      for (const table of source.match(/(?:^\|.*\|[ \t]*\n)+/gm) ?? []) {
        const cells = (row: string) => row.trim().split(/(?<!\\)\|/).length;
        const [header, ...rows] = table.trimEnd().split("\n");
        for (const row of rows)
          assert.equal(cells(row), cells(header), `${locale}/${name}: ${row.slice(0, 60)}`);
      }
    }
  }
});

test("anchors linked into the experimentals pages exist", () => {
  const walk = (directory: string): string[] =>
    readdirSync(directory).flatMap((name) => {
      const path = join(directory, name);
      if (statSync(path).isDirectory()) return walk(path);
      return path.endsWith(".md") ? [path] : [];
    });
  const ids = new Map<string, string[]>();
  let checked = 0;
  for (const path of walk(content)) {
    if (/\/content\/(?:generated|i18n|fr|zh-CN|pt-BR)\//.test(path)) continue;
    const source = readFileSync(path, "utf8").replace(/^```[\s\S]*?^```/gm, "");
    for (const [, target, fragment] of source.matchAll(
      /\]\(((?:\.{1,2}\/)*(?:[\w-]+\/)*experimentals(?:-vue-rfcs|-reference)?\.md)#([^)\s]+)\)/g,
    )) {
      const file = resolve(dirname(path), target);
      if (!ids.has(file)) {
        const rendered = native.transform(readFileSync(file, "utf8"), {});
        assert.deepEqual(rendered.errors, []);
        ids.set(
          file,
          [...rendered.html.matchAll(/\bid="([^"]+)"/g)].map(([, id]) => id),
        );
      }
      assert.ok(
        ids.get(file)!.includes(decodeURIComponent(fragment)),
        `${path.slice(root.length + 1)} links to missing ${target}#${fragment}`,
      );
      checked += 1;
    }
  }
  assert.ok(checked > 0);
});

test("CLI experimental flag notes cover every switch and link existing guide pages", () => {
  const cliDoc = read("npm/cli/docs/experimental-vue-rfc-flags.md");
  for (const { key } of canonical) assert.ok(documents(cliDoc, key), `${key} is documented`);
  for (const [, route] of cliDoc.matchAll(/https:\/\/vizejs\.dev\/(guide\/[\w-]+)/g))
    readFileSync(join(content, `${route}.md`));
});
