import assert from "node:assert/strict";
import { test } from "node:test";
import { crates } from "../../../tools/support/release/corsa_bootstrap/policy.ts";
import { readHandoff, type Commands } from "../../../tools/support/release/corsa_bootstrap/main.ts";
import {
  archive,
  boundedRequest,
  canonicalOwner,
  configs,
  ensurePublisher,
  exactVersion,
  intendedOwner,
  sha256,
  type Request,
} from "../../../tools/support/release/corsa_bootstrap/registry.ts";
const publisher = (name: string) => ({
  id: 1,
  crate: name,
  repository_owner: "ubugeeei-prod",
  repository_owner_id: 42,
  repository_name: "vize",
  workflow_filename: "release.yml",
  environment: "crates-io",
  created_at: "2026-10-18T00:00:00Z",
});
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
const list = (rows: unknown[], next_page: string | null = null, total = rows.length) =>
  json({ github_configs: rows, meta: { total, next_page } });
const assertString = (value: unknown) => {
  assert.ok(typeof value === "string");
  return value;
};

test("caller ID must be the live canonical direct user owner, never a foreign owner or team", async () => {
  const owner = { id: 7, login: "maintainer", kind: "user", url: "https://github.com/maintainer" };
  assert.deepEqual(
    await canonicalOwner(7, "maintainer", async () => json({ users: [owner] })),
    owner,
  );
  for (const change of [
    { id: 8 },
    { login: "foreign" },
    { kind: "team" },
    { url: "https://github.com/foreign" },
  ])
    await assert.rejects(
      canonicalOwner(7, "maintainer", async () => json({ users: [{ ...owner, ...change }] })),
    );
  await assert.rejects(canonicalOwner(7, "maintainer", async () => json({}, 503)));
});
test("current crates.io user inventory authenticates intended ID without guessed /me", async () => {
  const requests: string[] = [];
  await intendedOwner(7, "protected-token", async (url, init) => {
    requests.push(url);
    assert.equal(new Headers(init?.headers).get("authorization"), "protected-token");
    return list([]);
  });
  assert.equal(
    requests[0],
    "https://crates.io/api/v1/trusted_publishing/github_configs?user_id=7&per_page=100",
  );
  await assert.rejects(intendedOwner(7, "protected-token", async () => json({}, 403)));
});
test("seek pagination preserves selector, origin, complete count and unique server IDs", async () => {
  let count = 0;
  const rows = await configs({ crate: crates[0] }, "token", async () =>
    ++count === 1
      ? list([publisher(crates[0])], `?crate=${crates[0]}&seek=abc`, 2)
      : list([{ ...publisher(crates[0]), id: 2 }], null, 2),
  );
  assert.equal(rows.length, 2);
  for (const next of [
    "https://evil.example/api/v1/trusted_publishing/github_configs?crate=vize_corsa_core&seek=a",
    "?crate=vize_corsa&seek=a",
    "?crate=vize_corsa_core&user_id=7&seek=a",
    "?crate=vize_corsa_core&page=2",
    "/elsewhere?crate=vize_corsa_core&seek=a",
  ])
    await assert.rejects(configs({ crate: crates[0] }, "token", async () => list([], next, 1)));
  await assert.rejects(configs({ crate: crates[0] }, "token", async () => list([], null, 1)));
  await assert.rejects(
    configs({ crate: crates[0] }, "token", async () =>
      list([publisher(crates[0]), publisher(crates[0])]),
    ),
  );
});
test("Trusted Publisher creation uses current POST nested schema and rereads the server", async () => {
  const calls: string[] = [];
  const row = publisher(crates[0]);
  const request: Request = async (_url, init) => {
    calls.push(init?.method ?? "GET");
    if (init?.method === "POST") {
      assert.deepEqual(JSON.parse(assertString(init.body)), {
        github_config: {
          crate: crates[0],
          repository_owner: "ubugeeei-prod",
          repository_name: "vize",
          workflow_filename: "release.yml",
          environment: "crates-io",
        },
      });
      return json({ github_config: row });
    }
    return list(calls.length === 1 ? [] : [row]);
  };
  assert.deepEqual(
    await ensurePublisher(crates[0], 42, "token", request, () => {
      calls.push("QUALIFY");
    }),
    row,
  );
  assert.deepEqual(calls, ["GET", "QUALIFY", "POST", "GET"]);
  await assert.rejects(
    ensurePublisher(
      crates[0],
      42,
      "token",
      async () => list([{ ...row, environment: "other" }]),
      () => {
        throw new Error("unexpected mutation");
      },
    ),
  );
});
await test("publisher POST cannot follow a failed live qualification after its inventory await", async () => {
  let writes = 0;
  await assert.rejects(
    ensurePublisher(
      crates[0],
      42,
      "token",
      async (_url, init) => {
        if (init?.method === "POST") writes++;
        return list([]);
      },
      () => {
        throw new Error("full Check advanced after inventory");
      },
    ),
    /full Check advanced/u,
  );
  assert.equal(writes, 0);
});
test("registry exact-version metadata and whole downloaded bytes reject substitution", async () => {
  const bytes = Buffer.from("whole controlled archive bytes"),
    checksum = sha256(bytes);
  assert.equal(
    await exactVersion(crates[0], "0.999.0", async () =>
      json({ version: { crate: crates[0], num: "0.999.0", yanked: false, checksum } }),
    ),
    checksum,
  );
  await assert.rejects(
    exactVersion(crates[0], "0.999.0", async () =>
      json({ version: { crate: crates[1], num: "0.999.0", yanked: false, checksum } }),
    ),
  );
  const download: Request = async (url) =>
    url.endsWith("/download")
      ? new Response(null, {
          status: 302,
          headers: {
            location: `https://static.crates.io/crates/${crates[0]}/${crates[0]}-0.999.0.crate`,
          },
        })
      : new Response(bytes);
  assert.deepEqual(await archive(crates[0], "0.999.0", checksum, download), bytes);
  await assert.rejects(archive(crates[0], "0.999.0", "0".repeat(64), download));
  await assert.rejects(
    archive(
      crates[0],
      "0.999.0",
      checksum,
      async () =>
        new Response(null, { status: 302, headers: { location: "https://evil.example/token" } }),
    ),
  );
});
test("an expired inherited budget cannot perform even the first network request", async () => {
  let calls = 0;
  await assert.rejects(
    boundedRequest(Date.now() - 1, async () => {
      calls++;
      return json({});
    })("https://crates.io"),
  );
  assert.equal(calls, 0);
});

