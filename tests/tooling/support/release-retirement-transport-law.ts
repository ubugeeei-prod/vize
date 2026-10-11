import assert from "node:assert/strict";
import type { TestContext } from "node:test";
import { retirementReader } from "../../../tools/support/release/retirement_transport.ts";
import type { AbsenceTarget } from "../../../tools/support/release/retirement_evidence.ts";
import { verifyRetirementAbsence } from "../../../tools/support/release/retirement_absence.ts";
import { sourceFixture } from "./release-public-acceptance-fixtures.ts";

const target: AbsenceTarget = {
  channel: "npm",
  name: "vize",
  version: "0.444.0",
  method: "GET",
  url: "https://registry.npmjs.org/vize/0.444.0",
};
const network = (code: string) =>
  new TypeError("fetch failed", {
    cause: Object.assign(new Error("inert socket failure"), { code }),
  });

export async function retirementTransportLaw(t: TestContext) {
  const cycle = network("ECONNRESET");
  cycle.cause = cycle;
  const init: RequestInit = {
    credentials: "omit",
    redirect: "manual",
    signal: new AbortController().signal,
  };
  for (const code of [
    "ECONNRESET",
    "ETIMEDOUT",
    "EAI_AGAIN",
    "UND_ERR_CONNECT_TIMEOUT",
    "UND_ERR_SOCKET",
  ]) {
    let reads = 0;
    const accepted = new Response('"version not found: 0.444.0"', { status: 404 });
    const read = retirementReader(target, async (url, options) => {
      assert.equal(url, target.url);
      assert.equal(options, init, "retry retains the same anonymous request and deadline");
      if (++reads < 3) throw network(code);
      return accepted;
    });
    assert.equal(await read(target.url, init), accepted);
    assert.equal(reads, 3);
  }
  for (const error of [
    network("ENOTFOUND"),
    network("CERT_HAS_EXPIRED"),
    new Error("inert network error"),
    new TypeError("unsupported request"),
    new DOMException("stopped", "AbortError"),
    cycle,
    new TypeError("fetch failed", {
      cause: new AggregateError([network("ECONNRESET"), network("CERT_HAS_EXPIRED")]),
    }),
    new TypeError("fetch failed", {
      cause: new AggregateError([network("ECONNRESET"), new Error("unknown cause")]),
    }),
    new TypeError("fetch failed", {
      cause: new AggregateError(Array.from({ length: 9 }, () => network("ECONNRESET"))),
    }),
  ]) {
    let reads = 0;
    await assert.rejects(
      retirementReader(target, async () => {
        reads++;
        throw error;
      })(target.url, init),
      (failure: unknown) => {
        assert.ok(failure instanceof Error);
        assert.ok(failure.message.includes(target.url));
        assert.ok(failure.message.includes("npm vize"));
        assert.ok(failure.message.includes("attempts=1"));
        assert.equal(failure.cause, error);
        return true;
      },
    );
    assert.equal(reads, 1);
  }
  for (const status of [200, 301, 401, 403, 404, 429, 500, 503]) {
    let reads = 0;
    const response = new Response("unvalidated provider response", { status });
    assert.equal(
      await retirementReader(target, async () => {
        reads++;
        return response;
      })(target.url, init),
      response,
    );
    assert.equal(reads, 1, "HTTP responses go unchanged to existing absence/schema refusal");
  }
  let reads = 0;
  await assert.rejects(
    retirementReader(target, async () => {
      reads++;
      throw network("ECONNRESET");
    })(target.url, init),
    /attempts=3.*ECONNRESET/,
  );
  assert.equal(reads, 3);
  const controller = new AbortController();
  const pending = retirementReader(target, async () => {
    controller.abort();
    throw network("ECONNRESET");
  })(target.url, { signal: controller.signal });
  await assert.rejects(pending, /attempts=1/);
  const backoff = new AbortController();
  const paused = retirementReader(target, async () => {
    throw network("ECONNRESET");
  })(target.url, { signal: backoff.signal });
  setTimeout(() => backoff.abort(), 10);
  await assert.rejects(paused, (error: unknown) => {
    assert.ok(error instanceof Error);
    assert.ok(error.message.includes(target.url));
    assert.ok(error.message.includes("ECONNRESET"));
    assert.ok(error.cause instanceof Error && error.cause.name === "AbortError");
    return true;
  });
  const line = new Error("bad\n::error::inert");
  await assert.rejects(
    retirementReader(target, async () => {
      throw line;
    })(target.url),
    (error: unknown) => {
      assert.ok(error instanceof Error);
      assert.ok(!error.message.includes("\n::error::"));
      assert.ok(error.message.includes("\\n::error::"));
      return true;
    },
  );
  const source = sourceFixture(t);
  const first = source.plan.npm[0];
  const endpoint = `https://registry.npmjs.org/${encodeURIComponent(first.name)}/${first.version}`;
  let endpointReads = 0;
  const fetch: typeof globalThis.fetch = async (input) => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
    if (url === endpoint && ++endpointReads < 3) throw network("ECONNRESET");
    const result = new Response(JSON.stringify({ error: "unavailable" }), { status: 503 });
    Object.defineProperty(result, "url", { value: url });
    return result;
  };
  // A transient fetch may recover, but the recovered HTTP refusal remains fatal.
  await assert.rejects(
    verifyRetirementAbsence({ ...source.plan, npm: [first] }, { fetch }),
    /requires HTTP 404/,
  );
  await new Promise((resolve) => setTimeout(resolve, 800));
  assert.equal(endpointReads, 3, "the real retirement observer uses the bounded transport policy");
}
