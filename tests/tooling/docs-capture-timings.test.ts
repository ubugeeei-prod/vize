import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { runInNewContext } from "node:vm";
import { capturePageRender } from "../../docs/scripts/capture-page-render.ts";
import { CaptureTimings } from "../../docs/scripts/capture-render-timings.ts";

type CapturePage = Parameters<typeof capturePageRender>[0];
type Operation = { method: string; arguments: unknown[] };
function pageDouble(failure?: Error) {
  const calls: Operation[] = [];
  const world = {
    innerWidth: 1440,
    innerHeight: 960,
    scrollX: 0,
    scrollY: 7,
    document: { documentElement: { scrollHeight: 1200 } },
    scrollTo(x: number, y: number) {
      world.scrollX = x;
      world.scrollY = y;
    },
  };
  const record = (method: string, ...arguments_: unknown[]) =>
    calls.push({ method, arguments: structuredClone(arguments_) });
  const page = {
    viewportSize() {
      record("viewportSize");
      return { width: 1440, height: 960 };
    },
    async evaluate(operation: (...arguments_: never[]) => unknown, argument?: unknown) {
      record("evaluate", String(operation), argument);
      return runInNewContext(`(${String(operation)})(argument)`, { ...world, argument });
    },
    async screenshot(options: unknown) {
      record("screenshot", options);
      if (failure) throw failure;
      const header = Buffer.alloc(24);
      header.writeUInt32BE(1440, 16);
      header.writeUInt32BE(1200, 20);
      return header;
    },
    async setViewportSize(viewport: unknown) {
      record("setViewportSize", viewport);
    },
  };
  return { page: page as unknown as CapturePage, calls };
}

test("capture timing preserves original page calls, capture receipt and options", async () => {
  const output = await mkdtemp(path.join(os.tmpdir(), "vize-capture-timing-"));
  try {
    const plain = pageDouble();
    const observed = pageDouble();
    const timings = new CaptureTimings();
    const sink = timings.forJob({ index: 0, route: "/", device: "desktop" });
    const expected = await capturePageRender(plain.page, output, "page.png");
    const actual = await capturePageRender(observed.page, output, "page.png", sink);
    assert.deepEqual(actual, expected);
    assert.deepEqual(observed.calls, plain.calls);
    assert.deepEqual(
      observed.calls.filter(({ method }) => method === "screenshot"),
      [
        {
          method: "screenshot",
          arguments: [
            { path: path.join(output, "page.png"), fullPage: true, animations: "disabled" },
          ],
        },
      ],
    );
    await timings.write(output, {
      source: "705d927623a91a7bdd930b34f3656ed2d1862874",
      dist: "docs/dist",
      workers: 2,
      startedAt: new Date().toISOString(),
      status: "passed",
      scope: "general",
    });
    const sidecar = JSON.parse(await readFile(path.join(output, "capture-timings.json"), "utf8"));
    assert.equal(sidecar.screenshotIncludesPreparationEncodingTransferAndPngWrite, true);
    assert.equal(sidecar.workers, 2);
    assert.deepEqual(
      sidecar.events.map((event: { phase: string }) => event.phase),
      [
        "initial-viewport-scroll",
        "prepare",
        "tile-position",
        "screenshot-call-inclusive-write",
        "png-validation-hash",
        "final-coverage-layout",
        "restore-viewport",
        "restore-scroll",
        "snapshot-dispose",
      ],
    );
    for (const event of sidecar.events) {
      assert.equal(event.status, "passed");
      assert(event.durationMs >= 0);
      assert.equal(event.capture, "page.png");
    }
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});

test("capture timing retains original rejection and finally operations", async () => {
  const failure = new Error("original screenshot failure");
  const plain = pageDouble(failure);
  const observed = pageDouble(failure);
  const timings = new CaptureTimings();
  const sink = timings.forJob({ index: 3, route: "/rules/all", device: "desktop" });
  await assert.rejects(
    capturePageRender(plain.page, "/unused", "page.png"),
    (error) => error === failure,
  );
  await assert.rejects(
    capturePageRender(observed.page, "/unused", "page.png", sink),
    (error) => error === failure,
  );
  assert.deepEqual(observed.calls, plain.calls);
  const output = await mkdtemp(path.join(os.tmpdir(), "vize-capture-failure-timing-"));
  try {
    await timings.write(output, {
      source: null,
      dist: "docs/dist",
      workers: 2,
      startedAt: new Date().toISOString(),
      status: "failed",
      scope: "general",
    });
    const sidecar = JSON.parse(await readFile(path.join(output, "capture-timings.json"), "utf8"));
    const capture = sidecar.events.find(
      (event: { phase: string }) => event.phase === "screenshot-call-inclusive-write",
    );
    assert.equal(capture.status, "failed");
    assert.deepEqual(
      sidecar.events.slice(-3).map((event: { phase: string }) => event.phase),
      ["restore-viewport", "restore-scroll", "snapshot-dispose"],
    );
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});
