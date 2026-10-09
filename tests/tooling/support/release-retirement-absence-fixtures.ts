import assert from "node:assert/strict";
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
  marketplaceControls,
  marketplaceInventory,
} from "./release-retirement-marketplace-fixtures.ts";

type Reply = { status: number; body: unknown; headers?: Record<string, string>; url?: string };
function response(url: string, reply: Reply) {
  const body = typeof reply.body === "string" ? reply.body : JSON.stringify(reply.body);
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
      body: { error: index ? `version not found: ${target.version}` : "Not found" },
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
  const marketplace = `https://marketplace.visualstudio.com/_apis/gallery/publishers/${publisher}/extensions/${name}?flags=1&api-version=7.2-preview.2`;
  const openvsx = `https://open-vsx.org/api/${publisher}/${name}/${version}`;
  replies.set(marketplace, { status: 200, body: marketplaceInventory(publisher, name) });
  replies.set(openvsx, {
    status: 404,
    body: { error: `Extension not found: ${publisher}.${name} ${version}` },
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
  for (const call of calls) {
    assert.equal(call.init?.method, "GET");
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
  for (const url of [npm, crate, marketplace, openvsx]) {
    for (const status of [200, 301, 302, 307, 401, 403, 429, 500, 503]) {
      await reject(
        url,
        { status, body: { error: "Not found" } },
        /requires HTTP 404|missing response/,
      );
    }
    for (const body of [{}, [], null, { error: "upstream unavailable" }, "not JSON"]) {
      await reject(url, { status: 404, body }, /missing|JSON|Unexpected/);
    }
    await reject(
      url,
      { status: 404, body: {}, headers: { "content-type": "text/html" } },
      /JSON content type/,
    );
    await reject(
      url,
      { status: 404, body: {}, url: "https://example.invalid/redirect" },
      /URL changed/,
    );
    await reject(
      url,
      { status: 404, body: {}, headers: { location: "https://example.invalid/" } },
      /redirects refused/,
    );
  }
  await reject(npm, { status: 404, body: { error: "version not found: 9.9.9" } }, /npm missing/);
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
  await reject(npm, { status: 404, body: "x".repeat(65_537) }, /65536 bytes/);
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
  let bodyEntered!: () => void;
  const entered = new Promise<void>((done) => {
    bodyEntered = done;
  });
  let aborted = false;
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const pending = verifyRetirementAbsence(s.plan, {
    fetch: async (input, init) => {
      const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
      if (url !== npm) return fetch(input, init);
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
        { status: 404, headers: { "content-type": "application/json" } },
      );
      Object.defineProperty(result, "url", { value: npm });
      return result;
    },
  });
  await entered;
  t.mock.timers.tick(30_000);
  await assert.rejects(pending, /timed out/);
  assert.equal(aborted, true, "timeout covers the stalled response body");
}
