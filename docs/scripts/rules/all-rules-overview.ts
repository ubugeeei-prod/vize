import { categoryFiles, categoryIncludes } from "./catalogue-routes.ts";
import { cataloguePurpose } from "./catalogue-translations.ts";
import { ruleSlug } from "./catalogue-page.ts";
import { vueCategoryLabels, type VueLocale } from "./vue-category-labels.ts";

export const overviewIntroduction: Record<VueLocale, string> = {
  en: "Use this index to find a rule's purpose, default settings, and owning group. The group pages contain the explanations, configuration, complete Bad/Good examples, and current support limits. Existing rule and example links remain usable here.",
  ja: "ルールの目的、既定の設定、所属するカテゴリを探すための一覧です。説明・設定・完全な悪い例と良い例・現在の対応範囲は、各カテゴリのページで確認できます。この一覧の既存のルール・例へのリンクも引き続き使用できます。",
  fr: "Cet index permet de trouver l'objectif d'une règle, ses réglages par défaut et son groupe. Les pages des groupes présentent les explications, la configuration, les exemples complets incorrects et corrects ainsi que les limites actuelles. Les liens existants vers les règles et leurs exemples restent utilisables ici.",
  "pt-BR":
    "Use este índice para encontrar o objetivo de uma regra, as configurações padrão e seu grupo. As páginas dos grupos apresentam as explicações, a configuração, os exemplos completos incorretos e corretos e os limites atuais. Os links existentes para regras e exemplos continuam disponíveis aqui.",
  "zh-CN":
    "通过此索引查找规则的用途、默认配置及所属分组。分组页面包含说明、配置、完整的错误和正确示例，以及当前支持范围。此页面现有的规则和示例链接仍然可用。",
};

export const overviewProjectIntroduction: Record<VueLocale, string> = {
  en: "The 66 project entries link to complete shared files and Bad/Good examples on the cross-file page. The 60 analyzer codes retain their actual boundaries: 19 CLI codes (18 qualified source pairs and one illustrative project with its retained reactive graph), 16 experimental Rust analyzer codes not emitted individually by that CLI pass, and 25 contracts without a current producer. Configuring an ID does not enable an unavailable producer.",
  ja: "66 件のプロジェクトの項目から、ファイル間ルールのページにある共通ファイルと、省略のない悪い例・良い例に移動できます。解析器の 60 件の診断コードには、CLI の 19 件（検証済みのソース例 18 件と、参照関係を保持したリアクティブなグラフを含む説明用プロジェクト 1 件）、CLI が個別には生成しない実験的な Rust 解析器の 16 件、現在は診断を生成する処理のない契約 25 件があります。ID を設定しても、未対応の診断処理は有効になりません。",
  fr: "Les 66 entrées de projet renvoient aux fichiers partagés complets et aux exemples incorrects et corrects de la page des règles inter-fichiers. Les 60 codes d'analyse conservent leurs limites réelles : 19 codes CLI (18 paires de sources validées et un projet illustratif avec son graphe réactif conservé), 16 codes expérimentaux de l'analyseur Rust que cette passe CLI n'émet pas individuellement et 25 contrats sans producteur actuel. Configurer un identifiant n'active pas un producteur indisponible.",
  "pt-BR":
    "As 66 entradas de projeto levam aos arquivos compartilhados completos e aos exemplos incorretos e corretos na página de regras entre arquivos. Os 60 códigos do analisador mantêm seus limites reais: 19 códigos da CLI (18 pares de fontes validados e um projeto ilustrativo com seu grafo reativo preservado), 16 códigos experimentais do analisador Rust que essa etapa da CLI não emite individualmente e 25 contratos sem produtor atual. Configurar um ID não ativa um produtor indisponível.",
  "zh-CN":
    "66 个项目条目链接到跨文件规则页面中的完整共享文件以及错误和正确示例。60 个分析器代码保留其实际支持边界：19 个 CLI 代码（18 对已验证的源代码示例，以及一个保留响应式图的说明性项目）、16 个该 CLI 阶段不会单独输出的实验性 Rust 分析器代码，以及 25 个当前没有生成器的契约。配置 ID 不会启用尚不可用的生成器。",
};

export function owningRuleGroup(id: string, project: boolean) {
  if (project) return "cross-file";
  if (id.startsWith("vue/")) return "vue";
  const group = categoryFiles.find((file) => categoryIncludes(file, id));
  if (!group) throw new Error(`Missing rule overview group: ${id}`);
  return group;
}

const labels: Record<VueLocale, readonly [string, string]> = {
  en: ["Group examples", "Rule reference"],
  ja: ["カテゴリの例", "ルール参照"],
  fr: ["Exemples du groupe", "Fiche de la règle"],
  "pt-BR": ["Exemplos do grupo", "Referência da regra"],
  "zh-CN": ["分组示例", "规则参考"],
};

/** Retain native rule headings and legacy example fragments without copying full packets. */
export function ruleOverview(reference: string, id: string, locale: VueLocale, project = false) {
  const slug = ruleSlug(id);
  const group = `./${owningRuleGroup(id, project)}.md#${slug}`;
  const prefix = locale === "en" || locale === "ja" ? "./" : "/rules/";
  const target = `${prefix}${project ? "project" : "reference"}/${slug}.md`;
  const [examples, detail] = labels[locale];
  const { bad, good } = vueCategoryLabels[locale];
  return [
    `### \`${id}\``,
    "",
    cataloguePurpose(reference, id, locale, project),
    "",
    `<span id="${slug}-bad"></span>`,
    "",
    `[${bad}](${group}-bad)`,
    "",
    `<span id="${slug}-good"></span>`,
    "",
    `[${good}](${group}-good)`,
    "",
    `[${examples}](${group}) · [${detail}](${target})`,
    "",
  ].join("\n");
}
