import assert from "node:assert/strict";
import { jsonLines } from "./files.ts";
export interface CargoArtifact {
  reason: string;
  target: { name: string; kind: string[]; src_path: string };
  executable: string;
  fresh: boolean;
  features: string[];
  manifest_path: string;
  package_id: string;
  profile: { opt_level: string; debuginfo: number; test: boolean };
}
export function cargoArtifacts(log: string) {
  return jsonLines<CargoArtifact>(log).filter((row) => row.reason === "compiler-artifact");
}
export function executableArtifact(log: string, target: string, kind?: string) {
  const artifacts = cargoArtifacts(log).filter(
    (row) =>
      row.target.name === target && row.executable && (!kind || row.target.kind.includes(kind)),
  );
  assert.equal(artifacts.length, 1, target);
  return artifacts[0]!;
}
