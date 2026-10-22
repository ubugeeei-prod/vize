import { composed } from "./cross-extra.ts";
import { exampleLinks } from "./example-links.ts";
import { crossMetadata } from "./project-metadata.ts";
import { routerExamples } from "./router-project.ts";

export function projectIndex(root: string, ja: boolean, inline = false) {
  const label = (en: string, jp: string) => (ja ? jp : en);
  const entries = [
    ...Object.keys({ ...routerExamples, ...composed }).map((id) => ({ id, status: "cli" })),
    ...crossMetadata(root).map(({ code: id, status }) => ({ id, status })),
  ];
  return [
    "",
    ...(ja ? ['<span id="プロジェクト-ルールと-analyzer-契約-66"></span>', ""] : []),
    `## ${label("Project rules and analyzer contracts", "プロジェクトのルールと解析器の診断契約")} (${entries.length})`,
    "",
    label(
      "These project entries supplement the 266 single-file catalog entries above. Each example includes its component/project context or an explicit tracked-graph scenario. CLI, experimental library producers, and contracts without a producer have different support boundaries; see the [cross-file overview](./cross-file.md).",
      "以下の項目は、上の 266 件の単一ファイルのルールを補足します。それぞれの例には、コンポーネントやプロジェクトの構成、または参照を追跡するグラフの具体的な条件が含まれます。CLI の検査、実験的なライブラリーの生成元、生成元のない診断契約では対応範囲が異なります。[ファイル間検査の概要](./cross-file.md)を確認してください。",
    ),
    "",
    `| ${label("Rule / code", "ルール / コード")} | ${label("Examples", "例")} | ${label("Current support", "現在の対応")} |`,
    "| --- | --- | --- |",
    ...entries.map(({ id, status }) => {
      const slug = id.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
      const path = inline ? `#${slug}` : `./project/${slug}.md`;
      const support =
        status === "cli"
          ? id.endsWith("circular-reactive-dependency")
            ? label("CLI: tracked-graph scenario", "CLI: 参照を追跡するグラフの例")
            : label("CLI project pass", "CLI のプロジェクト検査")
          : status === "library"
            ? id.endsWith("async-no-suspense")
              ? label(
                  "Library producer; required source fact missing",
                  "ライブラリーの生成元あり。必要なソース情報が未対応",
                )
              : label(
                  "Experimental library producer; not this CLI code",
                  "実験的なライブラリーの生成元。CLI の個別コードでは未生成",
                )
            : label("Contract only; no producer", "契約のみ。生成元なし");
      return `| [\`${id}\`](${path}) | ${exampleLinks(inline ? "" : path, ja, inline ? slug : undefined)} | ${support} |`;
    }),
  ];
}
