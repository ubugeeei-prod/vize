import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

interface OrderMessage {
  ruleId: string;
  severity: number;
  range: [number, number];
  fix: { range: [number, number]; text: string };
}

interface OrderRecording {
  messages: OrderMessage[];
  output: string;
  fixed: boolean;
  secondPassMessageCount: number;
  secondPassOutput: string;
  secondPassFixed: boolean;
}

const fixtureDir = join(
  dirname(fileURLToPath(import.meta.url)),
  "..",
  "test",
  "nuxt-eslint-compat",
  "fixtures",
);
const corpus = JSON.parse(readFileSync(join(fixtureDir, "corpus.json"), "utf8")) as {
  nuxtConfigKeysOrderCases: Array<{ id: string; source: string }>;
};
const recording = JSON.parse(readFileSync(join(fixtureDir, "nuxt-eslint-output.json"), "utf8")) as {
  nuxtConfigKeysOrderCases: Record<string, OrderRecording>;
};

void test("recording covers exactly the config order rule corpus", () => {
  assert.deepEqual(
    Object.keys(recording.nuxtConfigKeysOrderCases).sort(),
    corpus.nuxtConfigKeysOrderCases.map((entry) => entry.id).sort(),
  );
});

for (const entry of corpus.nuxtConfigKeysOrderCases) {
  void test(`${entry.id}: recorded fix converges and is idempotent`, () => {
    const recorded = recording.nuxtConfigKeysOrderCases[entry.id];
    assert.ok(recorded);
    assert.equal(recorded.fixed, recorded.messages.length > 0);
    assert.equal(recorded.secondPassMessageCount, 0);
    assert.equal(recorded.secondPassOutput, recorded.output);
    assert.equal(recorded.secondPassFixed, false);
    for (const message of recorded.messages) {
      assert.equal(message.ruleId, "nuxt/nuxt-config-keys-order");
      assert.equal(message.severity, 2);
      assert.ok(message.range[0] < message.range[1]);
      assert.ok(message.fix.range[0] < message.fix.range[1]);
    }
  });
}
