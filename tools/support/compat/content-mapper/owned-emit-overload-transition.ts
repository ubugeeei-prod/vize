/** Certify one source-owned helper replacement before any packet equality shortcut. */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { certifiedTransition } from "./canonical-helper-transition.ts";
import { beforeCanonLeftovers, canonLeftoversSha256 } from "./strict-extra-keys-transition.ts";

export const emitOverloadsPath =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt";
const ownerPath = "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-props.d.ts.txt";
const preamblePath = "crates/vize_canon/src/virtual_ts/helpers/preamble.rs";
const previousOwnerHash = "ede8a64664b4f16e139b909b77ce10fd911fa11164ed5eff53ad07b4bb1d4d30";
const currentOwnerHash = "e54377ca91f2ba56f96d735c2116744e39b7c8c1559a387065e54272c40a8b22";
const previousPreambleHash = "38c8e437524c3983a57d089febbf49ac4898ddfc1318672ded78608728c0dde6";
const currentPreambleHash = "a8ef812b81fda63ef7d2f9104cccffaa5330df2ab5ea4e49360410536f0e4464";
const stableSourceHashes = [
  [
    "crates/vize_canon/src/virtual_ts/generator/emits.rs",
    "17b531f50a473ef1bdadac55d311756d2193bd547cc039f8af961355a05a538a",
  ],
  [
    "crates/vize_canon/tests/constructor_emission_projection.rs",
    "631c2b23b93178857a595f33d391027be961eef54300d7538d4a32493dad046d",
  ],
] as const;
const previousHash = "ef2a99a2e675a810472b7a3cc6c9a4c9a905f12369aa1366dc9605af94b9c6f5";
const historicalCurrentHash = "924bb683376f6c2ab3b9225cd6eff73328f8dfe4e87529a1c1a8068c03923e74";
const currentHash = "e49c7a2b6603230adcfa42ae7a27cd81265c25b5b62b678f374e5c3642e0bbc4";
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
  const baseHash = digest(before);
  const composedHistory = baseHash === previousHash;
  assert.ok(composedHistory || baseHash === historicalCurrentHash, "unknown previous emit helper");
  assert.equal(digest(after), currentHash, "unknown current emit helper");
  const historicalAfter = Buffer.from(beforeCanonLeftovers(emitOverloadsPath, after.toString()));
  assert.equal(digest(historicalAfter), historicalCurrentHash, "unknown historical emit helper");
  const line = (bytes: Buffer) => {
    const lines = bytes
      .toString()
      .split("\n")
      .filter((value) => value.startsWith(marker.toString()));
    assert.equal(lines.length, 1, "emit alias must occur exactly once in its asset");
    return lines[0];
  };
  if (composedHistory) {
    const previous = line(before),
      historicalCurrent = line(historicalAfter);
    assert.deepEqual(
      historicalAfter,
      Buffer.from(before.toString().replace(previous, historicalCurrent)),
      "only the owned historical emit alias may change",
    );
  } else {
    assert.deepEqual(before, historicalAfter, "canon leftovers predecessor emit helper");
  }
  const baseOwner = readFileSync(path.join(baseRoot, ownerPath));
  const headOwner = readFileSync(path.join(headRoot, ownerPath));
  assert.equal(
    digest(baseOwner),
    previousOwnerHash,
    `uncertified emit producer source: ${ownerPath}`,
  );
  assert.equal(
    digest(headOwner),
    currentOwnerHash,
    `uncertified emit producer source: ${ownerPath}`,
  );
  assert.deepEqual(
    Buffer.from(beforeCanonLeftovers(ownerPath, headOwner.toString())),
    baseOwner,
    "canon leftovers predecessor emit owner",
  );
  const assets = [
    [emitOverloadsPath, baseHash, currentHash],
    [ownerPath, previousOwnerHash, currentOwnerHash],
    [
      preamblePath,
      composedHistory ? previousPreambleHash : currentPreambleHash,
      currentPreambleHash,
    ],
    ...stableSourceHashes.map(([file, hash]) => [file, hash, hash] as const),
  ] as const;
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
  const selected = (revision: string, file: string): Buffer =>
    execFileSync("git", ["-C", sources.repository, "show", `${revision}:${file}`], {
      stdio: ["ignore", "pipe", "pipe"],
    });
  const previousPreamble = selected(sources.base, preamblePath);
  const listenerAnchor = '    include_str!("declarations/shared-emit-inference.d.ts.txt"),\n';
  const listenerAddition = '    include_str!("declarations/shared-listener-args.d.ts.txt"),\n';
  const currentPreamble = selected(sources.head, preamblePath);
  if (composedHistory) {
    assert.equal(previousPreamble.toString().split(listenerAnchor).length, 2);
    assert.deepEqual(
      currentPreamble,
      Buffer.from(
        previousPreamble.toString().replace(listenerAnchor, listenerAnchor + listenerAddition),
      ),
      "only the certified shared listener include may change the emit preamble dependency",
    );
  } else {
    assert.deepEqual(previousPreamble, currentPreamble, "emit preamble changed in successor");
  }
  return {
    before,
    after,
    beforeOwner: baseOwner,
    afterOwner: headOwner,
    provenance: {
      base: sources.base,
      head: sources.head,
      successorRecordSha256: canonLeftoversSha256,
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
  const previousBlock = Buffer.concat([certificate.before, certificate.beforeOwner]);
  const currentBlock = Buffer.concat([certificate.after, certificate.afterOwner]);
  const inspect = (text: Buffer) => ({
    previous: offsets(text, certificate.before),
    current: offsets(text, certificate.after),
    previousOwner: offsets(text, certificate.beforeOwner),
    currentOwner: offsets(text, certificate.afterOwner),
    previousBlock: offsets(text, previousBlock),
    currentBlock: offsets(text, currentBlock),
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
      old.previousOwner.length,
      old.previousBlock.length,
      old.currentBlock.length,
      old.aliases.length,
      next.previous.length,
      next.current.length,
      next.currentOwner.length,
      next.previousBlock.length,
      next.currentBlock.length,
      next.aliases.length,
    ],
    [1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1],
    "emit helper inventory must follow its selected source transition",
  );
  return certifiedTransition(
    before,
    after,
    previousBlock.toString(),
    currentBlock.toString(),
    "source-owned emit overload helper and owner",
  );
}
