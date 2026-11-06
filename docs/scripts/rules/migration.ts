import { readFileSync } from "node:fs";
import { resolve } from "node:path";

export function migrationPage(root: string, ruleNames: ReadonlySet<string>, ja: boolean) {
  const inventory = JSON.parse(
    readFileSync(resolve(root, "tests/_fixtures/patina-eslint-vue-rule-map.json"), "utf8"),
  ) as { entries: Record<string, { status: string; patinaRule?: string }> };
  const entries = Object.entries(inventory.entries).sort(([a], [b]) => a.localeCompare(b));
  const counts = new Map<string, number>();
  for (const [, value] of entries) counts.set(value.status, (counts.get(value.status) ?? 0) + 1);
  const label = (en: string, japanese: string) => (ja ? japanese : en);
  const lines = [
    "---",
    `title: ${label("ESLint rule migration map", "ESLint ルール移行対応表")}`,
    "---",
    "",
    `# ${label("ESLint rule migration map", "ESLint ルール移行対応表")}`,
    "",
    label(
      `The committed ESLint mapping records ${entries.length} rule IDs: ${counts.get("mapped")} mapped, ${counts.get("intentional-divergence")} intentional divergences, and ${counts.get("unimplemented")} unimplemented. A mapped ID identifies a Vize rule; it does not guarantee identical options, findings, fixes, or coverage.`,
      `コミットされた対応記録は ${entries.length} 件で、対応付け ${counts.get("mapped")} 件、意図的な差異 ${counts.get("intentional-divergence")} 件、未実装 ${counts.get("unimplemented")} 件です。「対応」はルール名の対応を示し、オプション・検出・修正・適用範囲の完全な一致を保証しません。`,
    ),
    "",
    label(
      "When ESLint remains the host, prefer the [`eslint.config.mjs` flat-config integration](../guide/tool-integrations.md#eslint-flat-config). The Vite+ configuration below is the native-task alternative.",
      "ESLint を実行基盤として残す場合は、[`eslint.config.mjs` の flat config 連携](../guide/tool-integrations.md#eslint-の-flat-config)を優先してください。以下の Vite+ 設定は、ネイティブタスクを使う場合の代替です。",
    ),
    "",
    label(
      "Enable Vize rules under `linter.rules` inside `withVue`. Keep JS/TS rules handled by Oxlint in the root `lint.rules` flat config. Do not copy an ESLint option tuple directly: Vize severity and typed `ruleOptions` are separate.",
      "Vite+ の設定では Vize のルールを `withVue` の `linter.rules` に指定します。Oxlint の JS / TS ルールはルートの `lint.rules` flat config に残します。ESLint の配列形式をそのまま移さず、重大度と型付き `ruleOptions` を分けて指定します。",
    ),
    "",
    '```ts annotate="remove:1,2;add:3,4,5,6"',
    ' import vue from "eslint-plugin-vue";',
    ' export default [{ plugins: { vue }, rules: { "vue/attributes-order": "warn" } }];',
    ' import { defineConfig } from "@vizejs/vite-plugin/vite-plus";',
    " export default defineConfig({}).withVue({",
    '   linter: { rules: { "vue/attribute-order": "warn" } },',
    " });",
    "```",
    "",
    label(
      "Remove only the overlapping Vue rule after checking the linked Bad/Good examples. Run `vp run lint`; if an existing package script occupies that task name, the generated task is `vp run vize:lint`. Explicit task configuration takes precedence.",
      "リンク先の悪い例・良い例を確認してから、重複する Vue ルールだけを取り除きます。`vp run lint` で検査します。既存のパッケージスクリプトと名前が衝突する場合は `vp run vize:lint` です。明示したタスクの設定が優先されます。",
    ),
    "",
    label(
      "`vue/component-definition-name-casing` has a deliberate scope difference: Vize checks the SFC filename, while ESLint checks the component definition name. The mapping below retains this distinction.",
      "`vue/component-definition-name-casing` には意図的な適用範囲の差があります。Vize は SFC のファイル名を検査し、ESLint はコンポーネントの定義名を検査します。この差を以下にも残しています。",
    ),
    "",
    label(
      "The `vue/comment-directive` row tracks the ESLint rule migration, not Vize's opt-in unused-comment reporting. The native check is available through `reportUnusedDisableDirectives`; see [unused disable comments](../guide/lint-execution.md#unused-disable-directives). This bounded check does not establish full ESLint rule parity.",
      "`vue/comment-directive` の行は ESLint ルールの移行状況であり、Vize で明示的に有効化する未使用コメント報告の対応状況ではありません。ネイティブの検査は `reportUnusedDisableDirectives` で有効にできます。[使われていない抑制コメント](../guide/lint-execution.md#unused-disable-directives)を参照してください。この限定した検査は、ESLint ルールとの完全な互換性を保証しません。",
    ),
    "",
    label(
      "For option conversion, see [Rule Options](./options.md) and [props destructuring modes](./reference/script-define-props-destructuring.md). Unsupported rows must stay with an existing checker until a supported replacement is verified.",
      "オプションの変換は[ルールのオプション](./options.md)と [props 分割代入](./reference/script-define-props-destructuring.md)を参照してください。未実装の行は、対応する代替の検査が確認できるまで、既存のチェッカーで検査を続けます。",
    ),
    "",
    label(
      "| ESLint ID | Vize ID / reference | Status |",
      "| ESLint ID | Vize ID / リファレンス | 対応状況 |",
    ),
    "| --- | --- | --- |",
  ];
  for (const [name, entry] of entries) {
    if (entry.patinaRule && !ruleNames.has(entry.patinaRule))
      throw new Error(`Migration target is missing: ${entry.patinaRule}`);
    const target = entry.patinaRule
      ? `[\`${entry.patinaRule}\`](./reference/${entry.patinaRule.replaceAll("/", "-")}.md)`
      : "—";
    const status = ja
      ? (
          {
            mapped: "対応付け",
            "intentional-divergence": "意図的な差異",
            unimplemented: "未実装",
          } as Record<string, string>
        )[entry.status]
      : entry.status;
    lines.push(`| \`${name}\` | ${target} | ${status} |`);
  }
  lines.push(
    "",
    `[${label("Committed mapping and capture identity", "対応記録と取得データの出典")}](https://github.com/ubugeeei-prod/vize/blob/main/tests/_fixtures/patina-eslint-vue-rule-map.json)`,
    "",
  );
  return lines.join("\n");
}
