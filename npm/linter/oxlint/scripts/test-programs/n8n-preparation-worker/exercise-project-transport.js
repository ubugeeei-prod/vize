import fs from "node:fs";
import { createHash } from "node:crypto";
const custody = JSON.parse(fs.readFileSync(process.env.VIZE_OXLINT_NATIVE_CUSTODY));
const common = {
  source: custody.source,
  binary: custody.binary.path,
  binarySha256: custody.binary.sha256,
};
const source = "controlled verifier input",
  sha = createHash("sha256").update(source).digest("hex");
const rows = [
  ...Array.from({ length: 5 }, (_, i) => ({
    ...common,
    kind: "load",
    pid: i + 1,
    control: "Guard-" + i + ".vue",
  })),
  { ...common, kind: "load", pid: 10 },
  ...Array.from({ length: 18 }, () => ({
    ...common,
    kind: "call",
    pid: 10,
    outcome: "return",
    hasRuleHint: true,
    args: [source, { filename: "/Controlled.vue", enabledRules: Array(51).fill("control-rule") }],
    originalSha256: sha,
  })),
];
fs.writeFileSync(custody.calls, rows.map((x) => JSON.stringify(x)).join("\n") + "\n");
fs.writeFileSync(
  process.env.VIZE_OXLINT_PROJECT_CAPTURE,
  JSON.stringify({ qualified: { imports: true, typeAware: true } }),
);
console.log("complete transport raw output");
