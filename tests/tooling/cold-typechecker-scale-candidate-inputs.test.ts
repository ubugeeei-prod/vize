import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";
import {
  verifyCandidateInputSeals,
  type CandidateInputSeal,
  materializedCandidateHelpers,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-inputs.ts";

test("native helper body has an exact source-defined printer LF and optional JSX prefix, no arbitrary drift", () => {
  const before = Buffer.from("whole shared before\n"),
    after = Buffer.from("whole shared after\n");
  const debug = {
    before: Buffer.concat([before, Buffer.from("\n")]),
    after: Buffer.concat([after, Buffer.from("\n")]),
  };
  const saved = [Buffer.from(debug.before), Buffer.from(debug.after)];
  for (const prefix of [Buffer.alloc(0), Buffer.from('/// <reference types="vue/jsx" />\n')]) {
    const body = Buffer.concat([prefix, before]);
    const result = materializedCandidateHelpers(debug, { bytes: body.length, sha256: hash(body) });
    assert(result.before.equals(body));
    assert(result.after.equals(Buffer.concat([prefix, after])));
  }
  assert(debug.before.equals(saved[0]) && debug.after.equals(saved[1]));
  for (const body of [
    debug.before,
    before.subarray(0, -1),
    Buffer.concat([Buffer.from("unowned prefix\n"), before]),
  ])
    assert.throws(() =>
      materializedCandidateHelpers(debug, { bytes: body.length, sha256: hash(body) }),
    );
});

const hash = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
function seal(rows: Record<string, unknown>[]): CandidateInputSeal {
  return {
    roots: ["/owned/project", "/owned/provider", "/owned/materialized"],
    rows,
    sha256: hash(JSON.stringify(rows)),
    files: rows.filter((row) => row.kind === "file").length,
    bytes: rows
      .filter((row) => row.kind === "file")
      .reduce((sum, row) => sum + Number(row.bytes), 0),
  };
}
function fixture() {
  const helpers = {
    before: Buffer.from("whole old preamble"),
    after: Buffer.from("whole repaired preamble"),
  };
  const file = (name: string, body: Buffer) => ({
    path: name,
    kind: "file",
    bytes: body.length,
    sha256: hash(body),
    mode: 33188,
    mtimeNs: "1",
  });
  const rows: Record<string, unknown>[] = [
    file("/owned/project/authored.vue", Buffer.from("authored")),
    {
      path: "/owned/project/node_modules",
      kind: "symlink",
      target: "../provider/node_modules",
      physical: "/owned/provider/node_modules",
    },
    file("/owned/provider/manifest", Buffer.from("provider")),
    { path: "/owned/materialized", kind: "directory", mode: 16877 },
    file("/owned/materialized/__vize_helpers.d.ts", helpers.before),
    file("/owned/materialized/A.vue.ts", Buffer.from("generated")),
    file("/owned/materialized/tsconfig.json", Buffer.from("whole config")),
  ];
  const afterRows = structuredClone(rows);
  afterRows[4] = file("/owned/materialized/__vize_helpers.d.ts", helpers.after);
  afterRows
    .filter((row) => row.kind === "file" && String(row.path).startsWith("/owned/materialized/"))
    .forEach((row) => {
      row.mtimeNs = "2";
    });
  return { before: seal(rows), after: seal(afterRows), helpers };
}

test("whole candidate input seal permits only bound helper body and recreated materialized mtimes", () => {
  const x = fixture(),
    saved = JSON.stringify(x);
  const result = verifyCandidateInputSeals(
    x.before,
    x.after,
    "/owned/materialized",
    x.helpers,
    true,
  );
  assert.equal(result.changedHelperPath, "/owned/materialized/__vize_helpers.d.ts");
  assert.equal(result.rows, 7);
  assert.equal(JSON.stringify(x), saved);
});

test("candidate input seal refuses config/source/provider/link/mode/unknown/extra/drop/order drift", () => {
  for (const mutate of [
    (rows: Record<string, unknown>[]) => {
      rows[0].mtimeNs = "2";
    },
    (rows: Record<string, unknown>[]) => {
      rows[1].target = "../different";
    },
    (rows: Record<string, unknown>[]) => {
      rows[2].sha256 = "a".repeat(64);
    },
    (rows: Record<string, unknown>[]) => {
      rows[3].mode = 16832;
    },
    (rows: Record<string, unknown>[]) => {
      rows[4].opaque = "changed";
    },
    (rows: Record<string, unknown>[]) => {
      rows[5].sha256 = "a".repeat(64);
    },
    (rows: Record<string, unknown>[]) => {
      rows[6].sha256 = "a".repeat(64);
    },
    (rows: Record<string, unknown>[]) => {
      rows.push({ ...rows[5], path: "/owned/materialized/extra.ts" });
    },
    (rows: Record<string, unknown>[]) => {
      rows.pop();
    },
    (rows: Record<string, unknown>[]) => {
      rows.reverse();
    },
    (rows: Record<string, unknown>[]) => {
      rows[4].path = "/owned/materialized/other/__vize_helpers.d.ts";
    },
  ]) {
    const x = fixture();
    mutate(x.after.rows);
    x.after = seal(x.after.rows);
    assert.throws(() =>
      verifyCandidateInputSeals(x.before, x.after, "/owned/materialized", x.helpers, true),
    );
  }
});

test("candidate input seal refuses stale/tampered hash, missing helper and a wrong whole debug body", () => {
  const x = fixture();
  x.after.sha256 = "0".repeat(64);
  assert.throws(() =>
    verifyCandidateInputSeals(x.before, x.after, "/owned/materialized", x.helpers, true),
  );
  const y = fixture();
  y.helpers.before = Buffer.from("another helper");
  assert.throws(() =>
    verifyCandidateInputSeals(y.before, y.after, "/owned/materialized", y.helpers, true),
  );
  const z = fixture();
  z.before = seal(z.before.rows.filter((row) => !String(row.path).endsWith("__vize_helpers.d.ts")));
  assert.throws(() =>
    verifyCandidateInputSeals(z.before, z.after, "/owned/materialized", z.helpers, true),
  );
  const plain = fixture();
  assert.throws(() =>
    verifyCandidateInputSeals(plain.before, plain.after, "/owned/materialized", null, false),
  );
  assert.doesNotThrow(() =>
    verifyCandidateInputSeals(plain.before, plain.before, "/owned/materialized", null, false),
  );
});

test("four-root attribution keeps the complete tmp root and does not loosen its authored parent", () => {
  const x = fixture();
  for (const s of [x.before, x.after]) {
    s.roots.push("/owned/tmp");
    s.rows.push({ path: "/owned/tmp", kind: "directory", mode: 16877 });
    s.sha256 = hash(JSON.stringify(s.rows));
  }
  assert.doesNotThrow(() =>
    verifyCandidateInputSeals(
      x.before,
      x.after,
      "/owned/materialized",
      x.helpers,
      true,
      "/owned/tmp",
    ),
  );
  const bad = structuredClone(x.after);
  bad.rows[0].mtimeNs = "different-authored-time";
  bad.sha256 = hash(JSON.stringify(bad.rows));
  assert.throws(() =>
    verifyCandidateInputSeals(x.before, bad, "/owned/materialized", x.helpers, true, "/owned/tmp"),
  );
  assert.throws(() =>
    verifyCandidateInputSeals(x.before, x.after, "/owned/materialized", x.helpers, true),
  );
});
