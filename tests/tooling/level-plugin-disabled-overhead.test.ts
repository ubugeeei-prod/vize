import assert from "node:assert/strict";
import { test } from "node:test";

import { corpus, measureDisabledPluginOverhead } from "./fixtures/davinci-plugin-sdk/bench.mjs";

const budgets = {
  compile: { ratio: 1.25, absoluteUsPerFile: 250 },
  lint: { ratio: 1.25, absoluteUsPerFile: 1_000 },
};

test("disabled plugin hosts preserve output without taxing compile or lint hot paths", () => {
  const files = corpus()
    .filter(({ source }) => /<template(?:\s[^>]*)?>[\s\S]*<\/template>/u.test(source))
    .filter(({ source }) => !/<template[^>]*\blang\s*=/u.test(source))
    .slice(0, 12);
  assert.equal(files.length, 12, "the gate requires twelve real Vue corpus files");

  const measured = measureDisabledPluginOverhead(files);
  assert.deepEqual(measured.parity, { compile: true, lint: true });
  assert.equal(measured.warmups, 2);
  assert.equal(measured.samples, 9);

  for (const [name, budget] of Object.entries(budgets)) {
    const observation = measured.observations[name];
    assert.equal(observation.defaultSamplesUs.length, measured.samples, name);
    assert.equal(observation.disabledSamplesUs.length, measured.samples, name);
    assert.ok(
      observation.disabledToDefaultRatio <= budget.ratio ||
        observation.overheadUsPerFile <= budget.absoluteUsPerFile,
      `${name} disabled-plugin overhead exceeded both budgets: ` +
        `${observation.disabledToDefaultRatio.toFixed(3)}x > ${budget.ratio.toFixed(2)}x and ` +
        `${observation.overheadUsPerFile.toFixed(1)}µs/file > ${budget.absoluteUsPerFile}µs/file`,
    );
  }
});
