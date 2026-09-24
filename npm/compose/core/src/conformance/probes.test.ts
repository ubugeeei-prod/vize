import assert from "node:assert/strict";
import { test } from "node:test";

import { COMPOSABLE_CATALOG } from "../catalog.ts";
import { probesGroupA } from "./probes-a.ts";
import { probesGroupB } from "./probes-b.ts";
import { probesGroupC } from "./probes-c.ts";
import { probesGroupD } from "./probes-d.ts";
import { probesGroupE } from "./probes-e.ts";
import { probesGroupF } from "./probes-f.ts";
import { composableProbes } from "./probes.ts";

void test("every catalog entry has exactly one conformance probe", () => {
  const entries = COMPOSABLE_CATALOG.entries.map((entry) => entry.subpath).sort();
  assert.deepEqual(Object.keys(composableProbes).sort(), entries);
  const groups = [
    probesGroupA,
    probesGroupB,
    probesGroupC,
    probesGroupD,
    probesGroupE,
    probesGroupF,
  ];
  const total = groups.reduce((sum, group) => sum + Object.keys(group).length, 0);
  assert.equal(total, entries.length, "a subpath is probed by more than one group");
});

void test("exempt probes explain why nothing is mounted", () => {
  for (const [subpath, probe] of Object.entries(composableProbes)) {
    if (probe.kind === "exempt") assert.ok(probe.reason.length > 10, `${subpath} needs a reason`);
    else assert.ok(probe.setup.includes('from "@/'), `${subpath} must import the entry via "@/"`);
  }
});
