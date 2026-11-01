import assert from "node:assert/strict";
import { test } from "node:test";
import { observeLoader } from "../../../tools/support/release/human_reports_11252_11253_public/loader-law.ts";
import { nativeActors } from "../../../tools/support/release/human_reports_11252_11253_public/native-law.ts";
import { authenticateFixCustody } from "../../../tools/support/release/human_reports_11252_11253_public/fix-custody.ts";
import { publicArchive } from "../../../tools/support/release/human_reports_11252_11253_public/network.ts";
import { mainManualContext } from "../../../tools/support/release/human_reports_11252_11253_public/context.ts";
import { gitBlob } from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";
import {
  observedFetch,
  requestUrl,
} from "../../../tools/support/release/human_reports_11252_11253_public/evidence.ts";
import { compareOutcome } from "../../../tools/support/release/human_reports_11252_11253_public/processes.ts";
import type {
  Oracle,
  Observation,
} from "../../../tools/support/release/human_reports_11252_11253_public/model.ts";
import type { NativeEvent } from "../../../tools/support/release/human_reports_11252_11253_public/model.ts";
test("pure loader conserves all arguments, this, returned identity and thrown identity", () => {
  const receiver = { synthetic: true },
    args = [{}, "synthetic.node", { third: true }, 42],
    returned = { same: true },
    events: string[] = [];
  let seen: unknown[] = [];
  const original = function (this: unknown, ...input: unknown[]) {
    seen = input;
    assert.equal(this, receiver);
    return returned;
  };
  const observed = observeLoader(
    original,
    () => true,
    (event) => events.push(event),
    () => {},
  );
  assert.equal(Reflect.apply(observed, receiver, args), returned);
  assert.deepEqual(seen, args);
  assert.deepEqual(events, ["attempt", "returned"]);
  const error = { sameError: true };
  const failed = observeLoader(
    function () {
      throw error;
    },
    () => true,
    (event) => events.push(event),
    () => {},
  );
  assert.throws(
    () => Reflect.apply(failed, receiver, args),
    (e) => e === error,
  );
  assert.deepEqual(events.slice(-2), ["attempt", "failed"]);
  const untouched = observeLoader(
    original,
    () => false,
    () => {
      throw Error("must not emit");
    },
    () => {
      throw Error("must not verify");
    },
  );
  assert.equal(Reflect.apply(untouched, receiver, args), returned);
  assert.deepEqual(seen, args);
});
const frame = (event: NativeEvent["event"]): NativeEvent => ({
  event,
  commandId: "synthetic",
  pid: 11,
  parentPid: 10,
  argv: ["synthetic"],
  nativePath: "/synthetic.node",
  sha256: "a".repeat(64),
});
test("finite per-PID native events permit init-only noncredit and every complete original load pair", () => {
  assert.deepEqual(nativeActors([frame("initialized"), frame("exit")], "synthetic"), []);
  assert.equal(
    nativeActors(
      [
        frame("initialized"),
        frame("attempt"),
        frame("returned"),
        frame("attempt"),
        frame("returned"),
        frame("exit"),
      ],
      "synthetic",
    ).length,
    2,
  );
  for (const events of [
    [frame("returned")],
    [frame("initialized"), frame("returned")],
    [frame("initialized"), frame("attempt")],
    [frame("initialized"), frame("initialized")],
    [frame("initialized"), frame("attempt"), frame("returned"), frame("returned")],
    [frame("initialized"), frame("attempt"), frame("failed")],
    [frame("initialized"), frame("exit"), frame("attempt")],
  ])
    assert.throws(() => nativeActors(events, "synthetic"));
  assert.throws(() =>
    nativeActors(
      [{ ...frame("initialized"), event: "unknown" } as unknown as NativeEvent],
      "synthetic",
    ),
  );
  assert.throws(() => nativeActors([frame("initialized")], "other-command"));
});
test("later production revert and forged Git body refuse even with the same merge ancestor", async () => {
  const path = "crates/synthetic.rs",
    fix = {
      issue: 11253 as const,
      pull: 11258,
      originalHead: "a".repeat(40),
      merge: "b".repeat(40),
      paths: [path],
    },
    source = "c".repeat(40),
    candidate = "d".repeat(40);
  const read = (mutant: string | null) => async (endpoint: string) => {
    const ref = endpoint.split("?ref=")[1];
    const raw = Buffer.from(ref === mutant ? "reverted" : "reviewed");
    return {
      path,
      type: "file",
      encoding: "base64",
      content: raw.toString("base64"),
      sha: gitBlob(raw),
    };
  };
  await authenticateFixCustody(fix, source, candidate, read(null));
  await assert.rejects(authenticateFixCustody(fix, source, candidate, read(source)));
  await assert.rejects(authenticateFixCustody(fix, source, candidate, read(candidate)));
  await assert.rejects(
    authenticateFixCustody(fix, source, candidate, async () => ({
      path,
      type: "file",
      encoding: "base64",
      content: Buffer.from("wrong").toString("base64"),
      sha: gitBlob(Buffer.from("reviewed")),
    })),
  );
});
function response(body: BodyInit, url: string): Response {
  const r = new Response(body);
  Object.defineProperty(r, "url", { value: url });
  return r;
}
test("bounded registry metadata rejects overflow and strict UTF8 before package/SRI claims", async () => {
  const receipts: unknown[] = [];
  const sink = (kind: string, value: unknown) => receipts.push({ kind, value });
  const over: typeof fetch = async (input) =>
    response(new Uint8Array(1024 * 1024 + 1), requestUrl(input));
  await assert.rejects(publicArchive("synthetic-overflow", "0", sink, over), /byte limit/);
  assert(receipts.some((v) => (v as { kind: string }).kind === "registry-body-refusal"));
  const invalid: typeof fetch = async (input) =>
    response(new Uint8Array([0xff]), requestUrl(input));
  await assert.rejects(publicArchive("synthetic-utf8", "0", sink, invalid));
  assert(receipts.some((v) => (v as { kind: string }).kind === "registry-body"));
  const badSri: typeof fetch = async (input) =>
    response(
      JSON.stringify({
        name: "synthetic-sri",
        version: "0",
        dist: { tarball: "https://registry.npmjs.org/synthetic.tgz", integrity: "invalid" },
      }),
      requestUrl(input),
    );
  await assert.rejects(publicArchive("synthetic-sri", "0", sink, badSri));
  assert(receipts.some((v) => (v as { kind: string }).kind === "registry-metadata"));
});
test("public execution context refuses PR, non-main, local and other repositories", () => {
  const env = {
    GITHUB_ACTIONS: "true",
    GITHUB_EVENT_NAME: "workflow_dispatch",
    GITHUB_REF: "refs/heads/main",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
  };
  mainManualContext(env);
  for (const patch of [
    { GITHUB_EVENT_NAME: "pull_request" },
    { GITHUB_REF: "refs/heads/feature" },
    { GITHUB_ACTIONS: "false" },
    { GITHUB_REPOSITORY: "other/repo" },
  ])
    assert.throws(() => mainManualContext({ ...env, ...patch }));
});
test(
  "observed bounded original rejects overflow without a tee and preserves small response metadata",
  { timeout: 1000 },
  async () => {
    const events: string[] = [];
    const sink = (kind: string) => events.push(kind);
    const over: typeof fetch = async () => new Response(new Uint8Array(2 * 1024 * 1024 + 1));
    await assert.rejects(
      observedFetch(sink, over)("https://example.invalid/oversize"),
      /byte limit/,
    );
    assert(events.includes("http-body-refusal"));
    const small: typeof fetch = async () => {
      const r = new Response("exact", {
        status: 201,
        statusText: "Created",
        headers: { "x-source": "same" },
      });
      Object.defineProperty(r, "url", { value: "https://example.invalid/original" });
      return r;
    };
    const r = await observedFetch(sink, small)("https://example.invalid/original");
    assert.equal(await r.text(), "exact");
    assert.equal(r.status, 201);
    assert.equal(r.statusText, "Created");
    assert.equal(r.headers.get("x-source"), "same");
    assert.equal(r.url, "https://example.invalid/original");
    const absent: typeof fetch = async () => new Response(null, { status: 204 });
    assert.equal((await observedFetch(sink, absent)("https://example.invalid/empty")).body, null);
  },
);
test("an unknown stdout oracle kind cannot enter the valid timing branch", () => {
  const literals = [
      "Type checked 1 files in ",
      " (collect: ",
      ", imports: ",
      ", gen: ",
      ", corsa: ",
      ")\n",
    ],
    fields = ["total", "collect", "imports", "gen", "corsa"].map((label) => ({
      label,
      maxNanoseconds: 1e9,
    }));
  const text =
    literals[0] +
    "1ms" +
    literals[1] +
    "1ms" +
    literals[2] +
    "1ms" +
    literals[3] +
    "1ms" +
    literals[4] +
    "1ms" +
    literals[5];
  const oracle = {
    id: "synthetic",
    status: 0,
    stderr: { kind: "bytes", base64: "" },
    stdout: { kind: "unknown", literals, fields },
  } as unknown as Omit<Oracle, "sourceOwnedBy">;
  const row = {
    id: "synthetic",
    status: 0,
    stdoutBase64: Buffer.from(text).toString("base64"),
    stderrBase64: "",
  } as Observation;
  assert.throws(() => compareOutcome(oracle, row), /unknown stdout oracle kind/);
});
