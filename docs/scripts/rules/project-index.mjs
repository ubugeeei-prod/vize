import { composed } from "./cross-extra.mjs";
import { exampleLinks } from "./example-links.mjs";
import { crossMetadata } from "./project-metadata.mjs";
import { routerExamples } from "./router-project.mjs";

export function projectIndex(root, ja) {
  const label = (en, jp) => (ja ? jp : en);
  const entries = [
    ...Object.keys({ ...routerExamples, ...composed }).map((id) => ({ id, status: "cli" })),
    ...crossMetadata(root).map(({ code: id, status }) => ({ id, status })),
  ];
  return [
    "",
    `## ${label("Project rules and analyzer contracts", "プロジェクト ルールと analyzer 契約")} (${entries.length})`,
    "",
    label(
      "These project entries supplement the 251 single-file catalog entries above. Each example includes its component/project context or an explicit tracked-graph scenario. CLI, experimental library producers, and contracts without a producer have different support boundaries; see the [cross-file overview](./cross-file.md).",
      "以下は上の 251 件の単一ファイル カタログに加わる項目です。コンポーネントやプロジェクトの文脈、または追跡する graph の例を示します。CLI、実験的な library の生成元、生成元のない契約では対応範囲が異なります。[ファイル間検査の概要](./cross-file.md)を確認してください。",
    ),
    "",
    `| ${label("Rule / code", "ルール / コード")} | ${label("Examples", "例")} | ${label("Current support", "現在の対応")} |`,
    "| --- | --- | --- |",
    ...entries.map(({ id, status }) => {
      const path = `./project/${id.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase()}.md`;
      const support =
        status === "cli"
          ? id.endsWith("circular-reactive-dependency")
            ? label("CLI: tracked-graph scenario", "CLI: 追跡 graph の例")
            : label("CLI project pass", "CLI のプロジェクト検査")
          : status === "library"
            ? id.endsWith("async-no-suspense")
              ? label(
                  "Library producer; required source fact missing",
                  "library の生成元あり。必要な source 情報が未対応",
                )
              : label(
                  "Experimental library producer; not this CLI code",
                  "実験的な library の生成元。CLI の個別コードでは未生成",
                )
            : label("Contract only; no producer", "契約のみ。生成元なし");
      return `| [\`${id}\`](${path}) | ${exampleLinks(path, ja)} | ${support} |`;
    }),
  ];
}
