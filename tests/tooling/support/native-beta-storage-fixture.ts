// Synthetic refusal controls only. No native execution or source-capture credit.
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";

export function storageFixture(authored = "/tmp/owned-beta-project") {
  const root = "/tmp/vize-corsa-overlay-Unit123/project";
  const main =
    '/*🧭é日本語\u2028\u2029*/\r\nimport { value } from "./support";\r\nexport const checked: number = value;\r\nexport const echo = value;\r\n';
  const support = "export const value = 1;\r\n";
  const virtual = 'export const bad: number = "x";\r\n';
  const hash = (value: string) => createHash("sha256").update(value).digest("hex");
  const text = JSON.stringify({
    extends: authored + "/tsconfig.json",
    compilerOptions: { allowImportingTsExtensions: true, noEmit: true },
    include: ["**/*"],
    exclude: [],
  });
  return {
    root,
    attached: {
      root: authored,
      mainUri: pathToFileURL(authored + "/main.ts").href,
      supportUri: pathToFileURL(authored + "/support.ts").href,
      rows: ["valid", "dependency-error", "dependency-repair", "dependency-close"].map((phase) => ({
        phase,
        mainText: main,
        supportOverlay:
          phase === "dependency-error"
            ? 'export const value = "oops";\r\n'
            : phase === "dependency-repair"
              ? "export const value = 2;\r\n"
              : support,
      })),
      materializedBatch: {
        virtualUri: pathToFileURL(authored + "/Virtual.vue.ts").href,
        mappedUri: pathToFileURL(root + "/Virtual.vue.ts").href,
        source: virtual,
        storageAuthority: {
          schema: 1,
          scope:
            "source getter and materialized file reads before close; no root inferred from observed RPC",
          authoredRoot: authored,
          clientProjectRoot: authored,
          materializedRoot: root,
          canonicalMaterializedRoot: root,
          rootIsDirectory: true,
          rootIsSymlink: false,
          documents: [
            ["main.ts", main],
            ["support.ts", support],
            ["Virtual.vue.ts", virtual],
          ].map(([relativePath, source]) => ({
            externalUri: pathToFileURL(authored + "/" + relativePath).href,
            mappedUri: pathToFileURL(root + "/" + relativePath).href,
            relativePath,
            bytes: Buffer.byteLength(source!),
            sha256: hash(source!),
          })),
          config: {
            path: root + "/tsconfig.json",
            bytes: Buffer.byteLength(text),
            sha256: hash(text),
            text,
          },
        },
      },
    },
  };
}
