import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../npm/native/scripts/formatter-history-build.mts";
import {
  captureWholeValue,
  domConsumer,
  hash,
  isRecord,
  loadAmpCases,
  observe,
  sourceProviders,
} from "./support/glyph-ampersand-contract.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
function nativeProvider() {
  const directory = path.join(root, "npm/native");
  const receiptBytes = fs.readFileSync(nativeHistoryReceipt(directory));
  const receipt: unknown = JSON.parse(receiptBytes.toString("utf8"));
  validateNativeHistoryBuild(directory, receipt);
  assert(isRecord(receipt) && isRecord(receipt.source) && isRecord(receipt.frozen));
  assert.equal(receipt.source.head, process.env.GITHUB_SHA);
  assert.equal(typeof receipt.frozen.path, "string");
  const image = receipt.frozen.path as string;
  const imageSha256 = hash(fs.readFileSync(image));
  const require = createRequire(import.meta.url);
  const saved = [
    "VIZE_PREFER_WORKSPACE_BINDING",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
  ].map((key) => [key, process.env[key]] as const);
  let binding: unknown;
  try {
    process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
    process.env.NAPI_RS_NATIVE_LIBRARY_PATH = image;
    delete process.env.NAPI_RS_FORCE_WASI;
    binding = require(path.join(directory, "index.js"));
  } finally {
    for (const [key, value] of saved) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
  assert(isRecord(binding) && typeof binding.formatSfc === "function");
  assert(require.cache[fs.realpathSync(image)]);
  return {
    binding: binding as unknown as {
      formatSfc(source: string, options: Record<string, unknown>): unknown;
    },
    image,
    imageSha256,
    receipt,
    receiptSha256: hash(receiptBytes),
  };
}

test("encoded ampersand DATA survives complete public formatter and real Vue DOM round trips", async () => {
  const provider = nativeProvider();
  const providers = sourceProviders();
  const consumer = domConsumer(providers);
  const corpus = loadAmpCases(root);
  const observations: Record<string, unknown>[] = [];
  const nativeResults: {
    first: ReturnType<typeof observe>;
    second: ReturnType<typeof observe> | undefined;
    third: ReturnType<typeof observe> | undefined;
  }[] = [];
  const output = path.join(
    root,
    "target/differential/glyph-encoded-ampersand-data-6098/native.json",
  );
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const persist = () =>
    fs.writeFileSync(
      output,
      JSON.stringify(
        {
          schema: "vize.glyph.ampersand.public.v1",
          source: process.env.GITHUB_SHA,
          provider: { ...provider, binding: undefined },
          referenceProvider: providers.receipt,
          catalogSha256: corpus.catalogSha256,
          observations,
        },
        null,
        2,
      ) + "\n",
    );
  const stack = Error.stackTraceLimit;
  Error.stackTraceLimit = 0;
  try {
    // Every native/reference/DOM observation is retained before any row verdict.
    for (const row of corpus.cases) {
      const options = {
        printWidth: 100,
        tabWidth: 2,
        singleQuote: false,
        semi: true,
        endOfLine: "lf",
        vueVersion: "3",
      };
      const optionsBefore = structuredClone(options);
      const first = observe(() => provider.binding.formatSfc(row.source, options));
      const code =
        first.kind === "returned" && isRecord(first.value) && typeof first.value.code === "string"
          ? first.value.code
          : undefined;
      const second =
        code === undefined ? undefined : observe(() => provider.binding.formatSfc(code, options));
      const secondCode =
        second?.kind === "returned" &&
        isRecord(second.value) &&
        typeof second.value.code === "string"
          ? second.value.code
          : undefined;
      const third =
        secondCode === undefined
          ? undefined
          : observe(() => provider.binding.formatSfc(secondCode, options));
      nativeResults.push({ first, second, third });
      let reference: unknown;
      try {
        const code = await providers.prettier.format(row.source, corpus.options);
        reference = {
          kind: "returned",
          code,
          repeated: await providers.prettier.format(code, corpus.options),
        };
      } catch (error) {
        reference = { kind: "threw", error: captureWholeValue(error) };
      }
      const consume = async (source: string) => {
        try {
          return { kind: "returned", value: await consumer.consume(source, row) };
        } catch (error) {
          return { kind: "threw", error: captureWholeValue(error) };
        }
      };
      const before = await consume(row.source);
      const after = code === undefined ? undefined : await consume(code);
      const referenceAfter =
        isRecord(reference) && typeof reference.code === "string"
          ? await consume(reference.code)
          : undefined;
      const observation = {
        row,
        options,
        optionsBefore,
        first: captureWholeValue(first),
        second: captureWholeValue(second),
        third: captureWholeValue(third),
        reference,
        before,
        after,
        referenceAfter,
      };
      observations.push(observation);
      persist();
      console.log("glyph-ampersand-packet " + JSON.stringify(observation));
    }
    assert.equal(observations.length, 13);
    assert.equal(hash(fs.readFileSync(provider.image)), provider.imageSha256);
    for (const [index, entry] of observations.entries()) {
      const row = entry.row as (typeof corpus.cases)[number];
      const actual = nativeResults[index];
      assert(actual && actual.first.kind === "returned" && isRecord(actual.first.value));
      assert.equal(typeof actual.first.value.code, "string");
      assert.deepEqual(actual.first.value, {
        code: actual.first.value.code,
        changed: actual.first.value.code !== row.source,
      });
      assert.deepEqual(actual.second, {
        kind: "returned",
        value: { code: actual.first.value.code, changed: false },
      });
      assert.deepEqual(actual.third, actual.second);
      assert.deepEqual(entry.options, entry.optionsBefore);
      const before = entry.before,
        after = entry.after;
      assert(isRecord(before) && before.kind === "returned" && isRecord(before.value));
      assert(isRecord(after) && after.kind === "returned" && isRecord(after.value));
      assert.deepEqual(
        after.value.result,
        before.value.result,
        `${row.id}: complete DOM/lifecycle/data observation`,
      );
      const result = before.value.result;
      assert(isRecord(result) && isRecord(result.outcome));
      const outcome = result.outcome;
      assert.equal(outcome.setups, 1);
      assert.equal(outcome.updates, 1);
      assert.equal(result.unmounts, 1);
      assert.deepEqual(result.warnings, []);
      assert.deepEqual(result.errors, []);
      assert(isRecord(outcome.initial));
      assert.equal(outcome.initial.title, row.head === ":title" ? row.expected.result : null);
      assert.deepEqual(
        outcome.calls,
        row.head === "@click" ? [...row.expected.calls, ...row.expected.calls] : [],
      );
      const reference = entry.reference;
      assert(isRecord(reference));
      if (row.reference.kind === "return") {
        assert.deepEqual(reference, {
          kind: "returned",
          code: row.reference.code,
          repeated: row.reference.code,
        });
        assert(
          isRecord(entry.referenceAfter) &&
            entry.referenceAfter.kind === "returned" &&
            isRecord(entry.referenceAfter.value),
        );
        assert.deepEqual(entry.referenceAfter.value.result, before.value.result);
      } else {
        assert.equal(row.id, "unknown-reference-control");
        assert.equal(reference.kind, "threw", "retain the independent parser refusal");
        assert(isRecord(reference.error));
        assert.equal(reference.error.errorName, "SyntaxError");
      }
      // Complete repeated NAPI objects, including changed, must be fixed points.
      const second = entry.second,
        third = entry.third;
      assert(isRecord(second) && isRecord(third));
      assert.deepEqual(third, second, `${row.id}: complete repeat object`);
    }
  } finally {
    persist();
    Error.stackTraceLimit = stack;
    await consumer.close();
  }
});
