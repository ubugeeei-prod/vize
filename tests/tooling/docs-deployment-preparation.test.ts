import assert from "node:assert/strict";
import { test } from "node:test";
import * as metadata from "../../tools/support/compat/github/docs-deployment-api.ts";
import { publisherFixture, repositoryId } from "./support/docs-deployment.ts";

const stepName = "Validate completed build and actual Pages publication floor";
const source = "f2f8a250c5b22882c33786a97727557d7c662927";
const observed = [
  [38065064638, 114250930859, "2026-10-10T15:48:23Z"],
  [38065628811, 114252559712, "2026-10-10T15:56:37Z"],
] as const;

function current(startedAt: string | null = null, index = 0) {
  const { publisher, job } = publisherFixture();
  publisher.id = job.run_id = observed[index][0];
  job.id = observed[index][1];
  publisher.head_sha = job.head_sha = source;
  publisher.status = job.status = "in_progress";
  publisher.conclusion = null;
  job.steps = [{ name: stepName, started_at: startedAt, completed_at: null, conclusion: null }];
  return { publisher, job };
}

void test("observed current-step null starts refresh to their authentic primary timestamp", async () => {
  const refresh = metadata.preparationMetadata;
  assert.equal(typeof refresh, "function");
  for (const [index, [, , authenticStart]] of observed.entries()) {
    let reads = 0;
    let pauses = 0;
    const value = await refresh(
      async () => current(++reads < 3 ? null : authenticStart, index),
      repositoryId,
      async () => {
        pauses++;
      },
    );
    assert.equal(value.job.steps[0].started_at, authenticStart);
    assert.equal(reads, 3);
    assert.equal(pauses, 2);
  }
});

void test("visible primary preparation metadata does not delay publication", async () => {
  let reads = 0;
  let pauses = 0;
  const value = await metadata.preparationMetadata(
    async () => {
      reads++;
      return current(observed[0][2]);
    },
    repositoryId,
    async () => {
      pauses++;
    },
  );
  assert.equal(value.job.steps[0].started_at, observed[0][2]);
  assert.equal(reads, 1);
  assert.equal(pauses, 0);
});

void test("exhausted null starts retain the six-snapshot bound and refuse publication", async () => {
  let reads = 0;
  let pauses = 0;
  await assert.rejects(
    () =>
      metadata.preparationMetadata(
        async () => {
          reads++;
          return current();
        },
        repositoryId,
        async () => {
          pauses++;
        },
      ),
    /preparation start remained unavailable/,
  );
  assert.equal(reads, 6);
  assert.equal(pauses, 5);
});

void test("foreign primary jobs and publisher sources cannot use metadata refresh", async () => {
  const mutations = [
    (value: ReturnType<typeof current>) => value.job.run_id++,
    (value: ReturnType<typeof current>) => value.job.run_attempt++,
    (value: ReturnType<typeof current>) => (value.job.head_sha = "a".repeat(40)),
    (value: ReturnType<typeof current>) => value.publisher.repository.id++,
    (value: ReturnType<typeof current>) => value.publisher.head_repository.id++,
    (value: ReturnType<typeof current>) => (value.publisher.head_branch = "feature"),
    (value: ReturnType<typeof current>) => (value.publisher.path = ".github/workflows/other.yml"),
    (value: ReturnType<typeof current>) => (value.publisher.event = "push"),
  ];
  for (const mutate of mutations) {
    let reads = 0;
    await assert.rejects(() =>
      metadata.preparationMetadata(
        async () => {
          reads++;
          const value = current(observed[0][2]);
          mutate(value);
          return value;
        },
        repositoryId,
        async () => assert.fail("Untrusted identity must not retry"),
      ),
    );
    assert.equal(reads, 1);
  }
});

void test("every refreshed snapshot retains the original run, attempt, job and source", async () => {
  const mutations = [
    (value: ReturnType<typeof current>) => (value.publisher.id = ++value.job.run_id),
    (value: ReturnType<typeof current>) => (value.publisher.run_attempt = ++value.job.run_attempt),
    (value: ReturnType<typeof current>) =>
      (value.publisher.head_sha = value.job.head_sha = "a".repeat(40)),
    (value: ReturnType<typeof current>) => value.publisher.workflow_id++,
    (value: ReturnType<typeof current>) => value.job.id++,
  ];
  for (const mutate of mutations) {
    let reads = 0;
    await assert.rejects(
      () =>
        metadata.preparationMetadata(
          async () => {
            const value = current(++reads === 1 ? null : observed[0][2]);
            if (reads > 1) mutate(value);
            return value;
          },
          repositoryId,
          async () => {},
        ),
      /same primary publisher and job/,
    );
    assert.equal(reads, 2);
  }
});

void test("terminal, missing, duplicate or malformed preparation metadata refuses immediately", async () => {
  const mutations = [
    (value: ReturnType<typeof current>) => (value.publisher.status = "completed"),
    (value: ReturnType<typeof current>) => (value.publisher.conclusion = "failure"),
    (value: ReturnType<typeof current>) => (value.job.status = "completed"),
    (value: ReturnType<typeof current>) => (value.job.steps = []),
    (value: ReturnType<typeof current>) => value.job.steps.push({ ...value.job.steps[0] }),
    (value: ReturnType<typeof current>) => (value.job.steps[0].started_at = "not-a-date"),
    (value: ReturnType<typeof current>) => (value.job.steps[0].conclusion = "skipped"),
    (value: ReturnType<typeof current>) => (value.job.steps[0].conclusion = "success"),
    (value: ReturnType<typeof current>) =>
      (value.job.steps[0].completed_at = "2026-10-10T15:56:38Z"),
  ];
  for (const mutate of mutations) {
    let reads = 0;
    await assert.rejects(() =>
      metadata.preparationMetadata(
        async () => {
          reads++;
          const value = current(observed[0][2]);
          mutate(value);
          return value;
        },
        repositoryId,
        async () => assert.fail("Invalid preparation metadata must not retry"),
      ),
    );
    assert.equal(reads, 1);
  }
});

void test("a null start cannot refresh into a terminal publisher, job or preparation step", async () => {
  const mutations = [
    (value: ReturnType<typeof current>) => (value.publisher.status = "completed"),
    (value: ReturnType<typeof current>) => (value.job.status = "completed"),
    (value: ReturnType<typeof current>) => (value.job.steps[0].conclusion = "success"),
  ];
  for (const mutate of mutations) {
    let reads = 0;
    let pauses = 0;
    await assert.rejects(() =>
      metadata.preparationMetadata(
        async () => {
          const value = current(++reads === 1 ? null : observed[0][2]);
          if (reads > 1) mutate(value);
          return value;
        },
        repositoryId,
        async () => {
          pauses++;
        },
      ),
    );
    assert.equal(reads, 2);
    assert.equal(pauses, 1);
  }
});
