import fs from "node:fs";
import path from "node:path";

export type ProjectionDigest = {
  diagnosticsSha256: string;
  mappingsSha256: string;
};

export type ProjectionFixture = {
  anchors: string[];
  contentMapperAnchors?: string[];
  coverage: string[];
  file: string;
  id: string;
  legacyVue2: boolean;
  lineEnding: "lf" | "crlf";
  optionsApi: boolean;
};

export type ProjectionMatrix = {
  claim: string;
  fixtures: ProjectionFixture[];
  normalization: string[];
  requiredCoverage: string[];
  schemaVersion: number;
  unproven: string[];
};

export function loadProjectionMatrix(root: string): ProjectionMatrix {
  const file = path.join(root, "tests/_fixtures/davinci-ts40-projection/matrix.json");
  return JSON.parse(fs.readFileSync(file, "utf8")) as ProjectionMatrix;
}

export function validateProjectionMatrix(root: string, matrix: ProjectionMatrix): void {
  assert(matrix.schemaVersion === 1, "TS-40 schemaVersion must be 1");
  assert(
    matrix.claim === "current-canon-maestro-behavior-only",
    "TS-40 must not claim Davinci or S2 parity",
  );
  assert(matrix.fixtures.length > 0, "TS-40 fixture matrix must not be empty");
  assert(matrix.normalization.length > 0, "TS-40 normalization policy must be explicit");
  assert(matrix.unproven.length > 0, "TS-40 unproven scope must be explicit");

  const ids = new Set<string>();
  const coverage = new Set<string>();
  for (const fixture of matrix.fixtures) {
    assert(!ids.has(fixture.id), `duplicate TS-40 fixture id: ${fixture.id}`);
    ids.add(fixture.id);
    assert(!path.isAbsolute(fixture.file), `${fixture.id} path must be repository-relative`);
    assert(fixture.anchors.length > 0, `${fixture.id} must have authored mapping anchors`);
    assert(fixture.coverage.length > 0, `${fixture.id} must declare coverage`);
    fixture.coverage.forEach((item) => coverage.add(item));

    const sourcePath = path.join(root, fixture.file);
    assert(fs.existsSync(sourcePath), `${fixture.id} source fixture is missing: ${fixture.file}`);
    assert(fs.statSync(sourcePath).isFile(), `${fixture.id} source fixture is not a file`);
    const source = fs.readFileSync(sourcePath, "utf8");
    assert(source.length > 0, `${fixture.id} source fixture must not be empty`);
    for (const anchor of [...fixture.anchors, ...(fixture.contentMapperAnchors ?? [])]) {
      assert(source.includes(anchor), `${fixture.id} is missing anchor ${JSON.stringify(anchor)}`);
    }
    if (fixture.legacyVue2) {
      assert(fixture.coverage.includes("vue2"), `${fixture.id} legacy mode must cover vue2`);
    }
  }

  for (const required of matrix.requiredCoverage) {
    assert(coverage.has(required), `TS-40 matrix is missing required coverage: ${required}`);
  }
}

export function verifyProjectionDigest(expected: ProjectionDigest, actual: ProjectionDigest): void {
  if (expected.mappingsSha256 !== actual.mappingsSha256) {
    throw new Error("TS-40 mapping drift");
  }
  if (expected.diagnosticsSha256 !== actual.diagnosticsSha256) {
    throw new Error("TS-40 diagnostic drift");
  }
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}
