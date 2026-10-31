import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { asRecord } from "../../json-value.ts";

const [root, output] = process.argv.slice(2);
assert.ok(root && output);
const require = createRequire(join(root, "package.json"));
const entry = require.resolve("vize/config");
try {
  const api = asRecord(await import(pathToFileURL(entry).href));
  assert.equal(typeof api.loadConfig, "function");
  const value: unknown = await Reflect.apply(
    api.loadConfig as (...args: unknown[]) => unknown,
    api,
    [root],
  );
  assert.notEqual(value, null, "original fluent config must actually load");
  writeFileSync(output, JSON.stringify({ entry, value }, null, 2) + "\n");
} catch (error) {
  writeFileSync(
    output,
    JSON.stringify(
      {
        entry,
        error:
          error instanceof Error
            ? { name: error.name, message: error.message, stack: error.stack, cause: error.cause }
            : { thrown: String(error) },
      },
      null,
      2,
    ) + "\n",
  );
  throw error;
}
