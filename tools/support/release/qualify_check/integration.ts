import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createIntegrationReadApi } from "./integration-http.ts";
import { createCheckObserver, type CheckedRun, type ReadApi, type Row } from "./check-state.ts";
import { waitForExistingCheck } from "./overlap.ts";

type Context = {
  integration: number;
  source: number;
  H: string;
  C: string;
  tag: string;
  candidate: string;
  base: string;
  integrationHead: string;
  integrationBody: string;
  event: "merge_group";
  catalogEqual: boolean;
  catalogAuthority: "main-integration" | "stable-C-to-H";
  metadataExact: true;
};
type Authority = {
  pinSha: string;
  leaseSha: string;
  operatorRunId: number | null;
  operatorRunAttempt: number | null;
  operatorSourceSha: string | null;
  originalDeadline: number | null;
  author: string;
};
export type IntegrationReceipt = Omit<Authority, "author"> & {
  kind: "vize.release.integration-qualification-v1";
  repository: string;
  context: Context;
  checkId: number;
  checkAttempt: number;
  checkCreatedAt: string;
  qualificationDeadline: number | null;
};
export type IntegrationOptions = {
  repository: string;
  candidateSha: string;
  mode: "wait" | "verify";
  api: ReadApi;
  now?: () => number;
  sleep?: (milliseconds: number) => Promise<void>;
  log?: (message: string) => void;
  observe?: (snapshot: unknown) => void;
  onPendingDeadline?: (deadline: number) => void;
};
const positive = (value: unknown, label: string): number => {
  assert.ok(typeof value === "number" && Number.isSafeInteger(value) && value > 0, label);
  return value;
};
const sha = (value: unknown): string => {
  assert.ok(
    typeof value === "string" && /^[0-9a-f]{40}$/u.test(value),
    "full lowercase SHA required",
  );
  return value;
};
const object = (value: unknown): Row => {
  assert.ok(
    value !== null && typeof value === "object" && !Array.isArray(value),
    "object required",
  );
  return value as Row;
};
function marker(body: unknown, key: string): string {
  assert.equal(typeof body, "string", "source body required");
  const prefix = `<!-- ${key}: `;
  const matches = (body as string).split("\n").filter((line) => line.startsWith(prefix));
  assert.equal(matches.length, 1, `missing/duplicate ${key}`);
  assert.ok(matches[0].endsWith(" -->"), `malformed ${key}`);
  const value = matches[0].slice(prefix.length, -4);
  assert.ok(value.length > 0, `empty ${key}`);
  return value;
}
function sameMessage(actual: unknown, expected: string): void {
  assert.ok(actual === expected || actual === `${expected}\n`, "noncanonical commit message");
}
export function validateIntegrationContext(
  value: unknown,
  repository: string,
  candidateSha: string,
): Context {
  assert.match(repository, /^[\w.-]+\/[\w.-]+$/u);
  const context = object(value);
  assert.deepEqual(
    Object.keys(context).sort(),
    [
      "integration",
      "source",
      "H",
      "C",
      "tag",
      "candidate",
      "base",
      "integrationHead",
      "integrationBody",
      "event",
      "catalogEqual",
      "catalogAuthority",
      "metadataExact",
    ].sort(),
    "exact Rust candidate context required",
  );
  assert.ok(
    typeof context.integrationBody === "string" && context.integrationBody.length > 0,
    "validated I body required",
  );
  positive(context.integration, "integration PR required");
  positive(context.source, "source PR required");
  assert.notEqual(context.integration, context.source);
  for (const field of ["H", "C", "candidate", "base", "integrationHead"]) sha(context[field]);
  assert.notEqual(context.H, context.C);
  assert.equal(context.candidate, sha(candidateSha), "candidate SHA changed");
  assert.match(String(context.tag), /^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/u);
  assert.equal(context.event, "merge_group", "only validated merge groups may join Q");
  assert.equal(context.metadataExact, true);
  assert.ok(
    (context.catalogAuthority === "main-integration" && context.catalogEqual === true) ||
      (context.catalogAuthority === "stable-C-to-H" && context.catalogEqual === false),
    "complete catalog authority required",
  );
  return context as Context;
}
async function authenticate(
  context: Context,
  repository: string,
  api: ReadApi,
): Promise<Authority> {
  const ref = async (branch: string) => {
    const value = await api.read(`git/ref/heads/${branch}`);
    assert.equal(value.ref, `refs/heads/${branch}`);
    assert.equal(object(value.object).type, "commit");
    return sha(object(value.object).sha);
  };
  const commit = async (id: string) => {
    const value = await api.read(`git/commits/${id}`);
    assert.equal(value.sha, id);
    sha(object(value.tree).sha);
    assert.ok(Array.isArray(value.parents), "commit parents required");
    return value;
  };
  const pr = async (number: number, branch: string, head: string, draft: boolean) => {
    const value = await api.read(`pulls/${number}`);
    assert.equal(value.number, number);
    assert.equal(value.state, "open");
    assert.equal(value.merged, false);
    assert.equal(value.draft, draft);
    for (const side of ["head", "base"])
      assert.equal(object(object(value[side]).repo).full_name, repository);
    assert.equal(object(value.head).ref, branch);
    assert.equal(object(value.head).sha, head);
    assert.equal(object(value.base).ref, "main");
    const user = object(value.user);
    assert.equal(user.type, "User");
    assert.equal(typeof user.login, "string", "authenticated PR author required");
    assert.match(user.login as string, /^[A-Za-z0-9][A-Za-z0-9-]*$/u);
    const permission = await api.read(`collaborators/${user.login}/permission`);
    assert.ok(["maintain", "admin"].includes(String(permission.role_name)), "maintainer required");
    assert.equal(await ref(branch), head, "current PR branch moved");
    return value;
  };
  const source = await pr(context.source, `release/${context.tag}`, context.H, true);
  assert.equal(marker(source.body, "vize-release-pin"), "immutable-v1");
  assert.equal(marker(source.body, "vize-release-pin-head"), context.H);
  assert.equal(marker(source.body, "vize-release-pin-cut"), context.C);
  assert.equal(marker(source.body, "vize-release-integration"), String(context.integration));
  const integration = await pr(
    context.integration,
    `release-integration/${context.tag}`,
    context.integrationHead,
    false,
  );
  assert.equal(integration.body, context.integrationBody, "validated integration body changed");
  for (const [key, expected] of Object.entries({
    source: String(context.source),
    head: context.H,
    cut: context.C,
    tag: context.tag,
  }))
    assert.equal(marker(integration.body, `vize-release-pin-${key}`), expected);
  const head = await commit(context.H);
  assert.deepEqual(
    (head.parents as Row[]).map((p) => p.sha),
    [context.C],
    "H single parent changed",
  );
  const candidate = await commit(context.candidate);
  assert.deepEqual(
    (candidate.parents as Row[]).map((p) => p.sha),
    [context.base],
    "candidate parent changed",
  );
  const tree = object(head.tree).sha;
  const pinSha = await ref(`release-pin/${context.tag}`);
  const pin = await commit(pinSha);
  assert.deepEqual(
    (pin.parents as Row[]).map((p) => p.sha),
    [context.H],
  );
  assert.equal(object(pin.tree).sha, tree, "pin tree differs from H");
  const baseVersion = marker(source.body, "vize-release-base-version");
  assert.match(baseVersion, /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/u);
  let pinMessage = `Vize immutable release cut\n\nSource-PR: #${context.source}\nSource-head: ${context.H}\nSource-cut: ${context.C}\nTag: ${context.tag}\nBase-version: ${baseVersion}\nIntegration-PR: #${context.integration}`;
  if (context.catalogAuthority === "stable-C-to-H") {
    const policy = sha(marker(source.body, "vize-release-stable-policy"));
    const plan = sha(marker(source.body, "vize-release-stable-plan"));
    const digest = marker(source.body, "vize-release-stable-sha256");
    assert.match(digest, /^[0-9a-f]{64}$/u);
    pinMessage += `\nStable-policy: ${policy}\nStable-plan: ${plan}\nStable-plan-sha256: ${digest}`;
  }
  sameMessage(pin.message, pinMessage);
  const leaseSha = await ref(`release-operator/${context.tag}`);
  const lease = await commit(leaseSha);
  assert.deepEqual(
    (lease.parents as Row[]).map((p) => p.sha),
    [context.H],
  );
  assert.equal(object(lease.tree).sha, tree, "operator lease tree differs from H");
  const author = String(object(source.user).login);
  const legacy = `Vize release operator\n\nTag: ${context.tag}\nSource-head: ${context.H}\nPID: ([1-9][0-9]*)\nNonce: ([1-9][0-9]*)`;
  if (
    typeof lease.message === "string" &&
    new RegExp(`^${legacy.replaceAll(".", "\\.")}\\n?$`, "u").test(lease.message)
  )
    return {
      pinSha,
      leaseSha,
      author,
      operatorRunId: null,
      operatorRunAttempt: null,
      operatorSourceSha: null,
      originalDeadline: null,
    };
  const prefix = `Vize release operator v2\n\nTag: ${context.tag}\nSource-head: ${context.H}\n`;
  assert.equal(typeof lease.message, "string");
  assert.ok((lease.message as string).startsWith(prefix), "unknown operator lease");
  const match =
    /^Operator-run: ([1-9][0-9]*)\nOperator-attempt: ([1-9][0-9]*)\nOperator-source: ([0-9a-f]{40})\nOriginal-deadline-unix-millis: ([1-9][0-9]*)\nPID: ([1-9][0-9]*)\nNonce: ([1-9][0-9]*)\n?$/u.exec(
      (lease.message as string).slice(prefix.length),
    );
  assert.ok(match, "strict original v2 lease fields required");
  const operatorRunId = positive(Number(match[1]), "operator run ID required");
  const operatorRunAttempt = positive(Number(match[2]), "operator attempt required");
  const originalDeadline = positive(Number(match[4]), "original Budget deadline required");
  assert.equal(match[3], context.C, "operator source is not C");
  assert.match(context.tag, /^v0\.(?:0|[1-9][0-9]*)\.0$/u, "only fresh minor START may wait");
  assert.equal(context.catalogAuthority, "main-integration");
  assert.equal(marker(source.body, "vize-release-bump"), "minor");
  const operator = await api.read(`actions/runs/${operatorRunId}`);
  for (const [key, expected] of Object.entries({
    id: operatorRunId,
    run_attempt: operatorRunAttempt,
    head_sha: context.C,
    head_branch: "main",
    event: "workflow_dispatch",
    path: ".github/workflows/release-operator.yml",
    status: "in_progress",
    conclusion: null,
  }))
    assert.equal(operator[key], expected, `official O ${key} differs`);
  for (const key of ["repository", "head_repository"])
    assert.equal(object(operator[key]).full_name, repository);
  for (const key of ["actor", "triggering_actor"]) {
    assert.equal(object(operator[key]).login, author);
    assert.equal(object(operator[key]).type, "User");
  }
  const workflow = await api.read("actions/workflows/release-operator.yml");
  positive(workflow.id, "official operator workflow ID required");
  assert.equal(typeof workflow.name, "string", "official operator workflow name required");
  assert.equal(workflow.id, operator.workflow_id);
  assert.equal(workflow.path, operator.path);
  const policy = await api.read(`contents/.github/workflows/release-operator.yml?ref=${context.C}`);
  assert.equal(policy.encoding, "base64");
  const text = Buffer.from(String(policy.content), "base64").toString("utf8");
  assert.equal(/^name: (.+)$/mu.exec(text)?.[1], workflow.name, "O source workflow policy differs");
  return {
    pinSha,
    leaseSha,
    author,
    operatorRunId,
    operatorRunAttempt,
    operatorSourceSha: context.C,
    originalDeadline,
  };
}
async function active(
  run: CheckedRun,
  repository: string,
  author: string | null,
  api: ReadApi,
): Promise<void> {
  assert.ok(
    ["queued", "in_progress", "waiting", "requested", "pending", "completed"].includes(run.status),
    "unknown Check status",
  );
  assert.equal(
    run.conclusion,
    run.status === "completed" ? "success" : null,
    "Check failed/cancelled/inconsistent",
  );
  for (const key of ["repository", "head_repository"])
    assert.equal(object(run[key]).full_name, repository);
  const actors = new Set<string>();
  for (const key of ["actor", "triggering_actor"]) {
    const actor = object(run[key]);
    assert.equal(actor.type, "User");
    assert.equal(typeof actor.login, "string");
    assert.match(actor.login as string, /^[A-Za-z0-9][A-Za-z0-9-]*$/u);
    if (author !== null) assert.equal(actor.login, author);
    actors.add(actor.login as string);
  }
  if (author === null)
    for (const login of actors) {
      const permission = await api.read(`collaborators/${login}/permission`);
      assert.ok(
        ["maintain", "admin"].includes(String(permission.role_name)),
        "Q maintainer required",
      );
    }
}
export async function qualifyIntegration(
  input: unknown,
  options: IntegrationOptions,
): Promise<IntegrationReceipt> {
  assert.ok(options.mode === "wait" || options.mode === "verify");
  const previous = options.mode === "verify" ? object(input) : undefined;
  if (previous)
    assert.equal(
      previous.kind,
      "vize.release.integration-qualification-v1",
      "prior catalog Q receipt required",
    );
  const context = validateIntegrationContext(
    previous ? previous.context : input,
    options.repository,
    options.candidateSha,
  );
  const immutable = new Map<string, Row>();
  const api: ReadApi = {
    read: async (resource) => {
      const cached = immutable.get(resource);
      if (cached) return cached;
      const value = await options.api.read(resource);
      if (
        /^git\/commits\/[0-9a-f]{40}$/u.test(resource) ||
        resource === `contents/.github/workflows/release-operator.yml?ref=${context.C}`
      )
        immutable.set(resource, value);
      options.observe?.({ resource, value });
      return value;
    },
    pages: async (resource, collection, query) => {
      const value = await options.api.pages(resource, collection, query);
      options.observe?.({ resource, collection, query, value });
      return value;
    },
  };
  const authority = await authenticate(context, options.repository, api);
  const candidate = {
    repository: options.repository,
    branch: `release/${context.tag}`,
    head: context.H,
    cut: context.C,
  };
  const now = options.now ?? Date.now;
  const initialObserver = createCheckObserver(candidate, {
    api,
    deadline: authority.originalDeadline ?? 1,
    now,
    log: options.log,
  });
  const initial = initialObserver.latest(await initialObserver.listRuns(true));
  assert.ok(initial, "exact existing Q missing; no bootstrap permitted");
  const qAuthor = authority.originalDeadline === null ? null : authority.author;
  await active(initial, options.repository, qAuthor, api);
  assert.equal(typeof initial.created_at, "string");
  assert.match(String(initial.created_at), /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,3})?Z$/u);
  const created = Date.parse(String(initial.created_at));
  assert.ok(
    Number.isSafeInteger(created) && created > 0 && created <= now(),
    "Q creation timestamp invalid",
  );
  const deadline =
    authority.originalDeadline === null
      ? null
      : Math.min(authority.originalDeadline, created + 345 * 60 * 1000);
  const { author: _author, ...identity } = authority;
  const receipt: IntegrationReceipt = {
    kind: "vize.release.integration-qualification-v1",
    repository: options.repository,
    context,
    ...identity,
    checkId: initial.id,
    checkAttempt: initial.run_attempt,
    checkCreatedAt: String(initial.created_at),
    qualificationDeadline: deadline,
  };
  if (previous)
    assert.deepEqual(previous, receipt, "catalog Q/attempt/original operator lease changed");
  const guardedApi: ReadApi = {
    read: (resource) => api.read(resource),
    pages: async (resource, collection, query) => {
      if (resource === "actions/runs")
        assert.deepEqual(
          await authenticate(context, options.repository, api),
          authority,
          "operator/pin/source authority changed",
        );
      const rows = await api.pages(resource, collection, query);
      if (resource === "actions/runs") {
        const current = initialObserver.latest(rows);
        assert.ok(current, "original Q disappeared");
        for (const key of ["id", "run_attempt", "created_at"])
          assert.equal(current[key], initial[key], "latest Q identity/attempt changed");
        await active(current, options.repository, qAuthor, api);
      }
      return rows;
    },
  };
  if (initial.status !== "completed") {
    assert.equal(options.mode, "wait", "final Q verification never waits");
    assert.ok(deadline !== null, "legacy/resume lease cannot wait for pending Q");
    assert.ok(deadline > now(), "original qualification deadline expired");
    options.onPendingDeadline?.(deadline);
    await waitForExistingCheck(candidate, {
      api: guardedApi,
      deadline,
      now,
      sleep: options.sleep,
      log: options.log,
    });
  } else {
    const observer = createCheckObserver(candidate, {
      api: guardedApi,
      deadline: deadline ?? 1,
      now,
      log: options.log,
    });
    await observer.finish(initial, true);
  }
  assert.deepEqual(
    await authenticate(context, options.repository, api),
    authority,
    "final operator/pin/source authority changed",
  );
  return receipt;
}
async function main(): Promise<void> {
  const [mode, ...extra] = process.argv.slice(2);
  assert.ok(
    (mode === "--wait" || mode === "--verify") && extra.length === 0,
    "use --wait or --verify with validated JSON on stdin",
  );
  assert.equal(process.env.GITHUB_EVENT_NAME, "merge_group");
  const repository = process.env.GITHUB_REPOSITORY!;
  const candidateSha = process.env.GITHUB_SHA!;
  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
  assert.ok(token, "read-only GitHub token required");
  const chunks: Buffer[] = [];
  let length = 0;
  for await (const chunk of process.stdin) {
    length += Buffer.byteLength(chunk);
    assert.ok(length <= 16_384, "bounded validated context required");
    chunks.push(Buffer.from(chunk));
  }
  const transport = createIntegrationReadApi({
    repository,
    token,
    apiUrl: process.env.GITHUB_API_URL,
  });
  const evidence = process.env.VIZE_RELEASE_INTEGRATION_EVIDENCE;
  assert.ok(evidence && path.isAbsolute(evidence), "absolute raw Q evidence path required");
  fs.mkdirSync(path.dirname(evidence), { recursive: true });
  const receipt = await qualifyIntegration(JSON.parse(Buffer.concat(chunks).toString("utf8")), {
    repository,
    candidateSha,
    mode: mode === "--wait" ? "wait" : "verify",
    api: transport.api,
    onPendingDeadline: transport.setPendingDeadline,
    log: console.error,
    observe: (snapshot) => fs.appendFileSync(evidence, `${JSON.stringify(snapshot)}\n`),
  });
  process.stdout.write(`${JSON.stringify(receipt)}\n`);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
