import { expect, it } from "vite-plus/test";
import { isUsingMock, loadWasm } from "../src/wasm/index";
import raw from "../../tests/_fixtures/differential/compiler/unicode-script-line-comments-11798/cases.json.txt?raw";
import registration from "../../tests/_fixtures/differential/compiler/unicode-script-line-comments-11798/corpus.json";

it("#11798 real WASM preserves all 36 complete SFC packets", async () => {
  const wasm = await loadWasm();
  expect(isUsingMock()).toBe(false);
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(raw));
  const corpusSha256 = Array.from(new Uint8Array(digest), (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("");
  expect(corpusSha256).toBe(registration.sha256);
  const corpus = JSON.parse(raw);
  expect(corpus.cases.length).toBe(36);
  const observations = [];
  try {
    for (const row of corpus.cases) {
      const result = wasm.compileSfc(row.source, { filename: "Case.vue", ssr: row.ssr });
      observations.push({ id: row.id, source: row.source, result });
      expect(result, row.id).toEqual(row.wasm);
    }
  } finally {
    console.log("11798_WASM_RAW", JSON.stringify({ corpusSha256, observations }));
  }
});
