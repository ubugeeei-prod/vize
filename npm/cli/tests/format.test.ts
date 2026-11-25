import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "vite-plus/test";

import { format, formatSync } from "../src/format.ts";

function project(run: (root: string) => Promise<void>): Promise<void> {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-format-api-")));
  fs.writeFileSync(path.join(root, "package.json"), '{ "type": "module" }\n');
  fs.writeFileSync(
    path.join(root, "vite.config.mjs"),
    "export default { vue: { formatter: { semi: false } } };\n",
  );
  return run(root).finally(() => fs.rmSync(root, { recursive: true, force: true }));
}

test("format follows the configuration vize fmt reads for the file", async () => {
  await project(async (root) => {
    // The target directory need not exist yet.
    const filename = path.join(root, "src/generated/routes.ts");
    assert.deepEqual(await format("const a = {b:1};", { filename }), {
      code: "const a = { b: 1 }\n",
      changed: true,
    });
    assert.deepEqual(formatSync("const a = { b: 1 }\n", { filename }), {
      code: "const a = { b: 1 }\n",
      changed: false,
    });
    // Options override the configuration; `config: false` skips it.
    assert.equal(
      (await format("const a = 1", { filename, semi: true, singleQuote: true })).code,
      "const a = 1;\n",
    );
    assert.equal((await format("const a = 1", { filename, config: false })).code, "const a = 1;\n");
    assert.equal(
      (await format("const a = 1", { filename: "a.ts", config: root })).code,
      "const a = 1\n",
    );
    assert.equal(fs.existsSync(path.join(root, "src")), false);
  });
});

test("format selects the formatter from the file name", async () => {
  const config = false as const;
  assert.equal(
    (await format("<template><div/></template>", { filename: "App.vue", config })).code,
    "<template>\n  <div />\n</template>\n",
  );
  assert.equal((await format('{"a":1}', { filename: "data.json", config })).code, '{ "a": 1 }\n');
  await assert.rejects(format("# notes", { filename: "notes.md", config }), /notes\.md/);
  await assert.rejects(
    format(".a{}", { filename: "theme.css", config }),
    /^Error: theme\.css: unsupported file type; expected \.vue, /,
  );
  await assert.rejects(format("const = ;", { filename: "broken.ts", config }), /broken\.ts/);
  await assert.rejects(
    format("a", { filename: "a.ts", config, quoteProps: "keep" as "preserve" }),
    {
      code: "InvalidArg",
      message: "a.ts: Unknown quoteProps; expected 'as-needed', 'consistent', or 'preserve'",
    },
  );
  assert.throws(() => formatSync("a", { filename: "" }), TypeError);
});
