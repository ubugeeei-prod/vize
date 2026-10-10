// Inert custody rejection/contract laws only. These grant no installed/native runtime credit.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  installedAuthority,
  sha256,
  validateCampaignPlan,
  type InstalledCampaignPlan,
} from "./support/n8n-installed-authority.ts";
import {
  installedCliRuntime,
  installedInvocationPreflight,
  validateNativeJournal,
} from "./support/n8n-installed-cli.ts";
import { root } from "./support/n8n-cli-config-inputs.mjs";
import { runtimeOverrides } from "../../tools/support/release/public_acceptance/installed.ts";

// Isolate inert laws from the test host's flags; real installed calls still reject every override.
function withoutAmbientOverrides(run: () => void): void {
  const before = runtimeOverrides.map((key) => [key, process.env[key]] as const);
  try {
    for (const key of runtimeOverrides) delete process.env[key];
    run();
  } finally {
    for (const [key, value] of before) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
}

const plan = (): InstalledCampaignPlan => ({
  schema: "vize.n8n.installed-campaign-v1",
  source: {
    C: "26031a4fbb30a1e86511919bafc7035b08440ef3",
    H: "a26243855bcf81005049252a6e71b6dd55e457f1",
    tag: "v0.439.0",
    R: "38011924629",
    sourcePr: "8351",
  },
  installReceipt: { path: "/inert/never-executed.json", sha256: "0".repeat(64) },
  collectorSha256: "a26411442a3fde2590d969160fd83b97685e9019d2de4decb0c103a376f10786",
});

test("installed campaign requires independently pinned source and collector/receipt identities", () => {
  validateCampaignPlan(plan()); // Syntax only; this does not grant installation credit.
  const mutations: Array<(value: InstalledCampaignPlan) => void> = [
    (value) => {
      value.source.H = value.source.C;
    },
    (value) => {
      value.source.H = "main";
    },
    (value) => {
      value.source.tag = "v0.439.1";
    },
    (value) => {
      value.source.R = "0";
    },
    (value) => {
      value.source.sourcePr = "8351/other";
    },
    (value) => {
      value.installReceipt.sha256 = "not-a-digest";
    },
    (value) => {
      value.collectorSha256 = "";
    },
  ];
  for (const change of mutations) {
    const value = plan();
    change(value);
    assert.throws(() => validateCampaignPlan(value));
  }
});

test("wrong or changed SHA-pinned install receipt is refused before provider startup", (t) => {
  withoutAmbientOverrides(() => {
    const temporary = fs.realpathSync(
      fs.mkdtempSync(path.join(os.tmpdir(), "n8n-installed-refusal-")),
    );
    t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
    const receipt = path.join(temporary, "inert-receipt.json");
    const value = plan();
    value.installReceipt.path = receipt;
    fs.writeFileSync(
      receipt,
      JSON.stringify({
        schema: "vize-public-registry-install-v1",
        success: true,
        version: "0.439.0",
        source: { ...value.source, H: "1".repeat(40) },
      }),
    );
    assert.throws(() => installedAuthority(value, root), /reviewed install receipt changed/u);
    value.installReceipt.sha256 = sha256(fs.readFileSync(receipt));
    assert.throws(() => installedAuthority(value, root), assert.AssertionError);
    fs.appendFileSync(receipt, "\n");
    assert.throws(() => installedAuthority(value, root), /reviewed install receipt changed/u);
    const output = path.join(temporary, "no-output");
    assert.throws(
      () => installedCliRuntime(output, ""),
      /explicit reviewed installed-campaign plan/u,
    );
    assert.equal(fs.existsSync(output), false);
  });
});

test("every ambient runtime/source override is refused before receipt access", () => {
  withoutAmbientOverrides(() => {
    for (const key of runtimeOverrides) {
      const before = process.env[key];
      try {
        process.env[key] = "inert-forbidden-value";
        assert.throws(
          () => installedAuthority(plan(), root),
          new RegExp("runtime override must be absent: " + key, "u"),
        );
      } finally {
        if (before === undefined) delete process.env[key];
        else process.env[key] = before;
      }
    }
  });
});

test("a newly introduced ambient preload is refused at per-call preflight before payload access", () => {
  withoutAmbientOverrides(() => {
    let observedPayload = false;
    const before = process.env.NODE_OPTIONS;
    try {
      process.env.NODE_OPTIONS = "--require /inert/preload-that-must-never-run.cjs";
      assert.throws(
        () =>
          installedInvocationPreflight(() => {
            observedPayload = true;
          }),
        /runtime override must be absent: NODE_OPTIONS/u,
      );
      assert.equal(observedPayload, false);
    } finally {
      if (before === undefined) delete process.env.NODE_OPTIONS;
      else process.env.NODE_OPTIONS = before;
    }
  });
});

const expected = {
  installRoot: "/inert/public-install",
  nativePath: "/inert/public-install/node_modules/@vizejs/native-darwin-arm64/vize.node",
  nativeSha256: "1".repeat(64),
  node: "/inert/node",
  argv: ["/inert/node", "/inert/public-install/node_modules/vize/bin/vize", "lint"],
  pid: 1234,
  status: 1,
  corsaPath: "/inert/public-install/node_modules/@typescript/typescript-darwin-arm64/lib/tsc",
};
function symbolicJournal(): Array<Record<string, unknown>> {
  const common = { schema: "vize-public-native-custody-event-v1", pid: expected.pid };
  return [
    {
      ...common,
      event: "initialized",
      installRoot: expected.installRoot,
      nativePath: expected.nativePath,
      sha256: expected.nativeSha256,
      node: expected.node,
      argv: expected.argv,
    },
    {
      ...common,
      event: "attempt",
      expectedNative: true,
      actualPath: expected.nativePath,
      sha256: expected.nativeSha256,
    },
    {
      ...common,
      event: "returned",
      expectedNative: true,
      actualPath: expected.nativePath,
      sha256: expected.nativeSha256,
      corsaPath: expected.corsaPath,
    },
    { ...common, event: "exit", code: 1 },
  ];
}
const encode = (events: Array<Record<string, unknown>>) =>
  events.map((event) => JSON.stringify(event)).join("\n") + "\n";

test("native journal laws distinguish successful loader return from ordinary lint exit one", () => {
  validateNativeJournal(encode(symbolicJournal()), expected); // Inert protocol model, never native execution.
  const zero = symbolicJournal();
  zero[3].code = 0;
  validateNativeJournal(encode(zero), { ...expected, status: 0 });
});

test("missing/failed/foreign native return and changed CLI/PID/argv/exit routes are rejected", () => {
  const mutations: Array<(events: Array<Record<string, unknown>>) => void> = [
    (events) => {
      events.splice(2, 1);
    },
    (events) => {
      events[2].event = "failed";
      events[2].errno = -2;
    },
    (events) => {
      events[2].event = "rejected";
    },
    (events) => {
      events[2].actualPath = "/inert/workspace/alternate.node";
    },
    (events) => {
      events[2].sha256 = "2".repeat(64);
    },
    (events) => {
      events[2].pid = 4321;
    },
    (events) => {
      events[2].corsaPath = "/inert/workspace/tsgo";
    },
    (events) => {
      events[0].argv = ["/inert/node", "workspace-vize", "lint"];
    },
    (events) => {
      events[3].code = 0;
    },
    (events) => {
      events.push(events[2]);
    },
    (events) => {
      [events[1], events[2]] = [events[2], events[1]];
    },
    (events) => {
      events.splice(2, 0, events[2]);
    },
    (events) => {
      events.splice(0, 0, events[0]);
    },
  ];
  for (const mutate of mutations) {
    const events = symbolicJournal();
    mutate(events);
    assert.throws(() => validateNativeJournal(encode(events), expected));
  }
  assert.throws(() => validateNativeJournal(encode(symbolicJournal()).trimEnd(), expected));
});
