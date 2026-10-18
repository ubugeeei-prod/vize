import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { once } from "node:events";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { SessionArtifacts } from "../../../npm/builder/vite-musea/src/cli/service/artifacts.ts";
import {
  assertApiReport,
  assertSessionReport,
  assertSummary,
  type ApiCapture,
  type VrtReport,
  type VrtSummary,
  type SessionCapture,
} from "../../../tools/support/release/public_acceptance/vrt_artifacts.ts";

// Inert envelope laws: no product, compiler, renderer, HTTP API or native provider is substituted.
function envelopes(status: "error" | "new" | "passed" | "failed" = "passed") {
  const shared = {
    artPath: "/guard-only/src/left/Button.art.vue",
    variantName: "Default",
    viewport: "authored-compact",
    snapshotPath: "/guard-only/baselines/left.png",
    ...(status === "error"
      ? { error: "guard-only capture error" }
      : { currentPath: "/guard-only/current/left.png" }),
    ...(status === "failed" ? { diffPath: "/guard-only/diff/left.png" } : {}),
    ...(["passed", "failed"].includes(status)
      ? { diffPercentage: status === "passed" ? 0 : 1 }
      : {}),
  };
  const summary: VrtSummary = {
    total: 1,
    passed: Number(status === "passed"),
    failed: Number(status === "failed"),
    new: Number(status === "new"),
    errors: Number(status === "error"),
    skipped: 0,
    duration: 17,
  };
  const packet: ApiCapture = {
    success: true,
    summary: { ...summary },
    results: [
      {
        ...shared,
        passed: status === "passed" || status === "new",
        ...(status === "new" ? { isNew: true } : {}),
      },
    ],
    artifacts: {
      snapshotDir: "/guard-only/baselines",
      jsonReportPath: "/guard-only/report.json",
      htmlReportPath: "/guard-only/report.html",
    },
  };
  const report: VrtReport = {
    summary: { ...summary },
    results: [
      {
        ...shared,
        art: "Button",
        variant: "Default",
        status,
        ...(["passed", "failed"].includes(status)
          ? { diffPixels: status === "passed" ? 0 : 576, totalPixels: 57600 }
          : {}),
      },
    ],
  };
  return { packet, report };
}
const rejects = (report: VrtReport, packet: ApiCapture) =>
  assert.throws(() => assertApiReport(report, packet, 320, 180), { name: "AssertionError" });

