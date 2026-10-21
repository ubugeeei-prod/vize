import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { environment7502 } from "./support/native-attribute-values-7502-loader.ts";

interface Capture {
  name: string;
  template: string;
  code: string;
}

const capturePath = process.env.VIZE_RUNTIME_PATCH_FLAGS_CAPTURE;
const required = process.env.VIZE_RUNTIME_PATCH_FLAGS_REQUIRE_CAPTURE === "1";
process.env.NODE_ENV = "development";
const environment = await environment7502("development");

async function execute(code: string) {
  const { component, runtime } = await environment.load(
    `${code}\nexport default { data: () => ({ label: "first" }), render }\n`,
    "dom",
  );
  const host = environment.window.document.createElement("div");
  const app = runtime.createApp(component);
  try {
    const proxy = app.mount(host) as { label: string };
    const before = host.textContent;
    proxy.label = "second";
    await runtime.nextTick();
    return { before, after: host.textContent };
  } finally {
    app.unmount();
  }
}

test(
  "static-attribute text roots update in the actual Vue runtime",
  {
    skip:
      !capturePath && !required ? "source capture is supplied by the native Actions lane" : false,
  },
  async () => {
    assert(capturePath, "a required source compiler capture must be provided");
    const captures: Capture[] = JSON.parse(readFileSync(capturePath, "utf8"));
    assert.equal(captures.length, 2);
    for (const capture of captures) {
      assert.match(capture.code, /, 1 \/\* TEXT \*\/\)\)/, capture.name);
      const expected = {
        before: capture.name.includes("text-and") ? "Hi first" : "first",
        after: capture.name.includes("text-and") ? "Hi second" : "second",
      };
      assert.deepEqual(await execute(capture.code), expected, capture.name);
    }
  },
);
