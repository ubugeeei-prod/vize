import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { beforeStaticListenerSource } from "./static-listener-prop-source.ts";
import { beforeGenericEventSpecialization } from "./generic-event-specialization-source.ts";

interface Routes {
  cheap: string[];
  direct: string[];
  subtypeWitness: string;
  inferred: string;
}

interface EmitGateOracle {
  version: 1;
  historical: Routes & {
    producer: string;
    inference: string;
    inferenceSha256: string;
    helpers: string;
    sourceHashes: Record<string, string>;
  };
  current: Routes & { source: string; producer: string; sourceHashes: Record<string, string> };
}

const fixture = "tests/_fixtures/tooling/emit-props-gate-11697.json";
const fixtureSha256 = "cf5afd61920c8a2c85fb680d701570f7d7572c988ff28b77f61608e4d543335d";
const sha256 = (text: string): string => createHash("sha256").update(text).digest("hex");

/** Select the authored successor only for its complete, physical producer image. */
export function emitGateOracle(read: (path: string) => string): EmitGateOracle {
  const raw = read(fixture);
  assert.equal(sha256(raw), fixtureSha256, "immutable emit gate oracle context");
  const oracle = JSON.parse(raw) as EmitGateOracle;
  assert.equal(sha256(oracle.historical.inference), oracle.historical.inferenceSha256);
  for (const [path, expected] of Object.entries(oracle.current.sourceHashes)) {
    assert.equal(
      sha256(
        beforeGenericEventSpecialization(
          read,
          path,
          beforeStaticListenerSource(read, path, read(path)),
        ),
      ),
      expected,
      `selected emit gate producer: ${path}`,
    );
  }
  return oracle;
}