test("handoff cannot consume fabricated JSON before authenticating whole artifact ZIP bytes", () => {
  const source = {
    repository: "ubugeeei-prod/vize",
    number: 1,
    head: "a".repeat(40),
    cut: "b".repeat(40),
    branch: "release/v0.999.0",
    tag: "v0.999.0",
  };
  const run = {
    id: 99,
    run_attempt: 1,
    head_sha: source.head,
    head_branch: source.branch,
    path: ".github/workflows/release-corsa-bootstrap.yml",
    event: "workflow_dispatch",
    status: "completed",
    conclusion: "success",
    repository: { full_name: source.repository },
  };
  let reads = 0;
  const io: Commands = {
    git: () => "",
    read: () => "",
    exists: () => false,
    run: () => {
      reads++;
      return Buffer.from("tampered ZIP bytes");
    },
    api: (resource) =>
      resource.includes("/workflows/")
        ? { total_count: 1, workflow_runs: [run] }
        : {
            total_count: 1,
            artifacts: [
              {
                id: 9,
                name: "corsa-first-publication",
                expired: false,
                digest: `sha256:${"0".repeat(64)}`,
                workflow_run: { id: 99, head_sha: source.head },
              },
            ],
          },
  };
  assert.throws(() => readHandoff(source, io), /whole artifact ZIP digest mismatch/u);
  assert.equal(reads, 1, "no unzip or JSON consumption after wrong archive digest");
  run.head_sha = source.cut;
  reads = 0;
  assert.throws(() => readHandoff(source, io));
  assert.equal(reads, 0, "foreign-H run cannot supply an artifact");
});
