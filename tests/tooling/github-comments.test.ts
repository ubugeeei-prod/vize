import assert from "node:assert/strict";
import { test } from "node:test";

import {
  isManagedComment as isBenchmarkManagedComment,
  markerForKey as benchmarkMarkerForKey,
} from "../../bench/comment-pr.mjs";
import {
  isManagedComment as isTestReportManagedComment,
  markerForKey as testReportMarkerForKey,
} from "../../bench/comment-test-report.mjs";

function comment(body: string, login = "github-actions[bot]") {
  return {
    body,
    user: {
      login,
      type: "Bot",
    },
  };
}

test("PR benchmark comments only match GitHub Actions comments with the marker as the first line", () => {
  const marker = benchmarkMarkerForKey("abc123");

  assert.equal(isBenchmarkManagedComment(comment(`${marker}\n## PR Benchmark`), marker), true);
  assert.equal(isBenchmarkManagedComment(comment(marker), marker), true);
  assert.equal(
    isBenchmarkManagedComment(comment(`> quoted old report\n${marker}\n## PR Benchmark`), marker),
    false,
  );
  assert.equal(
    isBenchmarkManagedComment(comment(`${marker}\n## PR Benchmark`, "dependabot[bot]"), marker),
    false,
  );
  assert.equal(
    isBenchmarkManagedComment(
      comment("<!-- vize-pr-benchmark:def456 -->\n## PR Benchmark"),
      marker,
    ),
    false,
  );
});

test("PR test report comments only match GitHub Actions comments with the marker as the first line", () => {
  const marker = testReportMarkerForKey("abc123");

  assert.equal(
    isTestReportManagedComment(comment(`${marker}\n## Detailed Test Report`), marker),
    true,
  );
  assert.equal(isTestReportManagedComment(comment(marker), marker), true);
  assert.equal(
    isTestReportManagedComment(
      comment(`Prior discussion mentions ${marker}\n## Detailed Test Report`),
      marker,
    ),
    false,
  );
  assert.equal(
    isTestReportManagedComment(
      comment(`${marker}\n## Detailed Test Report`, "github-actions-user[bot]"),
      marker,
    ),
    false,
  );
  assert.equal(
    isTestReportManagedComment(
      comment("<!-- vize-test-report:def456 -->\n## Detailed Test Report"),
      marker,
    ),
    false,
  );
});
