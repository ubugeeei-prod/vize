import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { after } from "node:test";
import { fileURLToPath } from "node:url";
import { publicNative } from "../public_acceptance/installed.ts";
import { assertFiles, digest, rejectSubstitution } from "../musea_static_acceptance/custody.ts";

rejectSubstitution();
const root = fs.realpathSync(process.cwd()),
  output = process.env.PUBLIC_OUTPUT!;
const preparation = JSON.parse(fs.readFileSync(path.join(output, "preparation.json"), "utf8"));
const custody = JSON.parse(fs.readFileSync(path.join(output, "archive-custody.json"), "utf8"));
const { loaded } = publicNative(preparation.version);
assert.equal(root, preparation.consumer);
assert.deepEqual(loaded, custody.loaded);
const entries = ["@vizejs/vite-plugin", "@vizejs/vite-plugin-musea"].map((name) => {
  const entry = fs.realpathSync(fileURLToPath(import.meta.resolve(name)));
  const record = custody.packages.find((item: { name: string }) => item.name === name);
  assert.ok(record && entry.startsWith(record.directory + path.sep));
  assert.ok(
    record.files.some(
      (file: { file: string; sha256: string }) =>
        file.file === path.relative(root, entry) && file.sha256 === digest(fs.readFileSync(entry)),
    ),
  );
  return { name, entry, sha256: digest(fs.readFileSync(entry)) };
});
after(() => {
  assert.deepEqual(publicNative(preparation.version).loaded, loaded);
  for (const record of custody.packages) assertFiles(root, record.files);
  for (const entry of entries) assert.equal(digest(fs.readFileSync(entry.entry)), entry.sha256);
  assertFiles(root, [
    ...preparation.fixtures.records,
    ...preparation.fixtures.adapters,
    ...preparation.harness,
  ]);
  fs.writeFileSync(
    path.join(output, "executed-provider.json"),
    JSON.stringify({ loaded, entries, complete: true }, null, 2) + "\n",
  );
});
// Execute both complete immutable original H laws with installed providers.
await import(new URL("../drivers/literal-variant-keys.browser.test.ts", import.meta.url).href);
