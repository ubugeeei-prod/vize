import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { wholeError } from "../hosts/contracts.ts";
import { hash, json, here, exists } from "./environment.ts";
import { compilePair } from "./compile.ts";
import { prepareReference } from "./reference.ts";
import { observeBrowser } from "./browser.ts";
import type {
  BrowserObservation,
  RuntimeSnapshot,
  DomTree,
  RuntimeInput,
  RuntimeLoader,
  RuntimeReport,
  ServerReport,
} from "./types.ts";
function assertSnapshot(snapshot: RuntimeSnapshot, index: number) {
  assert.deepEqual(
    snapshot.metrics.setup,
    { App: 1, Counter: 1 },
    "setup runs exactly once, never as a render callback",
  );
  assert.equal(snapshot.metrics.childEvents, index >= 2 ? 1 : 0);
  assert.equal(snapshot.metrics.parentEvents, index >= 2 ? 1 : 0);
  assert.deepEqual(
    snapshot.metrics.events,
    index >= 2
      ? [
          { owner: "Counter", type: "click", trusted: true },
          { owner: "App", type: "click", trusted: true },
        ]
      : [],
  );
  assert.deepEqual(snapshot.diagnostics, { warnings: [], errors: [] });
  const nodes: DomTree[] = [],
    pending = [...snapshot.tree];
  while (pending.length) {
    const node = pending.shift()!;
    nodes.push(node);
    pending.push(...node.children);
  }
  const button = nodes.filter((node) => node.nodeName === "BUTTON");
  assert.equal(button.length, 1);
  const attrs = Object.fromEntries(button[0].attributes!);
  assert.equal(attrs.type, "button");
  assert.equal(attrs.id, index === 1 || index === 2 ? "counter-updated" : "counter-initial");
  assert.equal(attrs["data-probe"], index === 1 || index === 2 ? "更新 café" : "雪 🧭");
  assert.equal(
    attrs.class,
    index === 1 || index === 2 ? "counter-own parent-warm" : "counter-own parent-cold",
  );
  assert.equal(button[0].disabled, false);
  assert.deepEqual(button[0].style, {
    color: index === 1 || index === 2 ? "red" : "blue",
    fontWeight: "700",
  });
  const text = button[0].children.map((node) => node.data ?? "").join("");
  assert.equal(text, ["雪 🧭:0", "更新 café:0", "更新 café:1", "雪 🧭:0"][index]);
}
export async function runRuntime(input: RuntimeInput, load: RuntimeLoader) {
  const report: RuntimeReport = {
    input,
    compiled: null,
    reference: null,
    server: null,
    browser: null,
    error: null,
  };
  const outputId = hash(Buffer.from(input.output)).slice(0, 16);
  const directory = path.join(
    input.runtime.root,
    "node_modules/.vize-babel-runtime",
    outputId,
    input.id,
  );
  try {
    assert(
      !exists(directory),
      "generated runtime modules must be fresh, never stale or redirected",
    );
    fs.mkdirSync(directory, { recursive: true });
    report.directory = directory;
    report.reference = prepareReference(input.runtime, directory);
    report.compiled = await compilePair(input, load, directory);
    const serverInput = path.join(directory, "server.input.json"),
      serverReport = path.join(directory, "server.json");
    json(serverInput, { runtime: input.runtime, directory, report: serverReport });
    const server = spawnSync(process.execPath, [path.join(here, "node-ssr.ts"), serverInput], {
      maxBuffer: 64 * 1024 * 1024,
      timeout: 60000,
    });
    fs.writeFileSync(path.join(directory, "server.stdout.raw"), server.stdout ?? Buffer.alloc(0));
    fs.writeFileSync(path.join(directory, "server.stderr.raw"), server.stderr ?? Buffer.alloc(0));
    json(path.join(directory, "server.process.json"), {
      status: server.status,
      signal: server.signal,
      error: server.error ? wholeError(server.error) : null,
    });
    report.server = fs.existsSync(serverReport)
      ? (JSON.parse(fs.readFileSync(serverReport, "utf8")) as ServerReport)
      : null;
    assert(report.server, "whole actual Node SSR report is mandatory");
    // Collect all browser counterparts before comparing complete raw output.
    report.browser = await observeBrowser(input, directory, report.server);
    assert.equal(server.status, 0);
    assert.equal(server.signal, null);
    assert.equal(server.stderr.length, 0);
    assert(report.server.observations.every((row) => row.failure === null));
    assert.equal(
      report.server.observations[0].html,
      report.server.observations[1].html,
      "entire raw SSR HTML equals the independently authored real Vue control",
    );
    assert.equal(report.browser.observations.length, 4);
    for (const row of report.browser.observations) {
      assert.equal(row.failure, null);
      assert.deepEqual(row.pageErrors, []);
      assert.deepEqual(row.requestFailures, []);
      assert.deepEqual(
        row.console.filter((message) => message.type === "warning" || message.type === "error"),
        [],
      );
      assert.deepEqual(
        row.snapshots.map((snapshot) => snapshot.name),
        ["initial", "updated", "clicked", "restored"],
      );
      row.snapshots.forEach(assertSnapshot);
      assert.equal(row.dispose!.html, "", "real app unmount releases the whole rendered block");
      assert.deepEqual(row.dispose!.diagnostics, { warnings: [], errors: [] });
      if (row.mode === "hydrate") {
        assert(row.hydration!.originalCount > 0);
        assert.equal(row.hydration!.sameCount, true);
        assert(
          row.hydration!.retainedInOrder.every(Boolean),
          "all original SSR nodes, including comments/text, retain identity",
        );
      }
    }
    for (const mode of ["mount", "hydrate"]) {
      const candidate: BrowserObservation | undefined = report.browser.observations.find(
        (row) => row.mode === mode && row.kind === "candidate",
      );
      const reference: BrowserObservation | undefined = report.browser.observations.find(
        (row) => row.mode === mode && row.kind === "reference",
      );
      assert.deepEqual(
        candidate!.snapshots,
        reference!.snapshots,
        "every raw HTML, complete DOM, authored effect and diagnostic packet matches the real Vue control",
      );
    }
  } catch (error) {
    report.error = wholeError(error);
  } finally {
    try {
      if (fs.existsSync(directory)) {
        const preserved = path.join(input.output, input.id + "-artifacts");
        assert(!exists(preserved), "preserve runtime artifacts without overwriting earlier proof");
        fs.cpSync(directory, preserved, { recursive: true, force: false, errorOnExist: true });
      }
    } catch (error) {
      report.preservationError = wholeError(error);
      report.error ??= report.preservationError;
    }
    json(input.report, report);
  }
  process.stdout.write(
    JSON.stringify({ id: input.id, report: input.report, failed: report.error !== null }) + "\n",
  );
  if (report.error) process.exitCode = 1;
}
