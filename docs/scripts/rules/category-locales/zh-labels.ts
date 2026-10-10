export const chineseCategoryTitles: Record<string, string> = {
  all: "全部规则",
  "type-and-script": "类型与脚本规则",
  html: "HTML 规则",
  accessibility: "无障碍规则",
  ssr: "SSR 规则",
  "petite-vue": "petite-vue 规则",
  vapor: "Vapor 规则",
  ecosystem: "生态系统规则",
  "musea-and-css": "Musea 与 CSS 规则",
  "cross-file": "跨文件规则",
};

export const chineseCategoryIntro =
  "本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。";

export const chineseCrossFileIntro =
  "本页完整展示 66 项项目检查的用途、共享项目文件，以及错误和正确示例的全部修改。两组示例都需使用各项提供的共享文件，并遵循依赖及版本说明。项目检查需要完整的已分析组件图。60 个公开跨文件诊断代码具有不同支持边界：19 个属于 CLI 分析遍（18 组已验证的源代码对照，以及一项附响应式流图的说明性 Vue 项目）；16 个有实验性 Rust 分析器实现，但该 CLI 分析遍不会单独发出这些代码；25 个仅是目前没有诊断实现的公开约定。说明性源代码和保留图的验证条件分别注明；启用规则 ID 不会激活尚不可用的实现。";
