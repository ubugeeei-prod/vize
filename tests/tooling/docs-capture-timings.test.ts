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
function pageDouble(
  failure?: Error,
  control: { scrollFailure?: Error; readFailure?: Error; wrongScroll?: number } = {},
) {
  const calls: Operation[] = [];
  let evaluations = 0;
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
      evaluations += 1;
      if (evaluations === 4 && control.scrollFailure) throw control.scrollFailure;
      if (evaluations === 5 && control.readFailure) throw control.readFailure;
      if (evaluations === 5 && control.wrongScroll !== undefined) return control.wrongScroll;
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

test("host scroll observations retain separate RPC refusals and bounded tile identity", async (t) => {
  const scrollFailure = new Error("original scroll RPC rejection");
  const readFailure = new Error("original scroll read RPC rejection");
  const screenshotFailure = new Error("original screenshot rejection");
  const controls = [
    { name: "passed", status: "passed", operations: ["passed", "passed"], shots: 1 },
    {
      name: "first RPC rejection",
      control: { scrollFailure },
      error: scrollFailure,
      status: "failed",
      operations: ["failed"],
      shots: 0,
    },
    {
      name: "read RPC rejection",
      control: { readFailure },
      error: readFailure,
      status: "failed",
      operations: ["passed", "failed"],
      shots: 0,
    },
    {
      name: "wrong scrollY",
      control: { wrongScroll: 1 },
      status: "failed",
      operations: ["passed", "passed"],
      shots: 0,
    },
    {
      name: "screenshot rejection",
      screenshotFailure,
      error: screenshotFailure,
      status: "passed",
      operations: ["passed", "passed"],
      shots: 1,
    },
  ];
  for (const control of controls)
    await t.test(control.name, async () => {
      const output = await mkdtemp(path.join(os.tmpdir(), "vize-scroll-operation-timing-"));
      try {
        const plain = pageDouble(control.screenshotFailure, control.control);
        const observed = pageDouble(control.screenshotFailure, control.control);
        const timings = new CaptureTimings();
        const sink = timings.forJob({ index: 9, route: "/rules/all", device: "desktop" });
        const plainCapture = capturePageRender(plain.page, output, "page.png");
        const timedCapture = capturePageRender(observed.page, output, "page.png", sink);
        if (control.error) {
          await assert.rejects(plainCapture, (error) => error === control.error);
          await assert.rejects(timedCapture, (error) => error === control.error);
        } else if (control.name === "wrong scrollY") {
          for (const capture of [plainCapture, timedCapture])
            await assert.rejects(capture, (error) => {
              assert(error instanceof assert.AssertionError);
              assert.match(error.message, /page\.png: exact capture position/);
              assert.equal(error.actual, 1);
              assert.equal(error.expected, 0);
              return true;
            });
        } else assert.deepEqual(await timedCapture, await plainCapture);
        assert.deepEqual(observed.calls, plain.calls, "complete original functions/args/calls");
        assert.equal(
          observed.calls.filter(({ method }) => method === "screenshot").length,
          control.shots,
        );
        assert.deepEqual(
          observed.calls.slice(-2).map(({ method }) => method),
          ["setViewportSize", "evaluate"],
        );
        const evaluations = observed.calls.filter(({ method }) => method === "evaluate");
        assert.deepEqual(
          evaluations.slice(3, control.name === "first RPC rejection" ? 4 : 5),
          [
            { method: "evaluate", arguments: ["(top) => scrollTo(0, top)", 0] },
            ...(control.name === "first RPC rejection"
              ? []
              : [{ method: "evaluate", arguments: ["() => scrollY", undefined] }]),
          ],
          "the original tile RPC bodies and arguments remain distinct and ordered",
        );
        await timings.write(output, {
          source: null,
          dist: "docs/dist",
          workers: 2,
          startedAt: new Date().toISOString(),
          status: control.name === "passed" ? "passed" : "failed",
          scope: "general",
        });
        type Event = {
          phase: string;
          startMs: number;
          durationMs: number;
          status: string;
          jobIndex: number;
          route: string;
          device: string;
          capture: string;
          tileIndex?: number;
          top?: number;
          operations?: { name: string; startMs: number; durationMs: number; status: string }[];
        };
        const sidecar = JSON.parse(
          await readFile(path.join(output, "capture-timings.json"), "utf8"),
        );
        const events: Event[] = sidecar.events;
        const tile = events.find(({ phase }) => phase === "tile-position");
        assert(tile);
        assert.equal(tile.status, control.status);
        assert.deepEqual(
          tile.operations?.map(({ name }) => name),
          control.operations.length === 1 ? ["scroll-to"] : ["scroll-to", "scroll-read"],
        );
        assert.deepEqual(
          tile.operations?.map(({ status }) => status),
          control.operations,
        );
        for (const event of events) {
          assert(Number.isFinite(event.startMs) && Number.isFinite(event.durationMs));
          assert(event.durationMs >= 0);
          assert.equal(event.jobIndex, 9);
          assert.equal(event.route, "/rules/all");
          assert.equal(event.device, "desktop");
          assert.equal(event.capture, "page.png");
          if (event.phase !== "tile-position") assert.equal(event.operations, undefined);
        }
        assert.equal(tile.tileIndex, 0);
        assert.equal(tile.top, 0);
        let previousEnd = tile.startMs;
        for (const operation of tile.operations!) {
          assert(Number.isFinite(operation.startMs) && Number.isFinite(operation.durationMs));
          assert(operation.durationMs >= 0);
          assert(operation.startMs >= previousEnd);
          previousEnd = operation.startMs + operation.durationMs;
          assert(previousEnd <= tile.startMs + tile.durationMs);
        }
        assert.deepEqual(
          events.slice(-3).map(({ phase }) => phase),
          ["restore-viewport", "restore-scroll", "snapshot-dispose"],
        );
      } finally {
        await rm(output, { recursive: true, force: true });
      }
    });
});
