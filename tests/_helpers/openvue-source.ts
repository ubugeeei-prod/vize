import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const TESTS_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

export interface OpenVueSource {
  schema: "vize.realWorldFixtureSource";
  version: 1;
  id: "openvue";
  repository: string;
  release: string;
  revision: string;
  discussion: string;
  license: { spdx: "MIT"; path: string; gitBlob: string };
  verifiedVueFiles: Record<string, number>;
}

export const OPENVUE_SOURCE = JSON.parse(
  fs.readFileSync(path.join(TESTS_DIR, "openvue-fixture-source.json"), "utf8"),
) as OpenVueSource;

export function verifyOpenVueFixtureSource(): void {
  const fixture = path.join(TESTS_DIR, "_fixtures", "_git", OPENVUE_SOURCE.id);
  assert.equal(
    fs.realpathSync(
      execFileSync("git", ["-C", fixture, "rev-parse", "--show-toplevel"], {
        encoding: "utf8",
      }).trim(),
    ),
    fs.realpathSync(fixture),
    "OpenVue fixture must be an initialized submodule checkout",
  );
  assert.equal(
    execFileSync("git", ["-C", fixture, "rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
    OPENVUE_SOURCE.revision,
    "OpenVue checkout must match the pinned release commit",
  );
  assert.equal(
    execFileSync("git", ["-C", fixture, "hash-object", OPENVUE_SOURCE.license.path], {
      encoding: "utf8",
    }).trim(),
    OPENVUE_SOURCE.license.gitBlob,
    "OpenVue MIT license must match the pinned upstream blob",
  );
}
