import assert from "node:assert/strict";

export const crates = [
  "vize_corsa_core",
  "vize_corsa_jsonrpc",
  "vize_corsa_client",
  "vize_corsa_lsp",
  "vize_corsa_orchestrator",
  "vize_corsa",
] as const;
export type Crate = (typeof crates)[number];
export const schema = "vize-corsa-first-publication-v1";
export const workflow = ".github/workflows/release-corsa-bootstrap.yml";
export const artifactName = "corsa-first-publication";
export const pausePrefix = "<!-- vize-corsa-first-publication:";
export type Candidate = {
  repository: string;
  number: number;
  branch: string;
  head: string;
  cut: string;
  tag: string;
};
export type Publisher = {
  id: number;
  crate: string;
  repository_owner: string;
  repository_owner_id: number;
  repository_name: string;
  workflow_filename: string;
  environment: string | null;
  created_at: string;
};
export type Published = {
  name: Crate;
  version: string;
  checksum: string;
  source: string;
  publisher: Publisher;
};
export type Receipt = {
  schema: typeof schema;
  candidate: Candidate;
  run: number;
  attempt: number;
  qualification: number;
  owner: number;
  ownerLogin: string;
  repositoryOwner: number;
  published: Published[];
  consumer: { mode: "sdk-only"; success: true; receiptSha256: string };
};
export function positive(value: unknown, label: string): asserts value is number {
  assert.ok(
    Number.isSafeInteger(value) && Number(value) > 0,
    `${label}: positive integer required`,
  );
}
export function record(value: unknown): Record<string, unknown> {
  assert.ok(
    value !== null && typeof value === "object" && !Array.isArray(value),
    "object required",
  );
  return value as Record<string, unknown>;
}
export function candidate(value: Candidate) {
  assert.equal(value.repository, "ubugeeei-prod/vize", "only the canonical repository is eligible");
  positive(value.number, "source PR");
  for (const sha of [value.head, value.cut]) assert.match(sha, /^[0-9a-f]{40}$/u);
  assert.notEqual(value.head, value.cut, "H and C must differ");
  assert.match(value.tag, /^v0\.[1-9][0-9]*\.0$/u, "a future minor release is required");
  assert.equal(value.branch, `release/${value.tag}`);
  return value;
}
export function sameCandidate(actual: unknown, expected: Candidate) {
  assert.deepEqual(actual, candidate(expected), "the immutable bootstrap tuple changed");
}
export function pauseMarker(value: Candidate) {
  return `${pausePrefix}${JSON.stringify({ schema, candidate: candidate(value) })} -->`;
}
export function hasPause(body: string, value: Candidate) {
  const markers = body.split("\n").filter((line) => line.startsWith(pausePrefix));
  assert.ok(markers.length <= 1, "duplicate first-publication markers");
  if (markers.length === 0) return false;
  assert.equal(markers[0], pauseMarker(value), "a different H already owns first publication");
  return true;
}
export function withPause(body: string, value: Candidate) {
  return hasPause(body, value) ? body : `${body}\n\n${pauseMarker(value)}\n`;
}
export function assertPublisher(value: unknown, name: Crate, ownerId: number): Publisher {
  const row = record(value);
  positive(row.id, "publisher ID");
  positive(ownerId, "repository owner ID");
  assert.equal(row.crate, name);
  assert.equal(row.repository_owner, "ubugeeei-prod");
  assert.equal(row.repository_owner_id, ownerId);
  assert.equal(row.repository_name, "vize");
  assert.equal(row.workflow_filename, "release.yml");
  assert.equal(row.environment, "crates-io");
  assert.ok(typeof row.created_at === "string" && Number.isFinite(Date.parse(row.created_at)));
  return row as Publisher;
}
export function assertReceipt(value: unknown, expected: Candidate, run: number, attempt: number) {
  const row = record(value);
  assert.equal(row.schema, schema);
  sameCandidate(row.candidate, expected);
  assert.equal(row.run, run);
  assert.equal(row.attempt, attempt);
  positive(row.qualification, "full Check qualification");
  positive(row.owner, "authenticated crates.io user");
  assert.ok(typeof row.ownerLogin === "string" && /^[a-zA-Z0-9-]+$/u.test(row.ownerLogin));
  positive(row.repositoryOwner, "repository owner ID");
  assert.ok(Array.isArray(row.published));
  assert.deepEqual(
    row.published.map((entry) => record(entry).name),
    crates,
    "exact serial six required",
  );
  for (const entry of row.published) {
    const item = record(entry);
    assert.equal(item.version, expected.tag.slice(1));
    assert.equal(item.source, expected.head);
    assert.match(String(item.checksum), /^[0-9a-f]{64}$/u);
    assertPublisher(item.publisher, item.name as Crate, row.repositoryOwner);
  }
  const consumer = record(row.consumer);
  assert.equal(consumer.mode, "sdk-only");
  assert.equal(consumer.success, true);
  assert.match(String(consumer.receiptSha256), /^[0-9a-f]{64}$/u);
  return row as Receipt;
}

/** Unknown/transient responses never become absence or permission to publish. */
export function namespaceResponse(name: Crate, status: number, body: unknown) {
  const row = record(body);
  if (status === 404) {
    assert.ok(
      Array.isArray(row.errors) && row.errors.length > 0,
      "crates.io missing-name schema required",
    );
    return "missing" as const;
  }
  assert.equal(status, 200, `registry lookup failed for ${name}: HTTP ${status}`);
  assert.equal(record(row.crate).name, name, "registry namespace substitution");
  return "existing" as const;
}

/** Missing namespace, or a prior partial attempt, requires a qualified handoff. */
export function decision(states: readonly string[], handoff: boolean) {
  assert.equal(states.length, crates.length);
  assert.ok(states.every((state) => state === "missing" || state === "existing"));
  if (handoff) {
    assert.ok(
      states.every((state) => state === "existing"),
      "handoff namespaces disappeared",
    );
    return "verified-handoff" as const;
  }
  return "pause" as const;
}
