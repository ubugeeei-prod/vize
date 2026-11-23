import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, textFile, writeJson, fileIdentity } from "../shared/files.ts";

import { readdirSync } from "node:fs";
const root = process.argv[2]!;
const stdout = textFile(join(root, "test.stdout"));
assert.ok(
  stdout.includes("test every_sfc_backend_keeps_host_globals_on_the_component_proxy ... ok"),
);
assert.match(stdout, /test result: ok[.] 1 passed; 0 failed; 0 ignored;/u);
const expected = ["globals", "local", "loop", "import"]
  .flatMap((name) =>
    ["dom", "vapor", "ssr"].flatMap((target) =>
      [false, true].map((inline) => [name, target, inline].join("-") + ".json"),
    ),
  )
  .sort();
assert.deepEqual(
  readdirSync(root)
    .filter((name) => name.endsWith(".json"))
    .sort(),
  [...expected, "legacy-fixture.json"].sort(),
);
function packet(name: string) {
  const file = join(root, name),
    whole = readJson<{ wholeSource: string; wholeResult: { code: string }; options: unknown }>(
      file,
    );
  assert.ok(whole.wholeSource && whole.wholeResult.code);
  const { bytes, sha256 } = fileIdentity(file);
  return { whole, identity: { name, bytes, sha256 } };
}
const wholePackets = expected.map((name) => packet(name).identity);
const legacy = packet("legacy-fixture.json");
assert.deepEqual(legacy.whole.options, {
  filename: "Layout.vue",
  scriptIsTs: false,
  templateIsTs: false,
  ssr: true,
  scriptOutput: "InlineTemplate",
});
writeJson(join(root, "receipt.json"), {
  producer: textFile(join(root, "producer.sha")).trim(),
  executed: 1,
  wholePackets,
  auxiliaryLegacyFixture: legacy.identity,
});
