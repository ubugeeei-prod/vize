import assert from "node:assert/strict";
import { test } from "node:test";
import { publishedReceipt } from "../../tools/support/compat/github/docs-deployment-policy.ts";
import {
  build,
  publisherFixture,
  run,
  sourceArtifacts,
  sourceSha,
} from "./support/docs-deployment.ts";

test("generic success and candidate receipts do not publish: only the exact real Pages step qualifies", () => {
  const { publisher, job, pages, receipt } = publisherFixture();
  assert.equal(publishedReceipt(receipt, publisher, job, build(), pages)?.sourceSha, sourceSha);
  assert.equal(
    publishedReceipt(receipt, { ...publisher, conclusion: "failure" }, job, build(), pages)
      ?.sourceSha,
    sourceSha,
    "cleanup failure cannot erase the actual Pages effect",
  );
  assert.equal(
    publishedReceipt(
      receipt,
      publisher,
      { ...job, steps: [{ ...job.steps[0], conclusion: "skipped" }] },
      build(),
      pages,
    ),
    null,
  );
  for (const conclusion of [null, "failure", "cancelled"])
    assert.throws(
      () =>
        publishedReceipt(
          receipt,
          publisher,
          { ...job, steps: [{ ...job.steps[0], conclusion }] },
          build(),
          pages,
        ),
      /uncertain Pages effect/,
    );
  assert.throws(() => publishedReceipt(null, publisher, job, build(), pages), /durable receipt/);
  assert.throws(
    () => publishedReceipt(receipt, publisher, { ...job, run_attempt: 2 }, build(), pages),
    /publisher attempt/,
  );
  assert.throws(
    () =>
      publishedReceipt(
        receipt,
        publisher,
        { ...job, steps: [job.steps[0], job.steps[0]] },
        build(),
        pages,
      ),
    /One actual/,
  );
  assert.throws(
    () =>
      publishedReceipt(
        { ...receipt, build: { ...receipt.build, sourceSha: "d".repeat(40) } },
        publisher,
        job,
        build(),
        pages,
      ),
    /forged source/,
  );
  assert.throws(() =>
    publishedReceipt(
      { ...receipt, publisher: { ...receipt.publisher, jobId: 999 } },
      publisher,
      job,
      build(),
      pages,
    ),
  );
  assert.throws(
    () =>
      publishedReceipt(
        {
          ...receipt,
          pagesArtifact: { ...receipt.pagesArtifact, digest: "sha256:" + "e".repeat(64) },
        },
        publisher,
        job,
        build(),
        pages,
      ),
    /forged Pages artifact/,
  );
  assert.throws(
    () =>
      publishedReceipt(
        receipt,
        { ...publisher, head_repository: { id: 999 } },
        job,
        build(),
        pages,
      ),
    /fork source/,
  );
  assert.throws(
    () =>
      publishedReceipt(receipt, publisher, job, build(), {
        ...pages,
        workflow_run: { ...pages.workflow_run, head_sha: "d".repeat(40) },
      }),
    /Artifact source SHA/,
  );
  const expiredArtifacts = sourceArtifacts().map((artifact) => ({ ...artifact, expired: true }));
  assert.equal(
    publishedReceipt(receipt, publisher, job, build(run(), expiredArtifacts, false), {
      ...pages,
      expired: true,
    })?.sourceSha,
    sourceSha,
    "Durable source identities survive artifact retention; availability never authorizes a new candidate",
  );
});
