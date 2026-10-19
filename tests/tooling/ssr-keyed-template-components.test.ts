// Admission refusals only: these placeholders are never native/compiler evidence.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { observeKeyedTemplates } from "./support/ssr-keyed-template-components.ts";

const raw = readFileSync(
  new URL("../_fixtures/differential/compiler/ssr-keyed-template-components.json", import.meta.url),
  "utf8",
);
const fixture = JSON.parse(raw);
function input() {
  return {
    fixture: raw,
    cases: fixture.cases.flatMap((entry: { name: string; source: string }) =>
      fixture.targets.map((target: string) => ({
        name: entry.name,
        target,
        files: ["Value", "TriggerPush", "TriggerVNode", "App"].map((name) => ({
          name,
          source: name === "App" ? entry.source : fixture.files[name],
          current: {
            code: "admission placeholder; must never execute",
            errors: [],
            warnings: [],
            map: {},
          },
        })),
      })),
    ),
  } as Parameters<typeof observeKeyedTemplates>[0];
}

for (const [name, mutate, reason] of [
  [
    "changed fixture bytes",
    (p: ReturnType<typeof input>) => {
      p.fixture += "\n";
    },
    /all native-authored fixture bytes/,
  ],
  [
    "dropped target",
    (p: ReturnType<typeof input>) => {
      p.cases.pop();
    },
    /complete target inventory/,
  ],
  [
    "duplicate target",
    (p: ReturnType<typeof input>) => {
      const first = p.cases.at(0);
      assert(first);
      p.cases.splice(-1, 1, first);
    },
    /unique native packet/,
  ],
  [
    "changed authored source",
    (p: ReturnType<typeof input>) => {
      const file = p.cases.at(0)?.files.at(0);
      assert(file);
      file.source += "\n";
    },
    /authored source/,
  ],
  [
    "compile warning",
    (p: ReturnType<typeof input>) => {
      const file = p.cases.at(0)?.files.at(0);
      assert(file);
      file.current.warnings.push("not clean");
    },
    /compiler warnings/,
  ],
  [
    "missing source map",
    (p: ReturnType<typeof input>) => {
      const file = p.cases.at(0)?.files.at(0);
      assert(file);
      file.current.map = null;
    },
    /complete source map/,
  ],
] as const) {
  test(`keyed template carrier refuses ${name} before evaluation`, async () => {
    const packet = input();
    mutate(packet);
    let calls = 0;
    const forbidden = () => {
      calls++;
      throw new Error("must not evaluate admission placeholders");
    };
    await assert.rejects(
      observeKeyedTemplates(packet, {
        reference: forbidden,
        evaluate: forbidden,
        evaluateModule: forbidden,
      } as unknown as Parameters<typeof observeKeyedTemplates>[1]),
      reason,
    );
    assert.equal(calls, 0);
  });
}

test("keyed template carrier refuses unavailable Vapor hydration instead of skipping", async () => {
  let calls = 0;
  const forbidden = () => {
    calls++;
    throw new Error("must not evaluate admission placeholders");
  };
  const seams = {
    vue: { version: "3.6.0-rc.9" },
    vapor: { version: "3.6.0-rc.9" },
    compiler: { version: "3.6.0-rc.9" },
    reference: forbidden,
    evaluate: forbidden,
    evaluateModule: forbidden,
  } as unknown as Parameters<typeof observeKeyedTemplates>[1];
  await assert.rejects(
    observeKeyedTemplates(input(), seams),
    /required supported Vapor hydration API/,
  );
  assert.equal(calls, 0);
});

test("keyed template carrier refuses unavailable Vapor VDOM interop instead of skipping", async () => {
  let calls = 0;
  const forbidden = () => {
    calls++;
    throw new Error("must not evaluate admission placeholders");
  };
  const seams = {
    vue: { version: "3.6.0-rc.9" },
    vapor: { version: "3.6.0-rc.9", createVaporSSRApp: forbidden },
    compiler: { version: "3.6.0-rc.9" },
    reference: forbidden,
    evaluate: forbidden,
    evaluateModule: forbidden,
  } as unknown as Parameters<typeof observeKeyedTemplates>[1];
  await assert.rejects(
    observeKeyedTemplates(input(), seams),
    /required supported Vapor VDOM interop plugin/,
  );
  assert.equal(calls, 0);
});
