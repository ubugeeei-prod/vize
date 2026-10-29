import assert from "node:assert/strict";
import path from "node:path";
import { readPinnedArtifact } from "./harness.mjs";

export type DialectCoverage = {
  state: "unknown" | "source-inspected";
  registryRef: null;
  evidence: Array<{
    kind: "source-inspection";
    dialects: string[];
    selector: string;
  }>;
};

type Input = { file: string; path: string; sha256: string };

const dialectOrder = ["vue3-sfc", "pug-template", "html-template", "js", "ts", "jsx", "tsx"];

function inspectSource(dialects: Set<string>, authoredFile: string, source: string) {
  const file = authoredFile.toLowerCase();
  if (file.endsWith(".vue")) {
    dialects.add("vue3-sfc");
    if (
      /<template\b[^>]*\blang\s*=\s*(?:"(?:pug|jade)"|'(?:pug|jade)'|(?:pug|jade)(?:\s|>))/i.test(
        source,
      )
    ) {
      dialects.add("pug-template");
    } else if (/<template(?:\s|>)/i.test(source)) dialects.add("html-template");
    for (const match of source.matchAll(/<script\b([^>]*)>/gi)) {
      const attributes = match[1];
      if (/\blang\s*=\s*["']tsx["']/i.test(attributes)) dialects.add("tsx");
      else if (/\blang\s*=\s*["']ts["']/i.test(attributes)) dialects.add("ts");
      else if (/\blang\s*=\s*["']jsx["']/i.test(attributes)) dialects.add("jsx");
      else dialects.add("js");
    }
  } else if (file.endsWith(".tsx")) dialects.add("tsx");
  else if (file.endsWith(".jsx")) dialects.add("jsx");
  else if (file.endsWith(".d.ts") || file.endsWith(".ts")) dialects.add("ts");
  else if (file.endsWith(".js")) dialects.add("js");
}

function assertRelativePath(file: string) {
  assert(file.length > 0 && !path.isAbsolute(file));
  assert(file.split(/[\\/]/).every((part) => part && part !== "." && part !== ".."));
}

export function typecheckerDialectCoverage(root: string, inputs: Input[]): DialectCoverage {
  const dialects = new Set<string>();
  for (const input of inputs) {
    const source = readPinnedArtifact(root, input).toString("utf8");
    if (input.file.toLowerCase() !== "project.json") {
      inspectSource(dialects, input.file, source);
      continue;
    }

    const project = JSON.parse(source) as Record<string, unknown>;
    assert.equal(project.schema, "vize.typechecker.nuxt-plugin-injection-types");
    assert.equal(project.version, 1);
    assert(
      project.files && typeof project.files === "object" && !Array.isArray(project.files),
      "authored project files are required",
    );
    const files = Object.entries(project.files as Record<string, unknown>);
    assert(files.length > 0, "authored project files cannot be empty");
    for (const [file, virtualSource] of files) {
      assertRelativePath(file);
      assert.equal(typeof virtualSource, "string", `${file}: authored source is required`);
      inspectSource(dialects, file, virtualSource as string);
    }
  }
  const ordered = dialectOrder.filter((dialect) => dialects.has(dialect));
  if (ordered.length === 0) return { state: "unknown", registryRef: null, evidence: [] };
  return {
    state: "source-inspected",
    registryRef: null,
    evidence: [
      {
        kind: "source-inspection",
        dialects: ordered,
        selector: "batch-typechecker-fixture-v1 pinned authored sources",
      },
    ],
  };
}
