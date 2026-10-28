import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { mutatedAdmission } from "../../../tests/tooling/support/type-admission-fixtures.ts";
import { scaleGraph } from "./type-admission-scale-graph.ts";
import { scaleSfc } from "./type-admission-scale-sfc.ts";
import {
  scaleTypes,
  validateScale,
  type ScaleOptions,
  type ScaleProbe,
} from "./type-admission-scale-types.ts";

export type ScaleDiagnostic = { file: string; line: number; column: number; code: number };
export const scaleDefaults: ScaleOptions = {
  sfcs: 2,
  sfcBytes: 8192,
  graphNodes: 8,
  shape: "diamond",
  typeWidth: 4,
  typeDepth: 4,
  stage: "valid",
};

/** Creates a new, original project. An existing directory is never a cold input. */
export function prepareAdmissionScale(project: string, provider: string, options: ScaleOptions) {
  validateScale(options);
  fs.mkdirSync(project);
  fs.symlinkSync(
    path.resolve(provider, "node_modules"),
    path.join(project, "node_modules"),
    process.platform === "win32" ? "junction" : "dir",
  );
  const expected: ScaleDiagnostic[] = [];
  const manifest: { file: string; bytes: number; sha256: string; lines: number }[] = [];
  const sfcs: { file: string; rows: number; bytes: number; lineEndings: string }[] = [];
  const write = (file: string, source: string, probes: ScaleProbe[] = []) => {
    for (const probe of probes) {
      if (options.stage === "valid") continue;
      const mutation = mutatedAdmission(source, { ...probe, name: file });
      source = mutation.source;
      expected.push({ file, line: mutation.line, column: mutation.column, code: mutation.code });
    }
    const absolute = path.join(project, file);
    fs.mkdirSync(path.dirname(absolute), { recursive: true });
    fs.writeFileSync(absolute, source);
    const entry = {
      file,
      bytes: Buffer.byteLength(source),
      sha256: createHash("sha256").update(source).digest("hex"),
      lines: source.split("\n").length,
    };
    manifest.push(entry);
    return entry;
  };
  write("types.ts", scaleTypes(options.typeWidth, options.typeDepth));
  const graph = scaleGraph(options);
  for (const [file, source] of Object.entries(graph.files)) {
    const probes = graph.probes
      .filter((probe) => probe.file === file)
      .map((probe) =>
        options.stage === "poisoned"
          ? {
              file,
              before: "type ProofLiveness = Assert<true>;",
              after: "type ProofLiveness = Assert<false>;",
              token: "false",
              code: 2344,
            }
          : probe,
      );
    write(file, source, probes);
  }
  for (let index = 0; index < options.sfcs; index++) {
    const sfc = scaleSfc(index, options);
    const written = write(sfc.file, sfc.source, sfc.probes);
    if (written.bytes < options.sfcBytes)
      throw new Error("mutation reduced the actual SFC below its requested size");
    sfcs.push({
      file: sfc.file,
      rows: sfc.rows,
      bytes: written.bytes,
      lineEndings: sfc.lineEndings,
    });
  }
  const config = {
    compilerOptions: {
      strict: true,
      noEmit: true,
      skipLibCheck: true,
      module: "ESNext",
      target: "ES2024",
      moduleResolution: "Bundler",
      lib: ["ES2024", "DOM"],
      types: [],
      incremental: false,
      paths: { "@admission/node-*": ["./packages/node-*/barrel.ts"] },
    },
    include: ["src/**/*.vue", "packages/**/*.ts", "types.ts"],
  };
  write("tsconfig.json", JSON.stringify(config, null, 2));
  write(
    "tsconfig.mapper.json",
    JSON.stringify(
      {
        ...config,
        contentMappers: [{ package: "vize", extensions: [".vue"] }],
      },
      null,
      2,
    ),
  );
  const corpus = {
    originalPublicFixture: true,
    options,
    sfcs,
    expected,
    manifest,
    sourceBytes: manifest.reduce((sum, entry) => sum + entry.bytes, 0),
    cold: "fresh tool process and original project; no persisted typechecker/mapper state; OS page cache not flushed",
  };
  fs.writeFileSync(path.join(project, "corpus.json"), JSON.stringify(corpus, null, 2));
  return corpus;
}
