import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  canonicalPaths,
  nativeAttributesPath,
} from "../../tools/support/compat/content-mapper/constructor-projection-parity.ts";
import { beforeCanonLeftovers } from "../../tools/support/compat/content-mapper/strict-extra-keys-transition.ts";
import { beforeSharedKebabCaseSource } from "../../tools/support/compat/content-mapper/shared-kebab-case-transition.ts";

export const root = fileURLToPath(new URL("../..", import.meta.url));
export const emitPath =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt";
export const ownerPath =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-props.d.ts.txt";
const preamblePath = "crates/vize_canon/src/virtual_ts/helpers/preamble.rs";
export const producerPaths = [
  preamblePath,
  "crates/vize_canon/src/virtual_ts/generator/emits.rs",
  "crates/vize_canon/tests/constructor_emission_projection.rs",
];
export function sourcePair() {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-emit-transition-"));
  const base = path.join(directory, "base"),
    head = path.join(directory, "head");
  const before = readFileSync(
    path.join(root, "tests/tooling/fixtures/emit-overload-transition/before.d.ts.txt"),
  );
  const after = readFileSync(path.join(root, emitPath));
  const afterOwner = readFileSync(path.join(root, ownerPath));
  const beforeOwner = Buffer.from(beforeCanonLeftovers(ownerPath, afterOwner.toString()));
  for (const destination of [base, head]) {
    for (const file of [
      ...canonicalPaths,
      nativeAttributesPath,
      emitPath,
      ownerPath,
      "crates/vize_canon/src/virtual_ts/helpers/declarations/content-mapper-type-imports.d.ts.txt",
      ...producerPaths,
    ]) {
      mkdirSync(path.dirname(path.join(destination, file)), { recursive: true });
      writeFileSync(path.join(destination, file), readFileSync(path.join(root, file)));
    }
    writeFileSync(path.join(destination, emitPath), before);
    writeFileSync(path.join(destination, ownerPath), beforeOwner);
  }
  const currentPreamble = Buffer.from(
    beforeSharedKebabCaseSource(preamblePath, readFileSync(path.join(root, preamblePath), "utf8")),
  );
  const listenerInclude =
    '            include_str!("declarations/shared-listener-args.d.ts.txt"),\n';
  assert.equal(currentPreamble.toString().split(listenerInclude).length, 2);
  const previousPreamble = Buffer.from(currentPreamble.toString().replace(listenerInclude, ""));
  for (const destination of [base, head])
    writeFileSync(path.join(destination, preamblePath), previousPreamble);
  const git = (...args: string[]) =>
    execFileSync(
      "git",
      [
        "-C",
        head,
        "-c",
        "core.autocrlf=false",
        "-c",
        "core.hooksPath=/dev/null",
        "-c",
        "commit.gpgsign=false",
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        ...args,
      ],
      { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    ).trim();
  git("init", "--quiet", "-b", "fixture");
  git("add", ".");
  git("commit", "--quiet", "-m", "test: retain the original helper image");
  const baseSha = git("rev-parse", "HEAD");
  writeFileSync(path.join(head, emitPath), after);
  writeFileSync(path.join(head, ownerPath), afterOwner);
  writeFileSync(path.join(head, preamblePath), currentPreamble);
  git("add", ".");
  git("commit", "--quiet", "-m", "test: retain the current helper image");
  return {
    git,
    directory,
    base,
    head,
    before,
    after,
    beforeOwner,
    afterOwner,
    sources: { repository: head, base: baseSha, head: git("rev-parse", "HEAD") },
  };
}
export function virtualPair(pair: ReturnType<typeof sourcePair>) {
  const source = "<template>日本 🧭</template>";
  const packet = (asset: Buffer) => {
    const prefix = "// 日本 🧭\n";
    const body = "const value = '日本 🧭';\n";
    const tail = "const result = value;\n";
    const owner = asset === pair.before ? pair.beforeOwner : pair.afterOwner;
    const start = Buffer.byteLength(prefix) + asset.length + owner.length;
    const end = start + Buffer.byteLength(body);
    return {
      virtual_ts: prefix + asset.toString() + owner.toString() + body + tail,
      virtual_ts_mappings: [
        {
          authored: { start: 0, end: Buffer.byteLength(source) },
          generated: { start, end },
          subSpans: [
            { authored: { start: 1, end: 2 }, generated: { start: start + 6, end: start + 11 } },
          ],
        },
      ],
      virtual_ts_semantic_links: [
        {
          source: { start: start + 6, end: start + 11 },
          target: { start: end + 6, end: end + 12 },
          kind: "original-kind",
        },
      ],
      diagnostics: [{ start: 0, length: 1, code: 100001, messageText: "original" }],
      opaque: { producer: "unchanged" },
    };
  };
  return { source, base: packet(pair.before), head: packet(pair.after) };
}

/** The observer's target/options and owned/absent population, with Unicode geometry. */
export function projectionRows(pair: ReturnType<typeof sourcePair>) {
  const sample = virtualPair(pair);
  const rows = (selected: "base" | "head") =>
    Array.from({ length: 228 }, (_, index) => {
      const mapper = index < 192;
      const owned = index < 128 || (index >= 192 && index < 216);
      const virtual = owned
        ? structuredClone(sample[selected])
        : {
            virtual_ts: "const absent = 1;",
            virtual_ts_mappings: [],
            virtual_ts_semantic_links: [],
            diagnostics: [],
            opaque: { producer: "unchanged" },
          };
      const packet = mapper
        ? {
            text: virtual.virtual_ts,
            mappings: virtual.virtual_ts_mappings.map(({ authored, generated }) => [
              generated.start,
              generated.end - generated.start,
              authored.start,
              authored.end - authored.start,
              1,
              0,
            ]),
            semanticLinks: virtual.virtual_ts_semantic_links.map(({ source, target, ...rest }) => ({
              ...rest,
              sourceStart: source.start,
              sourceLength: source.end - source.start,
              targetStart: target.start,
              targetLength: target.end - target.start,
            })),
            diagnostics: virtual.diagnostics,
            opaque: virtual.opaque,
          }
        : virtual;
      return {
        case: `case-${index}`,
        newline: "lf",
        source: sample.source,
        target: mapper ? "content-mapper" : "vue3",
        options: mapper ? index % 16 : undefined,
        packet,
      };
    });
  return {
    base: Buffer.from(JSON.stringify(rows("base"))),
    head: Buffer.from(JSON.stringify(rows("head"))),
  };
}
