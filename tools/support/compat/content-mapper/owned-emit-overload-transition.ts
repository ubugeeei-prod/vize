/** Certify one source-owned helper replacement before any packet equality shortcut. */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { certifiedTransition } from "./canonical-helper-transition.ts";

export const emitOverloadsPath =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt";
const ownerPath = "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-props.d.ts.txt";
const sourceHashes = [
  [ownerPath, "ede8a64664b4f16e139b909b77ce10fd911fa11164ed5eff53ad07b4bb1d4d30"],
  [
    "crates/vize_canon/src/virtual_ts/helpers/preamble.rs",
    "38c8e437524c3983a57d089febbf49ac4898ddfc1318672ded78608728c0dde6",
  ],
  [
    "crates/vize_canon/src/virtual_ts/generator/emits.rs",
    "4f43fdfe3c849f4018c301883bcdff070c6cc32edaeb98c37345d904372bf889",
  ],
  [
    "crates/vize_canon/tests/constructor_emission_projection.rs",
    "631c2b23b93178857a595f33d391027be961eef54300d7538d4a32493dad046d",
  ],
] as const;
const previousHash = "ef2a99a2e675a810472b7a3cc6c9a4c9a905f12369aa1366dc9605af94b9c6f5";
const currentHash = "924bb683376f6c2ab3b9225cd6eff73328f8dfe4e87529a1c1a8068c03923e74";
const marker = Buffer.from("type $OverloadUnionRecursive<");
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
export type SourcePair = { repository: string; base: string; head: string };

export function certifyEmitOverloadAssets(
  baseRoot: string,
  headRoot: string,
  sources?: SourcePair,
) {
  const before = readFileSync(path.join(baseRoot, emitOverloadsPath));
  const after = readFileSync(path.join(headRoot, emitOverloadsPath));
  if (before.equals(after) && !sources) return;
  assert.ok(sources, "changed emit helper requires selected source revisions");
  for (const revision of [sources.base, sources.head])
    assert.match(revision, /^[a-f0-9]{40}$/, "emit helper source must be a full commit SHA");
  for (const [revision, physical] of [
    [sources.base, before],
    [sources.head, after],
  ] as const) {
    const selected: Buffer = execFileSync(
      "git",
      ["-C", sources.repository, "show", `${revision}:${emitOverloadsPath}`],
      { stdio: ["ignore", "pipe", "pipe"] },
    );
    assert.deepEqual(physical, selected, "uncertified emit producer source image");
  }
  if (before.equals(after)) return;
  assert.equal(before.length, 1883, "unknown previous emit helper");
  assert.equal(after.length, 2535, "unknown current emit helper");
  assert.equal(digest(before), previousHash, "unknown previous emit helper");
  assert.equal(digest(after), currentHash, "unknown current emit helper");
  const line = (bytes: Buffer) => {
    const lines = bytes
      .toString()
      .split("\n")
      .filter((value) => value.startsWith(marker.toString()));
    assert.equal(lines.length, 1, "emit alias must occur exactly once in its asset");
    return lines[0];
  };
  const previous = line(before),
    current = line(after);
  assert.deepEqual(
    after,
    Buffer.from(before.toString().replace(previous, current)),
    "only the owned emit alias may change",
  );
  const assets = [
    [emitOverloadsPath, previousHash, currentHash],
    ...sourceHashes.map(([file, hash]) => [file, hash, hash]),
  ];
  for (const [file, baseHash, headHash] of assets) {
    for (const [root, revision, expectedHash] of [
      [baseRoot, sources.base, baseHash],
      [headRoot, sources.head, headHash],
    ]) {
      const source: Buffer = execFileSync(
        "git",
        ["-C", sources.repository, "show", `${revision}:${file}`],
        { stdio: ["ignore", "pipe", "pipe"] },
      );
      assert.equal(digest(source), expectedHash, `uncertified emit producer source: ${file}`);
      assert.deepEqual(
        readFileSync(path.join(root, file)),
        source,
        `emit producer image differs from selected source: ${file}`,
      );
    }
  }
  const owner = readFileSync(path.join(headRoot, ownerPath));
  assert.equal(owner.length, 242);
  return {
    before,
    after,
    owner,
    provenance: {
      base: sources.base,
      head: sources.head,
      assets: assets.map(([file, baseSha256, headSha256]) => ({ file, baseSha256, headSha256 })),
    },
  };
}
export type EmitOverloadCertificate = ReturnType<typeof certifyEmitOverloadAssets>;

function offsets(text: Buffer, needle: Buffer) {
  const found: number[] = [];
  for (let at = text.indexOf(needle); at >= 0; at = text.indexOf(needle, at + 1)) found.push(at);
  return found;
}

export function certifyEmitOverloadInventory(
  before: Buffer | undefined,
  after: Buffer | undefined,
  certificate: EmitOverloadCertificate,
) {
  if (!certificate) return;
  if (before === undefined || after === undefined) {
    assert.equal(before, after, "emit projection presence changed");
    return;
  }
  const inspect = (text: Buffer) => ({
    previous: offsets(text, certificate.before),
    current: offsets(text, certificate.after),
    owner: offsets(text, certificate.owner),
    aliases: offsets(text, marker),
  });
  const old = inspect(before),
    next = inspect(after);
  if ([...Object.values(old), ...Object.values(next)].every((values) => values.length === 0))
    return;
  assert.deepEqual(
    [
      old.previous.length,
      old.current.length,
      old.owner.length,
      old.aliases.length,
      next.previous.length,
      next.current.length,
      next.owner.length,
      next.aliases.length,
    ],
    [1, 0, 1, 1, 0, 1, 1, 1],
    "emit helper inventory must follow its selected source transition",
  );
  assert.equal(
    old.owner[0],
    old.previous[0] + certificate.before.length,
    "previous emit helper must be adjacent to its producer owner",
  );
  assert.equal(
    next.owner[0],
    next.current[0] + certificate.after.length,
    "current emit helper must be adjacent to its producer owner",
  );
  return certifiedTransition(
    before,
    after,
    certificate.before.toString(),
    certificate.after.toString(),
    "source-owned emit overload helper",
  );
}
