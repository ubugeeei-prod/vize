import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import type { TestContext } from "node:test";
import {
  retirementAbsenceMain,
  verifyRetirementAbsence,
  verifyRetirementSourceAbsence,
} from "../../../tools/support/release/retirement_absence.ts";
import { sourceFixture, tag } from "./release-public-acceptance-fixtures.ts";
import {
  retirementPrimaryResponseLaw,
  type Reply,
} from "./release-retirement-primary-response-law.ts";
import {
  marketplaceControls,
  marketplaceInventory,
  marketplaceEnvelope,
} from "./release-retirement-marketplace-fixtures.ts";
import {
  marketplaceQuery,
  marketplaceQueryUrl,
  marketplaceQueryAccept,
} from "../../../tools/support/release/marketplace_query.ts";

function response(url: string, reply: Reply) {
  const body = reply.rawBody ?? JSON.stringify(reply.body);
  const result = new Response(body, {
    status: reply.status,
    headers: { "content-type": "application/json", ...reply.headers },
  });
  Object.defineProperty(result, "url", { value: reply.url ?? url });
  return result;
}

/** Inert public HTTP shapes and temporary raw Git fixtures; no public retirement claim. */
export async function retirementAbsenceLaw(t: TestContext) {
  const s = sourceFixture(t);
  const replies = new Map<string, Reply>();
  for (const [index, target] of s.plan.npm.entries()) {
    replies.set(`https://registry.npmjs.org/${encodeURIComponent(target.name)}/${target.version}`, {
      status: 404,
      body:
        index === 0
          ? `version not found: ${target.version}`
          : { error: index === 1 ? "Not found" : `version not found: ${target.version}` },
    });
  }
  for (const [index, target] of s.plan.crates.entries()) {
    replies.set(`https://crates.io/api/v1/crates/${target.name}/${target.version}`, {
      status: 404,
      body: {
        errors: [
          {
            detail: index
              ? `crate \`${target.name}\` does not exist`
              : `crate \`${target.name}\` does not have a version \`${target.version}\``,
          },
        ],
      },
    });
  }
  const { publisher, name, version } = s.plan.editor;
  const marketplace = marketplaceQueryUrl;
  const openvsx = `https://open-vsx.org/api/${publisher}/${name}/${version}`;
  replies.set(marketplace, {
    status: 200,
    body: marketplaceEnvelope(marketplaceInventory(publisher, name)),
  });
  replies.set(openvsx, {
    status: 404,
    body: {
      deprecated: false,
      downloadable: false,
      error: `Extension not found: ${publisher}.${name} ${version}`,
    },
  });
  const calls: { url: string; init: RequestInit | undefined }[] = [];
  const fetch: typeof globalThis.fetch = async (input, init) => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
    calls.push({ url, init });
    assert.ok(replies.has(url), `unplanned endpoint: ${url}`);
    return response(url, replies.get(url)!);
  };
  s.write("npm/native/package.json", '{"version":"9.9.9"}');
  const receipt = await verifyRetirementSourceAbsence(s.root, s.head, tag, { fetch });
  assert.deepEqual(receipt.plan, s.plan);
  assert.equal(receipt.observations.length, s.plan.npm.length + s.plan.crates.length + 2);
  assert.deepEqual(new Set(receipt.observations.map((row) => row.url)), new Set(replies.keys()));
  assert.ok(
    receipt.observations.every(
      (row) =>
        row.status === replies.get(row.url)?.status && /^[a-f0-9]{64}$/.test(row.responseSha256),
    ),
  );
  for (const row of receipt.observations) {
    const reply = replies.get(row.url)!;
    const bytes = Buffer.from(JSON.stringify(reply.body));
    assert.deepEqual(row.response, reply.body, "retain the provider JSON type without wrapping");
    assert.equal(row.responseBytes, bytes.length);
    assert.equal(row.responseSha256, createHash("sha256").update(bytes).digest("hex"));
  }
  assert.equal(typeof receipt.observations[0].response, "string");
  await retirementPrimaryResponseLaw(
    s.plan,
    replies,
    [receipt.observations[0].url, openvsx, marketplace],
    fetch,
  );
  for (const call of calls) {
    assert.equal(call.init?.method, call.url === marketplace ? "POST" : "GET");
    if (call.url === marketplace) {
      assert.equal(typeof call.init?.body, "string");
      assert.ok(typeof call.init?.body === "string");
      assert.deepEqual(JSON.parse(call.init.body), marketplaceQuery(`${publisher}.${name}`));
      assert.equal(new Headers(call.init?.headers).get("accept"), marketplaceQueryAccept);
      assert.equal(new Headers(call.init?.headers).get("content-type"), "application/json");
    } else assert.equal(call.init?.body, undefined);
    assert.equal(call.init?.redirect, "manual");
    assert.equal(call.init?.credentials, "omit");
    assert.equal(call.init?.cache, "no-store");
    assert.equal(new Headers(call.init?.headers).get("authorization"), null);
  }
  const reject = async (url: string, changed: Reply, pattern: RegExp) => {
    const previous = replies.get(url)!;
    replies.set(url, changed);
    try {
      await assert.rejects(verifyRetirementAbsence(s.plan, { fetch }), pattern);
    } finally {
      replies.set(url, previous);
    }
  };
  const npm = [...replies.keys()][0],
    crate = [...replies.keys()][s.plan.npm.length];
  // Original parse/shape failures escaped contextual diagnostics entirely.
  for (const rawBody of ["not JSON\n::error::untrusted", "null", new Uint8Array([0xff, 0xfe])]) {
    const previous = replies.get(npm)!;
    replies.set(npm, { status: 404, body: null, rawBody });
    try {
      await assert.rejects(verifyRetirementAbsence(s.plan, { fetch }), (error: unknown) => {
        assert.ok(error instanceof Error);
        const bytes = typeof rawBody === "string" ? Buffer.from(rawBody) : Buffer.from(rawBody);
        assert.ok(error.message.includes(`HTTP 404 at ${npm}`));
        assert.ok(
          error.message.includes(
            `response sha256=${createHash("sha256").update(bytes).digest("hex")}`,
          ),
        );
        assert.ok(error.message.includes(`bytes=${bytes.length}`));
        assert.ok(error.message.includes(`body=${JSON.stringify(bytes.toString("utf8"))}`));
        assert.ok(!error.message.includes("\n::error::"), "untrusted body cannot inject log lines");
        return true;
      });
    } finally {
      replies.set(npm, previous);
    }
  }
  for (const url of [npm, crate, marketplace, openvsx]) {
    for (const status of [200, 301, 302, 307, 401, 403, 429, 500, 503]) {
      await reject(
        url,
        { status, body: { error: "Not found" } },
        /requires HTTP 404|missing response|Marketplace/,
      );
    }
    for (const body of [{}, [], null, { error: "upstream unavailable" }, "not JSON"]) {
      await reject(url, { status: 404, body }, /missing|JSON|Unexpected|HTTP 200/);
    }
    await reject(
      url,
      { status: 404, body: {}, headers: { "content-type": "text/html" } },
      /JSON content type|HTTP 200/,
    );
    await reject(
      url,
      { status: 404, body: {}, url: "https://example.invalid/redirect" },
      /URL changed|HTTP 200/,
    );
    await reject(
      url,
      { status: 404, body: {}, headers: { location: "https://example.invalid/" } },
      /redirects refused|HTTP 200/,
    );
  }
  await reject(npm, { status: 404, body: { error: "version not found: 9.9.9" } }, /npm missing/);
  for (const body of ["Not found", "version not found: 9.9.9", `version not found: ${version}\n`]) {
    await reject(npm, { status: 404, body }, /npm missing/);
  }
  for (const url of [crate, marketplace, openvsx]) {
    await reject(
      url,
      { status: 404, body: `version not found: ${version}` },
      /missing response|HTTP 200/,
    );
  }
  await reject(
    crate,
    { status: 404, body: { errors: [{ detail: "crate `other` does not exist" }] } },
    /crate missing/,
  );
  await reject(
    openvsx,
    { status: 404, body: { error: `Extension not found: ${publisher}.${name} 9.9.9` } },
    /Open VSX/,
  );
  const openvsxBody = replies.get(openvsx)!.body as Record<string, unknown>;
  for (const body of [
    { ...openvsxBody, deprecated: true },
    { ...openvsxBody, downloadable: true },
    { ...openvsxBody, deprecated: null },
    { ...openvsxBody, downloadable: "false" },
    { ...openvsxBody, extra: false },
    { error: openvsxBody.error, deprecated: false },
  ]) {
    await reject(openvsx, { status: 404, body }, /Open VSX/);
  }
  const originalOpenvsx = replies.get(openvsx)!;
  replies.set(openvsx, { status: 404, body: { error: openvsxBody.error } });
  try {
    await verifyRetirementAbsence(s.plan, { fetch });
  } finally {
    replies.set(openvsx, originalOpenvsx);
  }
  await reject(
    marketplace,
    {
      status: 404,
      body: {
        typeKey: "ExtensionVersionNotFoundException",
        message: "other.extension 9.9.9 was not found.",
      },
    },
    /Marketplace/,
  );
  await marketplaceControls(s.plan.editor, marketplace, reject, async (body) => {
    const previous = replies.get(marketplace)!;
    replies.set(marketplace, { status: 200, body });
    try {
      return (await verifyRetirementAbsence(s.plan, { fetch })).observations.find(
        (row) => row.channel === "marketplace",
      )!;
    } finally {
      replies.set(marketplace, previous);
    }
  });
  await reject(npm, { status: 404, body: null, rawBody: "x".repeat(65_537) }, /65536 bytes/);
  await reject(
    npm,
    { status: 404, body: {}, headers: { "content-length": "65537" } },
    /65536 bytes/,
  );
  let attempted = 0;
  await assert.rejects(
    verifyRetirementAbsence(s.plan, {
      fetch: async () => {
        attempted++;
        throw new Error("inert network error");
      },
    }),
    /inert network/,
  );
  assert.equal(attempted, 3, "one finite initial batch; no retries");
  const before = calls.length;
  await assert.rejects(
    verifyRetirementSourceAbsence(s.root, s.head, "v0.999.0", { fetch }),
    /tag differs/,
  );
  await assert.rejects(
    verifyRetirementAbsence({ ...s.plan, npm: [{ name: "@evil/escape", version }] }, { fetch }),
    /unsafe/,
  );
  assert.equal(calls.length, before);
  const output = path.join(s.root, "absence.json");
  const args = ["--root", s.root, "--head", s.head, "--tag", tag, "--output", output];
  const original = replies.get(openvsx)!;
  replies.set(openvsx, { status: 503, body: { error: "unavailable" } });
  await assert.rejects(retirementAbsenceMain(args, { fetch }), /requires HTTP 404/);
  assert.equal(fs.existsSync(output), false, "failed checks never emit a receipt");
  replies.set(openvsx, original);
  await retirementAbsenceMain(args, { fetch });
  assert.deepEqual(JSON.parse(fs.readFileSync(output, "utf8")).plan, s.plan);
  await assert.rejects(retirementAbsenceMain(args, { fetch }), /EEXIST/);
  await assert.rejects(
    retirementAbsenceMain([...args.slice(0, 6), "--fetch", output], { fetch }),
    /invalid/,
  );
  t.mock.timers.enable({ apis: ["setTimeout"] });
  for (const stalled of [npm, marketplace]) {
    let bodyEntered!: () => void;
    const entered = new Promise<void>((done) => {
      bodyEntered = done;
    });
    let aborted = false;
    const pending = verifyRetirementAbsence(s.plan, {
      fetch: async (input, init) => {
        const url =
          typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
        if (url !== stalled) return fetch(input, init);
        const result = new Response(
          new ReadableStream<Uint8Array>({
            pull(controller) {
              bodyEntered();
              init?.signal?.addEventListener(
                "abort",
                () => {
                  aborted = true;
                  controller.error(new Error("aborted"));
                },
                { once: true },
              );
              return new Promise(() => {});
            },
          }),
          {
            status: stalled === marketplace ? 200 : 404,
            headers: { "content-type": "application/json" },
          },
        );
        Object.defineProperty(result, "url", { value: stalled });
        return result;
      },
    });
    await entered;
    t.mock.timers.tick(30_000);
    await assert.rejects(pending, /timed out/);
    assert.equal(aborted, true, "timeout covers each stalled provider response body");
  }
}
