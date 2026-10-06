import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../../", import.meta.url));
const frameworkManifest = path.join(root, "npm/framework/nuxt/package.json");
const framework = createRequire(frameworkManifest);
const runtimeManifest = framework.resolve("nuxt/package.json");
const runtime = createRequire(runtimeManifest);
const { resolveModulePath } = await import(pathToFileURL(runtime.resolve("exsolve")).href);
const devtoolsEntry = resolveModulePath("@nuxt/devtools", {
  from: runtimeManifest,
  conditions: ["node", "import"],
});
const devtools = createRequire(devtoolsEntry);
const actualEditor = fs.realpathSync(devtools.resolve("launch-editor"));
const store = path.join(root, "node_modules/.pnpm");
const entries = fs
  .readdirSync(store)
  .filter((entry) => entry === "launch-editor@2.14.1" || entry.startsWith("launch-editor@2.14.1_"));
assert.ok(entries.length > 0, "the frozen Nuxt editor consumer must be installed");
assert.ok(
  entries.some((entry) =>
    actualEditor.startsWith(path.join(store, entry, "node_modules/launch-editor") + path.sep),
  ),
  "Nuxt devtools must resolve the tested editor instance",
);

for (const entry of entries) {
  test(`${entry} resolves patched shell-quote and retains explicit editor parsing`, () => {
    const directory = path.join(store, entry, "node_modules/launch-editor");
    const editor = createRequire(path.join(directory, "package.json"));
    assert.equal(editor(path.join(directory, "package.json")).version, "2.14.1");
    const quoteManifest = editor.resolve("shell-quote/package.json");
    assert.equal(editor(quoteManifest).version, "1.11.0");
    const shellQuote = editor("shell-quote");
    const guessEditor = editor(path.join(directory, "guess.js"));
    const command = 'code --goto "source folder/App.vue:12:4"';
    const tokens = ["code", "--goto", "source folder/App.vue:12:4"];
    assert.deepEqual(guessEditor(command), tokens);
    assert.deepEqual(shellQuote.parse(command), tokens);
    assert.equal(shellQuote.quote(tokens), "code --goto 'source folder/App.vue:12:4'");
    assert.deepEqual(shellQuote.parse(shellQuote.quote(tokens)), tokens);
    assert.deepEqual(shellQuote.parse("echo hello#reviewed"), [
      "echo",
      "hello",
      { comment: "reviewed" },
    ]);
    assert.equal(
      shellQuote.quote(["echo", { comment: "reviewed" }, "ordinary"]),
      "echo #reviewed ordinary",
    );

    for (const terminator of ["\n", "\r", "\u2028", "\u2029"]) {
      for (const input of [
        ["echo", { comment: "reviewed" }, `${terminator}injected`],
        ["echo", { comment: "reviewed" }, "ordinary", `${terminator}injected`],
        shellQuote.parse("echo hello#reviewed").concat(`${terminator}injected`),
      ]) {
        assert.throws(() => shellQuote.quote(input), {
          name: "TypeError",
          message: "a token after a `comment` must not contain line terminators",
        });
      }
      assert.equal(
        shellQuote.quote([`before${terminator}comment`, { comment: "reviewed" }]),
        `'before${terminator}comment' #reviewed`,
      );
    }
    console.log(
      JSON.stringify({
        editor: actualEditor,
        quote: fs.realpathSync(quoteManifest),
        version: "1.11.0",
      }),
    );
  });
}
