import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  gitBodyDigest,
  qualifyPathHostMove,
} from "../performance/support/warm-type-backed-path-host.ts";
import {
  pathHostCallers,
  pathHostMoveCallers,
} from "../../tools/support/compat/levels/path-host-callers.ts";
import { withoutPathHostReferences } from "./support/davinci-path-host-imports.ts";

const manifest = JSON.parse(
  fs.readFileSync(
    new URL("../performance/support/warm-type-backed-path-host-manifest.json", import.meta.url),
    "utf8",
  ),
) as { files: Array<[string, string | null, string | null]> };
const entries = new Map(manifest.files.map(([file, before, after]) => [file, { before, after }]));
const production = manifest.files
  .map(([file]) => file)
  .filter((file) => file !== "davinci/vize_l0/src/path.rs");
const digest = (side: "before" | "after", file: string) => entries.get(file)![side];

test("only the complete reviewed sixty-body path move qualifies", () => {
  const result = qualifyPathHostMove(production, new Set(), digest);
  assert.ok(result);
  assert.equal(result.completeBodyPairs, 60);
  assert.deepEqual(result.files, [...entries.keys()]);
  assert.deepEqual(
    qualifyPathHostMove([...production, "davinci/vize_l0/src/path.rs"], new Set(), digest),
    result,
  );
});

test("every original and transformed body and absent owner must match", () => {
  for (const [file] of manifest.files)
    for (const side of ["before", "after"] as const)
      assert.equal(
        qualifyPathHostMove(production, new Set(), (candidate, name) =>
          candidate === side && name === file ? "f".repeat(64) : digest(candidate, name),
        ),
        null,
        `${side}:${file}`,
      );
});

test("unknown and incomplete production footprints remain refused", () => {
  assert.equal(
    qualifyPathHostMove([...production, "crates/vize/src/unreviewed.rs"], new Set(), digest),
    null,
  );
  for (const file of production)
    assert.equal(
      qualifyPathHostMove(
        production.filter((name) => name !== file),
        new Set(),
        digest,
      ),
      null,
      file,
    );
  assert.equal(qualifyPathHostMove([...production, production[0]], new Set(), digest), null);
});

test("existing owned changes grant no arbitrary path-body exemption", () => {
  const owned = new Set(["crates/vize_canon/src/lsp_client.rs"]);
  assert.ok(qualifyPathHostMove([...production, ...owned], owned, digest));
  assert.equal(qualifyPathHostMove([...owned], owned, digest), null);
  assert.equal(
    qualifyPathHostMove([...production, ...owned], owned, (side, file) =>
      file.endsWith("path.rs") ? null : digest(side, file),
    ),
    null,
  );
});

test("historical c45 caller custody permits unrelated current edits but preserves live boundaries", () => {
  const root = path.resolve(import.meta.dirname, "../..");
  const producer = "c45e0901fc05cfa51c50d167ac97a34ec466dd18";
  const current = [
    [
      "crates/vize_canon/src/corsa_bridge/vue_document/build/native_union_tests.rs",
      [0, 1, 0],
      "caa7368b47c94497c8ca518149140681aafaafbb45094524fce6dfc33505764c",
    ],
    [
      "crates/vize_maestro/src/server/native_requests/tests.rs",
      [0, 1, 0],
      "048f5d5074d011e47d08c46ed183a169ee57581603a0bca1ee968af619334023",
    ],
    [
      "crates/vize_maestro/src/server/project_contexts/operation_admission_tests.rs",
      [0, 2, 0],
      "19e041fb9f70de3300646729bcde1d9142e1eeb6abb3b51bc83f587cfd17ae12",
    ],
  ] as const;
  assert.equal(Object.keys(pathHostMoveCallers).length, 56);
  assert.equal(
    createHash("sha256").update(JSON.stringify(pathHostMoveCallers)).digest("hex"),
    "403b313152abd29a6771372df7e5b7128094ff6073757fb54e55fda095914139",
    "all original caller identities and API count tuples remain exact",
  );
  assert.equal(Object.keys(pathHostCallers).length, 59);
  assert.deepEqual(pathHostCallers, {
    ...pathHostMoveCallers,
    ...Object.fromEntries(current.map(([file, counts]) => [file, counts])),
  });
  assert.deepEqual(
    manifest.files
      .map(([file]) => file)
      .filter(
        (file) =>
          ![
            "davinci/vize_l0/src/path.rs",
            "crates/vize_carton/src/path.rs",
            "davinci/vize_l0/src/lib.rs",
            "crates/vize_carton/src/lib.rs",
          ].includes(file),
      ),
    Object.keys(pathHostMoveCallers).toSorted(),
  );
  for (const [file, , expected] of current) {
    assert.equal(gitBodyDigest(root, producer, file), expected, `reviewed producer: ${file}`);
    const historical = execFileSync(
      "git",
      ["--no-replace-objects", "show", `${producer}:${file}`],
      { cwd: root },
    ).toString("utf8");
    const pathReference = /\bvize_carton::path\b/u;
    assert.doesNotMatch(withoutPathHostReferences(historical, file), pathReference);
    const physical = path.join(root, file);
    const stat = fs.lstatSync(physical);
    assert.ok(stat.isFile() && (stat.mode & 0o111) === 0, `regular current source: ${file}`);
    const currentBody = fs.readFileSync(physical, "utf8");
    const laterBody = `${currentBody}\n#[test]\nfn unrelated_source_law() { assert_eq!(1 + 1, 2); }\n`;
    assert.notEqual(
      createHash("sha256").update(laterBody).digest("hex"),
      createHash("sha256").update(currentBody).digest("hex"),
    );
    for (const body of [currentBody, laterBody]) {
      assert.doesNotMatch(withoutPathHostReferences(body, file), pathReference);
      for (const extra of [
        "vize_carton::path::canonicalize_non_verbatim(path);",
        "vize_carton::path::normalize_windows_verbatim_path(path);",
        "use vize_carton::path::*;",
      ])
        assert.match(withoutPathHostReferences(`${body}\n${extra}`, file), pathReference);
      assert.match(withoutPathHostReferences(body, `${file}.other`), pathReference);
    }
    // Historical custody reads only the immutable producer, never today's body.
    assert.equal(gitBodyDigest(root, producer, file), expected);
    assert.equal(qualifyPathHostMove([...production, file], new Set(), digest), null);
    assert.ok(qualifyPathHostMove([...production, file], new Set([file]), digest));
  }
});
