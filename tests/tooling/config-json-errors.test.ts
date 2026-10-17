import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { loadConfig } from "../../npm/cli/src/config.ts";

/**
 * Characterization tests for `parseJsonConfig` error surfacing.
 *
 * An explicit `configFile: "serialized-config.json"` reads UTF-8 text and
 * hands the raw text to `JSON.parse` (no BOM stripping, no empty-file
 * shortcut). Any parse failure is wrapped as
 * `Failed to parse vize config JSON at <absolute path>: <JSON.parse message>`.
 *
 * Each case writes `serialized-config.json` into a fresh temporary workspace
 * and selects it explicitly, preserving the internal serialized API, then asserts
 * that the wrapped rejection carries the file path (and, where stable, the
 * underlying JSON.parse detail).
 */

async function withJsonConfig<T>(
  write: (configPath: string) => void,
  run: (dir: string, configPath: string) => Promise<T>,
): Promise<T> {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-json-errors-"));
  const configPath = path.join(dir, "serialized-config.json");
  try {
    write(configPath);
    return await run(dir, configPath);
  } finally {
    fs.rmSync(dir, { force: true, recursive: true });
  }
}

test("malformed JSON config rejects with the file path in the message", async () => {
  await withJsonConfig(
    (configPath) => fs.writeFileSync(configPath, "{ not json"),
    async (dir, configPath) => {
      await assert.rejects(
        loadConfig(dir, { configFile: "serialized-config.json" }),
        (error: unknown) => {
          assert.ok(error instanceof Error);
          assert.match(
            error.message,
            /^Failed to parse vize config JSON at .*serialized-config\.json: /,
          );
          // The wrapped message embeds the exact failing file path.
          assert.ok(error.message.includes(configPath));
          return true;
        },
      );
    },
  );
});

test("zero-byte JSON config rejects with 'Unexpected end of JSON input'", async () => {
  await withJsonConfig(
    (configPath) => fs.writeFileSync(configPath, ""),
    async (dir, configPath) => {
      await assert.rejects(
        loadConfig(dir, { configFile: "serialized-config.json" }),
        (error: unknown) => {
          assert.ok(error instanceof Error);
          assert.match(
            error.message,
            /^Failed to parse vize config JSON at .*serialized-config\.json: /,
          );
          assert.ok(error.message.includes(configPath));
          // Empty file is NOT treated as empty config: JSON.parse("") throws.
          assert.match(error.message, /Unexpected end of JSON input/);
          return true;
        },
      );
    },
  );
});

test("whitespace-only JSON config rejects with 'Unexpected end of JSON input'", async () => {
  await withJsonConfig(
    (configPath) => fs.writeFileSync(configPath, "   \n\t  "),
    async (dir, configPath) => {
      await assert.rejects(
        loadConfig(dir, { configFile: "serialized-config.json" }),
        (error: unknown) => {
          assert.ok(error instanceof Error);
          assert.match(
            error.message,
            /^Failed to parse vize config JSON at .*serialized-config\.json: /,
          );
          assert.ok(error.message.includes(configPath));
          assert.match(error.message, /Unexpected end of JSON input/);
          return true;
        },
      );
    },
  );
});

test("JSON config with a UTF-8 BOM rejects (BOM is not stripped before JSON.parse)", async () => {
  const bom = Buffer.from([0xef, 0xbb, 0xbf]);
  await withJsonConfig(
    (configPath) =>
      fs.writeFileSync(
        configPath,
        Buffer.concat([bom, Buffer.from('{"formatter":{"printWidth":88}}')]),
      ),
    async (dir, configPath) => {
      await assert.rejects(
        loadConfig(dir, { configFile: "serialized-config.json" }),
        (error: unknown) => {
          assert.ok(error instanceof Error);
          assert.match(
            error.message,
            /^Failed to parse vize config JSON at .*serialized-config\.json: /,
          );
          assert.ok(error.message.includes(configPath));
          // A leading BOM makes the otherwise-valid object un-parseable, proving
          // config.ts does not strip the BOM before handing text to JSON.parse.
          assert.match(error.message, /is not valid JSON/);
          return true;
        },
      );
    },
  );
});

test("explicit serialized JSON preserves UTF-8 and the exact trailing-content parser diagnostic", async () => {
  const content = '{"name":"日本語"} trailing';
  let expected: Error | undefined;
  try {
    JSON.parse(content);
  } catch (error) {
    assert.ok(error instanceof Error);
    expected = error;
  }
  assert.ok(expected, "the parser control must reject trailing content");
  await withJsonConfig(
    (configPath) => fs.writeFileSync(configPath, content, "utf8"),
    async (dir, configPath) => {
      await assert.rejects(
        loadConfig(dir, { configFile: "serialized-config.json" }),
        (error: unknown) => {
          assert.ok(error instanceof Error);
          assert.equal(
            error.message,
            `Failed to parse vize config JSON at ${configPath}: ${expected.message}`,
          );
          return true;
        },
      );
    },
  );
});
