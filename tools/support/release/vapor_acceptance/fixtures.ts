import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";

export const reportedVue = "3.6.0-rc.10";
export const reportedRuntimeEntry = "vue/dist/vue.runtime.esm-bundler.js";
export const campaigns = [
  {
    issue: 8624,
    directory: "vapor-svg-8624",
    app: "App.vue.txt",
    child: "Child.vue.txt",
    count: 3,
  },
  {
    issue: 8625,
    directory: "vapor-root-fallthrough-8625",
    app: "App.vue.json",
    child: "Child.vue.txt",
    count: 1,
  },
  {
    issue: 8626,
    directory: "vapor-slot-destructure-8626",
    app: "App.vue.txt",
    child: "Child.vue.json",
    count: 1,
  },
  {
    issue: 8627,
    directory: "vapor-typed-element-handler-8627",
    app: "App.vue.txt",
    child: "Child.vue.txt",
    count: 1,
  },
] as const;
export const digest = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
export type Row = {
  issue: number;
  name: string;
  directory: string;
  app: string;
  child: string;
  context: Record<string, unknown>;
  steps: Record<string, unknown>[];
  rootProps?: Record<string, unknown>;
  childModule?: string;
  expectedTree?: unknown;
  expectedTrees?: unknown[];
  expectedNamespaces?: string[];
  expectedEvents?: unknown[];
  wrapper?: string;
};

/** Original encoded sources and all six original control rows remain immutable H blobs. */
export function materializeFixtures(root: string, read: (file: string) => Buffer) {
  const authorities: { file: string; sha256: string }[] = [];
  const rows: Row[] = [];
  for (const campaign of campaigns) {
    const prefix = `tests/_fixtures/differential/compiler/${campaign.directory}/`;
    const retained = (file: string) => {
      const source = prefix + file;
      const bytes = read(source);
      authorities.push({ file: source, sha256: digest(bytes) });
      const target = path.join(root, "original", campaign.directory, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, bytes, { flag: "wx" });
      return bytes.toString("utf8");
    };
    retained("original-issue.md");
    retained("reported.vue.txt");
    const decode = (file: string) => {
      const text = retained(file);
      const value = file.endsWith(".json") ? JSON.parse(text) : text;
      assert.equal(typeof value, "string", "complete original SFC required");
      return value;
    };
    const app = decode(campaign.app),
      child = decode(campaign.child);
    const controls = JSON.parse(retained("controls.json"));
    assert.equal(controls.length, campaign.count, "unchanged original control denominator");
    for (const control of controls) {
      assert.ok(control.name && Array.isArray(control.steps));
      assert.ok(control.context && typeof control.context === "object");
      assert.match(control.childModule ?? "./Child.vue", /^\.\/[A-Za-z]+\.vue$/u);
      rows.push({
        ...control,
        issue: campaign.issue,
        directory: campaign.directory,
        app: control.app ?? app,
        child: control.child ?? child,
      });
    }
    if (campaign.issue === 8624) {
      rows.push({
        issue: 8624,
        directory: campaign.directory,
        name: "authored-live-svg-tier-update",
        app,
        child,
        context: { tier: 1 },
        steps: [{ patch: { tier: 2 } }, { patch: { tier: 1 } }],
        wrapper:
          '<script setup vapor>import Original from "./App.vue"; import { state } from "./fixture";</script>\n<template><Original :tier="state.tier" /></template>\n',
      });
      rows.push({
        issue: 8624,
        directory: campaign.directory,
        name: "authored-svg-class-style-removal",
        app: '<script setup vapor>import { state } from "./fixture";</script>\n<template><svg id="icon" class="icon" :class="state.classes" :style="state.style" width="64" height="64" viewBox="0 0 64 64"><g><rect x="16" y="16" width="32" height="32" /></g></svg></template>\n',
        child,
        context: {
          classes: ["array", { active: true, absent: false }],
          style: { color: "rgb(200, 0, 0)" },
        },
        steps: [
          {
            patch: {
              classes: { selected: true, active: false },
              style: { color: "rgb(0, 0, 200)" },
            },
          },
          { patch: { classes: null, style: null } },
        ],
      });
    }
  }
  assert.equal(rows.length, 8, "six original rows and two separately authored SVG controls");
  return { authorities, rows };
}

export function assertTrace(
  row: Row,
  trace: { tree: unknown[]; namespaces: string[]; events: unknown[] }[],
) {
  assert.equal(trace.length, row.steps.length + 2, `${row.name}: complete ordered phases`);
  assert.ok(trace[0].tree.length, `${row.name}: original mount must render`);
  assert.deepEqual(trace.at(-1)?.tree, [], `${row.name}: actual unmount`);
  if (row.expectedTree) assert.deepEqual(trace[0].tree, row.expectedTree, row.name);
  if (row.expectedTrees)
    assert.deepEqual(
      trace.map((phase) => phase.tree),
      row.expectedTrees,
      row.name,
    );
  if (row.expectedNamespaces)
    assert.deepEqual(trace[0].namespaces, row.expectedNamespaces, row.name);
  if (row.expectedEvents) assert.deepEqual(trace.at(-2)?.events, row.expectedEvents, row.name);
  if (row.issue === 8624 && row.name.startsWith("original-")) {
    const root = trace[0].tree[0] as { attributes: Record<string, string> };
    const tier = row.rootProps!.tier;
    assert.equal(root.attributes.class, `icon tier-${String(tier)}`);
    assert.doesNotMatch(root.attributes.style, /\[object Object\]/u);
    assert.match(root.attributes.style, tier === 1 ? /rgb\(200, 0, 0\)/u : /rgb\(0, 0, 200\)/u);
  }
  if (row.wrapper) {
    for (const [index, tier] of [1, 2, 1].entries()) {
      assert.deepEqual(trace[index].namespaces, Array(3).fill("http://www.w3.org/2000/svg"));
      const root = trace[index].tree[0] as { attributes: Record<string, string> };
      assert.equal(root.attributes.class, `icon tier-${tier}`);
      assert.match(root.attributes.style, tier === 1 ? /rgb\(200, 0, 0\)/u : /rgb\(0, 0, 200\)/u);
    }
  }
  if (row.name === "authored-svg-class-style-removal") {
    for (const [index, expected] of ["icon array active", "icon selected", "icon"].entries()) {
      const root = trace[index].tree[0] as { attributes: Record<string, string> };
      assert.equal(root.attributes.class, expected);
      assert.deepEqual(trace[index].namespaces, Array(3).fill("http://www.w3.org/2000/svg"));
    }
  }
}
