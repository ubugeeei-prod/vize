// Capture independent official Vue compiler output, never candidate Vize bytes.
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { compileTemplate } from "@vue/compiler-sfc";

const fixture = "tests/fixtures/vdom/pure-comment-recovery.pkl";
const snapshot = "tests/expected/vdom/pure-comment-recovery.snap";
const result = spawnSync("pkl", ["eval", "--format", "json", fixture], { encoding: "utf8" });
if (result.error) throw result.error;
if (result.status !== 0) throw new Error(result.stderr);
const cases = JSON.parse(result.stdout).cases as {
  name: string;
  input: string;
  options: { hoistStatic?: boolean; cacheHandlers?: boolean; ssr?: boolean };
}[];
if (cases.length !== 2) throw new Error("Expected the two authored rewind cases");
let content = "";
for (const testCase of cases) {
  const compiled = compileTemplate({
    source: testCase.input,
    filename: "test.vue",
    id: "test",
    compilerOptions: {
      mode: "module",
      prefixIdentifiers: true,
      hoistStatic: testCase.options.hoistStatic ?? false,
      cacheHandlers: testCase.options.cacheHandlers ?? false,
      ssr: testCase.options.ssr ?? false,
    },
  });
  if (compiled.errors.length !== 0) throw new Error(JSON.stringify(compiled.errors));
  content += `===\nname: ${testCase.name}\noptions: vdom\n--- INPUT ---\n${testCase.input.trim()}\n--- OUTPUT ---\n${compiled.code.trim()}\n`;
}
writeFileSync(snapshot, content);
const require = createRequire(import.meta.url);
const packageInfo = JSON.parse(
  readFileSync(require.resolve("@vue/compiler-sfc/package.json"), "utf8"),
);
const sha256 = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const receipt = {
  schema: 1,
  oracle: { package: "@vue/compiler-sfc", version: packageInfo.version },
  producer: path.relative(process.cwd(), import.meta.filename),
  fixtureSha256: sha256(readFileSync(fixture)),
  snapshotSha256: sha256(content),
  cases: cases.length,
};
writeFileSync(
  snapshot.replace(".snap", ".provenance.json"),
  `${JSON.stringify(receipt, null, 2)}\n`,
);
console.log(JSON.stringify(receipt));
