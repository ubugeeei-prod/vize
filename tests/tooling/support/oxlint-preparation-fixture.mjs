import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { referenceSource } from "../../../npm/oxlint/scripts/n8n-reference-source.mjs";

export const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const read = (file) => JSON.parse(fs.readFileSync(file));
export async function until(predicate) {
  const deadline = performance.now() + 6000;
  while (!predicate()) {
    if (performance.now() >= deadline) throw new Error("controlled process observation deadline");
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}

// Controlled bytes exercise scheduling and custody refusal only. They never
// qualify a product addon, an n8n diagnostic, or a performance measurement.
export function preparationFixture(t) {
  const artifacts = fs.mkdtempSync(path.join(os.tmpdir(), "vize-preparation-"));
  t.after(() => fs.rmSync(artifacts, { recursive: true, force: true }));
  const binary = path.join(artifacts, "controlled-binary.bin");
  fs.writeFileSync(binary, "controlled process-law bytes\n");
  const receipt = {
    source: { head: "a".repeat(40), tree: "b".repeat(40), workingDiff: hash("") },
    toolchain: { rustc: "controlled compiler identity" },
    frozen: { sha256: hash(fs.readFileSync(binary)) },
  };
  fs.writeFileSync(path.join(artifacts, "build-receipt.json"), JSON.stringify(receipt));
  const output = path.join(artifacts, "n8n");
  fs.mkdirSync(path.join(output, "bridge"), { recursive: true });
  const custody = {
    schema: "vize.oxlint.n8n-source-native",
    version: 1,
    source: receipt.source,
    toolchain: receipt.toolchain,
    binary: { path: fs.realpathSync(binary), sha256: receipt.frozen.sha256 },
    calls: path.join(output, "native-calls.jsonl"),
    sources: path.join(output, "sources"),
  };
  fs.writeFileSync(custody.calls, "controlled joined native calls\r\n");
  fs.writeFileSync(
    path.join(output, "bridge/bridge-vectors.jsonl"),
    "controlled whole vectors\r\n",
  );
  const referenceRoot = path.join(artifacts, "reference-runtime");
  fs.mkdirSync(path.join(referenceRoot, "dist"), { recursive: true });
  fs.writeFileSync(
    path.join(referenceRoot, "source.mjs"),
    "controlled original reference source\n",
  );
  const bundles = ["dist/index.mjs", "dist/cli.mjs"].map((file) => {
    const bytes = `controlled original reference bundle ${file}\n`;
    fs.writeFileSync(path.join(referenceRoot, file), bytes);
    const saved = path.join(output, "reference-bundles", file);
    fs.mkdirSync(path.dirname(saved), { recursive: true });
    fs.writeFileSync(saved, bytes);
    return { file, sha256: hash(bytes) };
  });
  const reference = {
    source: referenceSource,
    physicalSource: [
      {
        filename: "npm/oxlint/source.mjs",
        sha256: hash(fs.readFileSync(path.join(referenceRoot, "source.mjs"))),
      },
    ],
    build: { args: ["pack"], cwd: referenceRoot },
    bundles,
    limits: ["controlled process-law reference only"],
  };
  fs.writeFileSync(path.join(output, "reference-source.json"), JSON.stringify(reference));
  return {
    artifacts,
    binary,
    receipt,
    replay: {
      output,
      custody,
      inventory: { controlled: "whole fixture descriptor retained" },
      hosts: [],
      hostCustodies: [],
      reference: {
        ...reference,
        plugin: path.join(referenceRoot, "dist/index.mjs"),
        wrapper: path.join(referenceRoot, "dist/cli.mjs"),
      },
    },
  };
}