test("whole physical JSON summaries and each shared field bind exactly to the parsed packet", async (t) => {
  const directory = await mkdtemp(path.join(os.tmpdir(), "vize-vrt-report-law-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  for (const status of ["error", "new", "passed", "failed"] as const) {
    const { packet, report } = envelopes(status);
    const file = path.join(directory, `${status}.json`);
    await writeFile(file, JSON.stringify(report));
    const written = JSON.parse(await readFile(file, "utf8"));
    assert.doesNotThrow(() =>
      assertApiReport(written, JSON.parse(JSON.stringify(packet)), 320, 180),
    );
    assert.equal(written.results[0].status, status);
  }
  for (const key of [
    "total",
    "new",
    "passed",
    "failed",
    "errors",
    "skipped",
    "duration",
  ] as const) {
    const { packet, report } = envelopes();
    report.summary[key]++;
    rejects(report, packet);
  }
  for (const [key, changed] of Object.entries({
    artPath: "/guard-only/src/right/Button.art.vue",
    variantName: "Wrong",
    viewport: "different",
    snapshotPath: "/foreign.png",
    currentPath: "/foreign-current.png",
    diffPath: "/unexpected-diff.png",
    diffPercentage: 2,
    error: "unexpected capture error",
  })) {
    const { packet, report } = envelopes();
    Reflect.set(report.results[0], key, changed);
    rejects(report, packet);
  }
  const omitted = envelopes();
  delete omitted.report.results[0].currentPath;
  rejects(omitted.report, omitted.packet);
});

test("report display aliases, explicit flag status and physical pixel arithmetic are independent guards", () => {
  for (const [key, changed] of Object.entries({
    art: "Left",
    variant: "Wrong",
    status: "failed",
    diffPixels: 1,
    totalPixels: 57599,
  })) {
    const { packet, report } = envelopes();
    Reflect.set(report.results[0], key, changed);
    rejects(report, packet);
  }
  const normalized = envelopes();
  const objectViewport = { name: "authored-compact", width: 320, height: 180 };
  Reflect.set(normalized.packet.results[0], "viewport", objectViewport);
  Reflect.set(normalized.report.results[0], "viewport", objectViewport);
  rejects(normalized.report, normalized.packet);
  for (const status of ["error", "new", "passed", "failed"] as const) {
    const { packet, report } = envelopes(status);
    report.results[0].status = status === "failed" ? "passed" : "failed";
    rejects(report, packet);
  }
  for (const changed of [-1, 57601, 0.5, Number.NaN]) {
    const { packet, report } = envelopes();
    report.results[0].diffPixels = changed;
    rejects(report, packet);
  }
});

test("new API and CLI phases pin every counter and exact new/passed result flags", () => {
  const expected = { new: 1, passed: 0, failed: 0 };
  const { packet, report } = envelopes("new");
  assert.doesNotThrow(() => assertSummary(packet, 1, expected));
  assert.doesNotThrow(() => assertSummary(report, 1, expected));
  for (const key of ["total", "new", "passed", "failed", "errors", "skipped"] as const) {
    for (const envelope of [structuredClone(packet), structuredClone(report)]) {
      envelope.summary[key]++;
      assert.throws(() => assertSummary(envelope, 1, expected), { name: "AssertionError" });
    }
  }
  for (const [key, changed] of [
    ["passed", false],
    ["isNew", false],
    ["diffPercentage", 0],
  ] as const) {
    const altered = structuredClone(packet);
    Reflect.set(altered.results[0], key, changed);
    assert.throws(() => assertSummary(altered, 1, expected), { name: "AssertionError" });
  }
  const missingFlag = structuredClone(packet);
  delete missingFlag.results[0].isNew;
  assert.throws(() => assertSummary(missingFlag, 1, expected), { name: "AssertionError" });
  const falseStatus = structuredClone(report);
  falseStatus.results[0].status = "passed";
  assert.throws(() => assertSummary(falseStatus, 1, expected), { name: "AssertionError" });
});

/** Actual product projector and frozen HTTP artifact registrations; no renderer/native substitute. */
async function sessionFixture(t: TestContext, status: "new" | "passed" | "failed" = "new") {
  const root = await mkdtemp(path.join(os.tmpdir(), "vize-public-session-law-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const { packet: privatePacket, report } = envelopes(status);
  const result = privatePacket.results[0],
    written = report.results[0];
  for (const field of ["snapshotPath", "currentPath", "diffPath"] as const) {
    if (!result[field]) continue;
    const file = path.join(root, `${field}.png`);
    // The bridge compares opaque complete bytes; PNG decoding remains a separate original law.
    await writeFile(file, Buffer.from(`${status}/${field}/whole artifact bytes`));
    result[field] = written[field] = file;
  }
  const jsonFile = path.join(root, "report.json");
  await writeFile(jsonFile, JSON.stringify(report));
  const json = await readFile(jsonFile),
    html = Buffer.from("<html>whole physical report</html>");
  const service = new SessionArtifacts();
  const projected = await service.results([
    {
      ...result,
      viewport: { name: "authored-compact", width: 320, height: 180 },
      diffPixels: written.diffPixels,
      totalPixels: written.totalPixels,
    },
  ]);
  const packet: SessionCapture = JSON.parse(
    JSON.stringify({
      success: true,
      summary: report.summary,
      results: projected,
      reports: {
        json: service.add(json, "application/json"),
        html: service.add(html, "text/html"),
      },
    }),
  );
  const server = createServer((request, response) => {
    if (!service.send(request.url ?? "", response)) {
      response.statusCode = 404;
      response.end("unregistered artifact");
    }
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  t.after(() => new Promise<void>((resolve) => server.close(() => resolve())));
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  const fetchArtifact = async (route: string, type: string) => {
    const response = await fetch(`http://127.0.0.1:${address.port}${route}`);
    assert.equal(response.status, 200);
    assert.equal(response.headers.get("content-type"), type);
    return Buffer.from(await response.arrayBuffer());
  };
  return {
    root,
    packet,
    service,
    width: 320,
    height: 180,
    report: { data: JSON.parse(json.toString()) as VrtReport, json, html },
    fetch: fetchArtifact,
  };
}

test("actual SessionArtifacts projection binds stored reports and every opaque image/report body", async (t) => {
  for (const status of ["new", "passed", "failed"] as const) {
    const input = await sessionFixture(t, status);
    assert.equal(Object.hasOwn(input.packet.results[0], "snapshotPath"), false);
    const bodies = await assertSessionReport(input);
    const images = input.packet.results[0].images;
    assert.equal(bodies.size, Object.keys(images).length + 2);
    assert.deepEqual(bodies.get(input.packet.reports.json), input.report.json);
    assert.deepEqual(bodies.get(input.packet.reports.html), input.report.html);
    assert.deepEqual(
      bodies.get(images.snapshot),
      await readFile(input.report.data.results[0].snapshotPath),
    );
  }
});

test("public projection refuses extra/missing fields, wrong statuses and every malformed artifact member", async (t) => {
  const input = await sessionFixture(t);
  const changes: Array<(packet: SessionCapture) => void> = [
    (p) => {
      Reflect.set(p, "unexpected", true);
    },
    (p) => {
      Reflect.deleteProperty(p, "reports");
    },
    (p) => {
      p.success = false;
    },
    (p) => {
      p.summary.new = 0;
    },
    (p) => {
      p.results = [];
    },
    (p) => {
      p.results.push(structuredClone(p.results[0]));
    },
    (p) => {
      Reflect.set(p.results[0], "unexpected", true);
    },
    (p) => {
      Reflect.set(p.results[0], "snapshotPath", "/private/path.png");
    },
    (p) => {
      Reflect.deleteProperty(p.results[0], "passed");
    },
    (p) => {
      p.results[0].passed = false;
    },
    (p) => {
      p.results[0].isNew = false;
    },
    (p) => {
      p.results[0].artPath += "foreign";
    },
    (p) => {
      p.results[0].viewport = "foreign";
    },
    (p) => {
      Reflect.deleteProperty(p.results[0].images, "snapshot");
    },
    (p) => {
      p.results[0].images.extra = p.results[0].images.current;
    },
    (p) => {
      p.results[0].images.snapshot = "https://foreign.invalid/artifacts/image";
    },
    (p) => {
      p.results[0].images.snapshot = "/artifacts/../private";
    },
    (p) => {
      p.results[0].images.snapshot = "/artifacts/not-a-registration";
    },
    (p) => {
      p.results[0].images.snapshot = "/artifacts/00000000-0000-4000-8000-000000000000";
    },
    (p) => {
      p.results[0].images.current = p.results[0].images.snapshot;
    },
    (p) => {
      Reflect.deleteProperty(p.reports, "html");
    },
    (p) => {
      Reflect.set(p.reports, "extra", p.reports.json);
    },
    (p) => {
      p.reports.json = p.results[0].images.current;
    },
    (p) => {
      p.reports.html = p.reports.json;
    },
  ];
  for (const change of changes) {
    const packet = structuredClone(input.packet);
    change(packet);
    await assert.rejects(assertSessionReport({ ...input, packet }));
  }
});

test("frozen session bodies refuse changed image/report bytes and symlinked physical ownership", async (t) => {
  const input = await sessionFixture(t);
  for (const route of [
    ...Object.values(input.packet.results[0].images),
    input.packet.reports.json,
    input.packet.reports.html,
  ]) {
    await assert.rejects(
      assertSessionReport({
        ...input,
        fetch: async (url, type) => {
          const bytes = await input.fetch(url, type);
          return url === route ? Buffer.concat([bytes, Buffer.from("tampered")]) : bytes;
        },
      }),
      /whole physical bytes/,
    );
  }
  const filename = input.report.data.results[0].snapshotPath;
  const original = await readFile(filename);
  await writeFile(filename, Buffer.concat([original, Buffer.from("changed after registration")]));
  await assert.rejects(assertSessionReport(input), /whole physical bytes/);
  await writeFile(filename, original);
  const link = path.join(input.root, "linked.png");
  await symlink(filename, link);
  input.report.data.results[0].snapshotPath = link;
  input.report.json = Buffer.from(JSON.stringify(input.report.data));
  await assert.rejects(assertSessionReport(input));
});
