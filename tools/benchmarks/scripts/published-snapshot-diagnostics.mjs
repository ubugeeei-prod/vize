import { ENGINE_CLASS_TEXT } from "./compare-tools-report.mjs";
import { surfaceLabel } from "./published-snapshot-locales.mjs";

const TEXT = {
  ja: {
    heading: "エンジン種別ごとの順位",
    columns: ["エンジン種別", "比較対象", "中央値", "同種の最速比"],
    groups: {
      "typescript-js": "JavaScript TypeScript エンジン (tsc)",
      "tsgo-native": "ネイティブ TypeScript エンジン (tsgo)",
    },
    note: "速度比は同じネイティブ tsgo エンジンの比較に限定しています。診断範囲は異なるため、精度の同等性を示すものではありません。JavaScript TypeScript エンジンの vue-tsc は同じ実行の参考値として掲載し、エンジンをまたぐ速度比は算出しません。",
    rejected: "検証で除外。実行時間と順位は掲載しません。",
    validation:
      "型検査の実行検証（厳密なテンプレート検査、ウォームアップ前の検証、毎回の診断再確認）:",
    proofColumns: [
      "比較対象",
      "最小エラー注入",
      "コーパスへの注入",
      "診断数",
      "終了コード",
      "診断 SHA-256",
    ],
    passed: "成功",
    failed: "失敗",
    sampleColumns: ["比較対象", "中央値 (ms)", "各回の時間 (ms)"],
    phases: { preflight: "事前検証", measurement: "計測" },
  },
  "zh-CN": {
    heading: "按引擎类别分别排名",
    columns: ["引擎类别", "对比项", "中位数", "相对同类最快项"],
    groups: {
      "typescript-js": "JavaScript TypeScript 引擎 (tsc)",
      "tsgo-native": "原生 TypeScript 引擎 (tsgo)",
    },
    note: "速度比仅比较使用相同原生 tsgo 引擎的工具。诊断覆盖范围可能不同，因此这不代表准确性相同。使用 JavaScript TypeScript 引擎的 vue-tsc 仅作为同次运行的时间参考，不计算跨引擎速度比。",
    rejected: "验证未通过，不公布耗时或排名。",
    validation: "类型检查工作验证（严格模板检查；预热前验证；每次运行重新核对诊断）:",
    proofColumns: [
      "对比项",
      "最小错误注入",
      "语料库错误注入",
      "诊断数",
      "退出状态",
      "诊断 SHA-256",
    ],
    passed: "通过",
    failed: "失败",
    sampleColumns: ["对比项", "中位数 (ms)", "各次耗时 (ms)"],
    phases: { preflight: "预检", measurement: "测量" },
  },
  fr: {
    heading: "classement séparé par moteur",
    columns: ["Classe de moteur", "Variante", "Médiane", "Rapport au plus rapide de la classe"],
    groups: {
      "typescript-js": "Moteur TypeScript JavaScript (tsc)",
      "tsgo-native": "Moteur TypeScript natif (tsgo)",
    },
    note: "Les rapports comparent uniquement les outils utilisant le même moteur tsgo natif. La couverture des diagnostics peut différer : cela ne démontre pas une précision équivalente. vue-tsc, qui utilise le moteur TypeScript JavaScript, figure comme référence chronométrée lors de la même exécution, sans rapport entre moteurs différents.",
    rejected: "rejeté lors de la validation ; aucun temps ni classement publié.",
    validation:
      "Validation du travail de vérification des types (templates stricts ; avant préchauffage ; diagnostics revérifiés à chaque exécution) :",
    proofColumns: [
      "Variante",
      "Erreurs minimales injectées",
      "Injection dans le corpus",
      "Diagnostics",
      "Code de sortie",
      "SHA-256 des diagnostics",
    ],
    passed: "réussie",
    failed: "échouée",
    sampleColumns: ["Variante", "Médiane (ms)", "Exécutions (ms)"],
    phases: { preflight: "validation préalable", measurement: "mesure" },
  },
  "pt-BR": {
    heading: "classificação separada por mecanismo",
    columns: ["Classe de mecanismo", "Variante", "Mediana", "Em relação ao mais rápido da classe"],
    groups: {
      "typescript-js": "Mecanismo TypeScript JavaScript (tsc)",
      "tsgo-native": "Mecanismo TypeScript nativo (tsgo)",
    },
    note: "As razões comparam apenas ferramentas que usam o mesmo mecanismo tsgo nativo. A cobertura dos diagnósticos pode diferir; isso não demonstra precisão equivalente. O vue-tsc, que usa o mecanismo TypeScript JavaScript, aparece como referência de tempo da mesma execução, sem razão entre mecanismos diferentes.",
    rejected: "rejeitado na validação; nenhum tempo ou posição publicado.",
    validation:
      "Validação do trabalho de verificação de tipos (templates estritos; antes do aquecimento; diagnósticos conferidos a cada execução):",
    proofColumns: [
      "Variante",
      "Erros mínimos injetados",
      "Injeção no corpus",
      "Diagnósticos",
      "Código de saída",
      "SHA-256 dos diagnósticos",
    ],
    passed: "aprovada",
    failed: "reprovada",
    sampleColumns: ["Variante", "Mediana (ms)", "Execuções (ms)"],
    phases: { preflight: "pré-validação", measurement: "medição" },
  },
};

export function diagnosticText(locale) {
  if (locale === "en") return ENGINE_CLASS_TEXT;
  const t = TEXT[locale];
  if (!t) throw new Error(`Unknown diagnostic locale: ${locale}`);
  return {
    ...t,
    heading: (surface) => `${surfaceLabel(surface, locale)}: ${t.heading}`,
    group: (group) => t.groups[group.engineClass],
    note: () => t.note,
    rejected: (variant) =>
      `${variant.label} (${t.phases[variant.phase] ?? variant.phase}): ${t.rejected}`,
  };
}
