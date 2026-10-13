import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { test } from "node:test";

import { runMoonScript } from "./_helpers/moonbit.ts";
import {
  controlledPublishFixture,
  maxActivePublishers,
} from "./support/moonbit-publish-concurrency-fixture.ts";

const receiptPath = process.env.VIZE_PUBLISH_BENCHMARK_RECEIPT;

// Credential-free controlled measurement of the actual native MoonBit
// scheduler. This never invokes npm or the real single-package publisher.
// Example:
// VIZE_PUBLISH_BENCHMARK_RECEIPT=/tmp/native-publish.json node --test \
//   --test-name-pattern='controlled platform publish benchmark' \
//   tests/tooling/moonbit-publish-concurrency-benchmark.test.ts
test("controlled platform publish benchmark", { skip: !receiptPath }, () => {
  const fixture = controlledPublishFixture();
  const samples: Array<{
    repetition: number;
    concurrency: number;
    runner_walltime_ms: number;
    peak_publishers: number;
    publish_receipt: ReturnType<typeof fixture.receipt>;
  }> = [];
  try {
    // Compilation and dependency downloads are outside the measured workload.
    const warmup = runMoonScript("publish_npm_package_dirs", [], {
      buildOnly: true,
      env: fixture.env,
    });
    assert.equal(warmup.status, 0, `${warmup.stderr}\n${warmup.stdout}`);
    for (let repetition = 1; repetition <= 3; repetition++) {
      // Alternate the order to reduce systematic warm-cache or load bias.
      for (const concurrency of repetition % 2 === 1 ? [1, 4] : [4, 1]) {
        const previousEvents = fs.existsSync(
          path.join(path.dirname(fixture.receiptPath), "events.jsonl"),
        )
          ? fixture.events().length
          : 0;
        const started = performance.now();
        const result = runMoonScript(
          "publish_npm_package_dirs",
          [
            fixture.baseDir,
            "--concurrency",
            String(concurrency),
            "--receipt",
            fixture.receiptPath,
            "--provenance",
          ],
          {
            env: { ...fixture.env, MOCK_PUBLISH_DELAY_MS: "1000" },
          },
        );
        const runner_walltime_ms = performance.now() - started;
        assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
        const publish_receipt = fixture.receipt();
        const events = fixture.events().slice(previousEvents);
        const peak_publishers = maxActivePublishers(events);
        assert.equal(peak_publishers, concurrency);
        assert.equal(publish_receipt.package_count, 8);
        assert.equal(publish_receipt.results.length, 8);
        assert.ok(publish_receipt.results.every((item) => item.exit_code === 0));
        samples.push({
          repetition,
          concurrency,
          runner_walltime_ms,
          peak_publishers,
          publish_receipt,
        });
      }
    }
    const median = (values: number[]) =>
      [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
    const baseline = median(
      samples
        .filter((sample) => sample.concurrency === 1)
        .map((sample) => sample.publish_receipt.walltime_ms),
    );
    const candidate = median(
      samples
        .filter((sample) => sample.concurrency === 4)
        .map((sample) => sample.publish_receipt.walltime_ms),
    );
    const receipt = {
      schema_version: 1,
      measurement_kind: "controlled-native-mocked-publisher",
      workload: {
        package_count: 8,
        publisher_delay_ms: 1000,
        repetitions: 3,
        real_npm_publish: false,
      },
      samples,
      comparison: {
        baseline_concurrency: 1,
        candidate_concurrency: 4,
        baseline_median_walltime_ms: baseline,
        candidate_median_walltime_ms: candidate,
        speedup: baseline / candidate,
        reduction_percent: (1 - candidate / baseline) * 100,
      },
      limitation:
        "Controlled registry-wait-like subprocess delays; actual npm registry release speedup requires a subsequent authorized release.",
    };
    fs.mkdirSync(path.dirname(receiptPath!), { recursive: true });
    fs.writeFileSync(receiptPath!, `${JSON.stringify(receipt, null, 2)}\n`);
    console.log(JSON.stringify(receipt.comparison));
  } finally {
    fixture.cleanup();
  }
});
