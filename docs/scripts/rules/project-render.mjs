import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { original0 } from "./cross-original-0.mjs";
import { original1 } from "./cross-original-1.mjs";
import { original2 } from "./cross-original-2.mjs";
import { extra, composed } from "./cross-extra.mjs";
import { crossMetadata, explanation } from "./project-metadata.mjs";
import { routerExamples } from "./router-project.mjs";
import { projectExplanations } from "./project-explanations.mjs";
import { exampleLinks } from "./example-links.mjs";
import { contractExamples } from "./project-contracts.mjs";
const examples = { ...original0, ...original1, ...original2, ...extra };
const slug = (id) => id.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
export function generateProjectPages(root, checking) {
  const metadata = crossMetadata(root);
  const contracts = metadata
    .filter((rule) => rule.status === "contract")
    .map((rule) => rule.name)
    .sort();
  if (JSON.stringify(contracts) !== JSON.stringify(Object.keys(contractExamples).sort()))
    throw new Error("Contract examples must match all published producer-free contracts");
  for (const locale of ["", "ja/"]) {
    const ja = Boolean(locale);
    const directory = resolve(root, `docs/content/${locale}rules`);
    if (!checking) mkdirSync(resolve(directory, "project"), { recursive: true });
    const label = (en, jp) => (ja ? jp : en);
    const lines = [
      "---",
      `title: ${label("Cross-file rules", "ファイル間ルール")}`,
      "---",
      "",
      `# ${label("Cross-file rules", "ファイル間ルール")}`,
      "",
      label(
        "Project checks need the complete analyzed component graph. Each linked page gives shared files and the exact Bad/Good changes; apply shared files to both examples.",
        "プロジェクトの検査には解析対象のコンポーネント構成が必要です。各ページに共通ファイルと悪い例・良い例を示します。共通ファイルは両方の例で使ってください。",
      ),
      "",
      label(
        "Start with the Vite+ configuration below. `vp run lint` invokes Vize and Oxlint; built-in `vp lint` runs its own upstream checker.",
        "次の Vite+ 設定から始めてください。vp run lint は Vize と Oxlint を実行します。組み込みの vp lint は Vite+ 自身の検査を実行します。",
      ),
      "",
      ...config("cross-file", ja),
      "",
      label(
        "The public CLI exposes the same pass with `vize lint --cross-file`. Displayed `vize:croquis/cf/*` codes use `croquis/cf/*` in `lint.vize.rules` (omit `vize:`). Information/hint diagnostics become CLI warnings. Related locations explain the source/consumer relationship.",
        "CLI では vize lint --cross-file で同じ検査を実行できます。表示コード vize:croquis/cf/* は、lint.vize.rules には vize: を除いた croquis/cf/* として指定します。information / hint は CLI では warning として表示されます。関連位置から提供元と使用側の関係を確認できます。",
      ),
      "",
      label(
        "The 60 published cross-file codes have different support boundaries: 19 belong to the CLI pass (18 complete source pairs and one reactive-graph scenario); 16 have experimental Rust analyzer producers but are not individually emitted by that pass; 25 are published contracts without a current diagnostic producer. Enabling a rule ID does not activate an unavailable producer.",
        "公開されている 60 のコードは対応範囲が異なります。19 は CLI の検査対象（18 の完全なソースの例と 1 つの参照構成の例）で、16 は実験的な Rust analyzer に実装があるものの CLI では個別コードとして生成されません。25 は現在の生成元がない公開契約です。ルール名を設定しても未対応の生成元は有効になりません。",
      ),
      "",
      `## ${label("Project-specific lint IDs", "プロジェクト固有の lint ID")}`,
      "",
      `| ${label("Rule", "ルール")} | ${label("Examples", "例")} | ${label("Severity", "重大度")} |`,
      "| --- | --- | --- |",
    ];
    for (const [id, example] of [...Object.entries(routerExamples), ...Object.entries(composed)]) {
      const severity = example.severity ?? "warning";
      lines.push(
        `| [\`${id}\`](./project/${slug(id)}.md) | ${exampleLinks(`./project/${slug(id)}.md`, ja)} | ${severity} |`,
      );
      output(
        resolve(directory, `project/${slug(id)}.md`),
        projectDetail(id, example, ja, severity),
        checking,
      );
    }
    lines.push(
      "",
      `## ${label("Published analyzer codes", "公開 analyzer コード")}`,
      "",
      `| ${label("Code", "コード")} | ${label("Examples", "例")} | ${label("Status", "対応状況")} |`,
      "| --- | --- | --- |",
    );
    for (const rule of metadata) {
      const status =
        rule.status === "cli"
          ? label("CLI", "CLI")
          : rule.status === "library"
            ? label(
                "Rust analyzer; CLI uses a different surface or disables this pass",
                "Rust analyzer。CLI は別の表示または未有効",
              )
            : label("Contract only; no current producer", "契約のみ。現在の生成元なし");
      lines.push(
        `| [\`${rule.code}\`](./project/${slug(rule.code)}.md) | ${exampleLinks(`./project/${slug(rule.code)}.md`, ja)} | ${status} |`,
      );
      output(
        resolve(directory, `project/${slug(rule.code)}.md`),
        crossDetail(root, rule, ja),
        checking,
      );
    }
    output(resolve(directory, "cross-file.md"), `${lines.join("\n")}\n`, checking);
  }
}
function config(id, _ja) {
  return [
    "```ts",
    'import { defineConfig } from "@vizejs/vite-plugin/vite-plus";',
    "",
    "export default defineConfig({",
    "  lint: {",
    '    vize: { preset: "incremental", crossFile: true,',
    `      rules: { "${id}": "warn" },`,
    "    },",
    "  },",
    "});",
    "```",
    "",
    "```sh",
    "vp run lint",
    "```",
  ];
}
function crossDetail(root, rule, ja) {
  const { code, name, status, severity, producer } = rule;
  const label = (en, jp) => (ja ? jp : en);
  const info = explanation(root, code, ja);
  if (name === "hydration-risk")
    info.purpose = label(
      "This code groups several reactivity findings, including a prop copied into a ref. It does not imply that every Date.now() expression is detected by the cross-file pass.",
      "このコードはprop を ref にコピーする操作など、複数のリアクティビティ検出をまとめています。ファイル間検査がすべての Date.now() 式を検出するという意味ではありません。",
    );
  const lines = [
    "---",
    `title: "${code}"`,
    "---",
    "",
    `# \`${code}\``,
    "",
    info.purpose,
    "",
    `${label("Default severity", "既定の重大度")}: ${status === "contract" ? label("Not emitted", "現在は生成されません") : severity}  `,
    `${label("Applies to", "適用範囲")}: ${label("Analyzed component graph and the supported facts described below", "解析対象のコンポーネント構成と、下記に示す対応済みの情報")}  `,
    `${label("Automatic fix", "自動修正")}: ${label("None; review related files and apply the repair", "なし。関連ファイルを確認して修正してください")}  `,
    `${label("Options", "オプション")}: ${label("No per-code options; supported CLI findings accept severity overrides", "コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます")}`,
    "",
  ];
  if (status === "cli") lines.push(...config(code.slice(5), ja), "");
  else
    lines.push(
      label(
        status === "contract"
          ? "This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger."
          : "The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.",
        status === "contract"
          ? "この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。"
          : "実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。",
      ),
      "",
    );
  if (name === "async-no-suspense")
    lines.push(
      "Current support: `no-source-async-fact`",
      "",
      label(
        "The boundary producer reads macros.is_async(), but source parsing currently records top-level await on the script-setup scope instead. The complete Bad/Good source pair below therefore produces no async-no-suspense finding through the current CLI. It explains the Suspense convention; supplying the missing macro fact is implementation follow-up work.",
        "生成元は macros.is_async() を読みますが、現在のソース解析は top-level await を script-setup の scope に記録します。そのため以下の完全な悪い例・良い例では、現在の CLI は async-no-suspense を検出しません。Suspense の使い方を説明する例で、必要な macro 情報を渡す処理は今後の実装課題です。",
      ),
      "",
    );
  if (name === "provide-inject-type")
    lines.push(
      label(
        "This check compares explicit provider/consumer type annotations, not inferred literal value types. Keep the provider's `as string` annotation in this example.",
        "この検査は提供元と使用側の明示的な型注釈を比較し、リテラルからの型推論は使いません。この例の提供元の as string 注釈を残してください。",
      ),
      "",
    );
  if (name === "uncaught-error")
    lines.push(
      label(
        "The current producer scans template expressions such as JSON.parse(input). It does not report a throw statement that exists only in the script block.",
        "現在の生成元は JSON.parse(input) などのテンプレート式を検査します。script ブロックだけにある throw 文は対象外です。",
      ),
      "",
    );
  const example = examples[name] ?? contractExamples[name];
  const contractNote = contractExamples[name]?.note?.[ja ? "ja" : "en"];
  if (contractNote) lines.push(contractNote, "");
  if (example) lines.push(...fixture({ ...example, ...projectExplanations.get(name) }, ja));
  else {
    if (status !== "contract" && name !== "circular-reactive-dependency")
      throw new Error(`Missing complete project scenario: ${code}`);
    lines.push(
      `## ${label("Bad", "悪い")}`,
      "",
      name === "circular-reactive-dependency"
        ? label(
            "The analyzer's tracked reactive-flow graph contains a cycle: provider A → consumer B → provider A. Both references are the same graph identities, rather than unrelated variables that share a name.",
            "analyzer が追跡するリアクティブな参照の流れが提供元 A → 使用側 B → 提供元 A と循環しています。同名の無関係な変数ではなく、同一の参照として記録された構成です。",
          )
        : info.purpose,
      "",
      ...(name === "circular-reactive-dependency"
        ? [
            "```text",
            "Tracked references: A = provider source; B = consumer reference",
            "Tracked flows: A -> B; B -> A",
            "```",
            "",
          ]
        : []),
      `## ${label("Good", "良い")}`,
      "",
      name === "circular-reactive-dependency"
        ? label(
            "Remove the B → A flow: let A own the source, and let B read a computed value or emit an action instead of feeding that reference back. The tracked flow graph becomes acyclic.",
            "B → A の流れをなくします。元の値は A が管理し、B は computed の読み取りや action の通知を使います。同じ参照を A に戻さなければ、追跡対象の循環がなくなります。",
          )
        : info.help,
      "",
      ...(name === "circular-reactive-dependency"
        ? [
            "```text",
            "Tracked references: A = provider source; B = consumer reference",
            "Tracked flows: A -> B",
            "```",
            "",
          ]
        : []),
    );
  }
  lines.push(
    `[${label("Public explanation", "公開の説明")}](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/${ja ? "ja" : "en"}.txt)`,
    "",
  );
  for (const row of producer)
    lines.push(
      `[${label("Producer", "生成元")}](https://github.com/ubugeeei-prod/vize/blob/main/${row.path})`,
      "",
    );
  lines.push(`[${label("Cross-file index", "ファイル間ルール一覧")}](../cross-file.md)`, "");
  return lines.join("\n");
}
function projectDetail(id, example, ja, severity) {
  const label = (en, jp) => (ja ? jp : en);
  const description = example.note
    ? ja
      ? example.noteJa
      : example.note
    : id.startsWith("html/")
      ? label(
          "Check actual HTML nesting after imported components are composed.",
          "import した子コンポーネントの要素を合成して HTML の入れ子を検査します。",
        )
      : label(
          "A parent passes attributes to a resolved child whose root cannot inherit them and does not explicitly use $attrs.",
          "属性を渡す親と、属性を自動継承できず $attrs も使っていない子コンポーネントの関係を検査します。",
        );
  return [
    "---",
    `title: "${id}"`,
    "---",
    "",
    `# \`${id}\``,
    "",
    description,
    "",
    `${label("Default severity", "既定の重大度")}: ${severity}  `,
    `${label("Applies to", "適用範囲")}: ${label("Reachable project declarations and imported components", "到達可能なプロジェクトの宣言と import したコンポーネント")}  `,
    `${label("Options", "オプション")}: ${label("crossFile; rule severity (off/warn/error)", "crossFile と重大度（off/warn/error）")}  `,
    `${label("Automatic fix", "自動修正")}: ${label("None", "なし")}`,
    "",
    ...(id.startsWith("ecosystem/")
      ? [
          label(
            "The complete installed router must be reachable from the application's createApp(...).use(router). Unknown/dynamic route tables do not prove unknown-name findings. Missing params are warnings because navigation may inherit a value from the current route.",
            "アプリの createApp(...).use(router) から登録済み router に到達できる構成が必要です。未確定または動的な定義では未知の名前と断定しません。省略パラメーターは現在のルートの値を継承する場合があるため warning です。",
          ),
          "",
        ]
      : []),
    ...config(id, ja),
    "",
    ...fixture({ ...example, ...projectExplanations.get(id) }, ja),
    `[${label("Cross-file index", "ファイル間ルール一覧")}](../cross-file.md)`,
    "",
  ].join("\n");
}
function fixture(example, ja) {
  const label = (en, jp) => (ja ? jp : en);
  const lines = [
    `## ${label("Shared project files", "共通のプロジェクト ファイル")}`,
    "",
    label(
      "Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.",
      "以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。",
    ),
    "",
  ];
  const files = (inputs) => {
    for (const [path, source] of Object.entries(inputs))
      lines.push(
        `\`${path}\``,
        "",
        `\`\`\`${path.endsWith(".vue") ? "vue" : path.endsWith(".html") ? "html" : "ts"}`,
        source,
        "```",
        "",
      );
  };
  files(example.shared ?? {});
  for (const [key, title] of [
    ["bad", label("Bad", "悪い")],
    ["good", label("Good", "良い")],
  ]) {
    lines.push(`## ${title}`, "");
    const rationale = example[`${key}Explanation`]?.[ja ? "ja" : "en"];
    if (!rationale) throw new Error(`Missing project example explanation: ${title}`);
    lines.push(rationale, "");
    files(example[key]);
  }
  lines.push(
    label(
      "The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.",
      "良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。",
    ),
    "",
  );
  return lines;
}
function output(path, text, checking) {
  if (checking) {
    if (readFileSync(path, "utf8") !== text) throw new Error(`Stale project reference: ${path}`);
  } else writeFileSync(path, text);
}
