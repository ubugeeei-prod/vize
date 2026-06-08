import { describe, expect, it } from "vite-plus/test";
import {
  createInspectorUrl,
  createPullRequestBody,
  decodeInspectorPayload,
  encodeInspectorPayload,
  readInspectorPayloadFromUrl,
} from "./share";
import type { InspectorPayload } from "./types";

const payload: InspectorPayload = {
  version: 1,
  target: "dom",
  selectedFile: "src/App.vue",
  options: {
    customRenderer: false,
    templateSyntax: "quirks",
  },
  files: [
    {
      path: "src/App.vue",
      source: "<template><div>{{ msg }}</div></template>",
    },
  ],
};

describe("inspector share payloads", () => {
  it("round-trips encoded payloads", () => {
    expect(decodeInspectorPayload(encodeInspectorPayload(payload))).toEqual(payload);
  });

  it("creates and reads playground URLs", () => {
    const url = createInspectorUrl(payload, "https://vizejs.dev/play/?tab=atelier");

    expect(url).toMatch(/^https:\/\/vizejs\.dev\/play\/\?tab=inspector#inspector=/);
    expect(readInspectorPayloadFromUrl(url)).toEqual(payload);
  });

  it("builds a complete PR body with the permalink and diff summary", () => {
    const body = createPullRequestBody({
      permalink: "https://vizejs.dev/play/?tab=inspector#inspector=abc",
      payload,
      stats: {
        additions: 2,
        removals: 1,
        unchanged: 5,
      },
    });

    expect(body).toEqual(`## Compiler inspector

Playground permalink: https://vizejs.dev/play/?tab=inspector#inspector=abc

### Scope

- Target: DOM
- Files: 1
- Output delta: +2 / -1
- Includes: Vue output, Vize output, Virtual TS, VIR, and cross-file graph

### Repro files

- \`src/App.vue\`

### Notes

Generated from the Vize playground compiler inspector. Please add the minimized fixture or full snapshot that explains the parity change.`);
  });

  it("labels vapor PR bodies", () => {
    const body = createPullRequestBody({
      permalink: "https://vizejs.dev/play/?tab=inspector#inspector=abc",
      payload: { ...payload, target: "vapor" },
      stats: {
        additions: 0,
        removals: 0,
        unchanged: 1,
      },
    });

    expect(body).toContain("- Target: Vapor");
  });
});
