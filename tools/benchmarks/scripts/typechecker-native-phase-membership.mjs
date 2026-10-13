/** Exact native checker identities; module membership and diagnostics stay unchanged. */
import assert from "node:assert/strict";
import { relative } from "node:path";

function memberNames(shards) {
  return shards.flatMap((shard) => shard.members.map((member) => relative(shard.cwd, member.path)));
}

export function nativeVueCheckName(file, shards) {
  assert(file.endsWith(".vue"), `unknown authored Vue source: ${file}`);
  const candidates = [`${file}.ts`, `${file}.__vize_javascript__.ts`];
  const owned = memberNames(shards).filter(
    (name) => name.startsWith(`${file}.`) && /\.tsx?$/u.test(name),
  );
  assert(
    owned.every((name) => candidates.includes(name)),
    `unknown native Vue checker identity: ${file}`,
  );
  const identities = [...new Set(owned)];
  assert.equal(identities.length, 1, `missing or ambiguous native Vue checker identity: ${file}`);
  return identities[0];
}

export function assertSavedNativeProjection(file, shards) {
  const shared = file.file === "__vize_helpers.d.ts";
  assert(shared || file.file.endsWith(".vue.virtual.ts"), `unknown saved output: ${file.file}`);
  const nativeName = shared
    ? file.file
    : nativeVueCheckName(file.file.replace(/\.virtual\.ts$/u, ""), shards);
  const matches = shards.flatMap((shard) =>
    shard.members.filter((member) => relative(shard.cwd, member.path) === nativeName),
  );
  assert(matches.length > 0, `saved output absent from native program: ${file.file}`);
  assert(
    matches.every((member) => member.sha256 === file.sha256 && member.bytes === file.bytes),
    `saved output differs from native program bytes: ${file.file}`,
  );
}

export function nativeVueSources(shards) {
  return new Set(
    memberNames(shards)
      .filter((name) => /\.vue(?:\.__vize_javascript__)?\.ts$/u.test(name))
      .map((name) => name.replace(/(?:\.__vize_javascript__)?\.ts$/u, "")),
  );
}
