export const accessibilityIntroductions: Readonly<Record<string, readonly string[]>> = {
  core: [
    "Use these rules to check the basic accessibility information in a Vue template: image alternatives,",
    "link text, valid ARIA attributes, keyboard equivalents, and form labels. Start here when adding",
    "images, links, or inputs to a component.",
    "",
    "For example, open [form-control-has-label](./reference/a11y-form-control-has-label.md) before adding",
    "an input: compare its Bad/Good examples, enable the rule with the shown Vite+ configuration, and",
    "run `vp run lint`. The [complete accessibility guide](./accessibility.md) explains the broader rule set.",
  ],
  structure: [
    "Use these rules to review how a page is organized and described: headings, landmarks, labels,",
    "frames, media, and focusable interactive elements. Start here when a component adds a new section,",
    "embedded frame, image, or media player.",
    "",
    "For example, [heading-levels](./reference/a11y-heading-levels.md) provides a concrete heading-order",
    "check. Compare its Bad/Good examples, enable the rule using its configuration, and run `vp run lint`.",
    "For the full set, follow the [accessibility guide](./accessibility.md).",
  ],
  interactions: [
    "Use these rules when reviewing how people interact with a component: keyboard access, focus,",
    "mouse handlers, hidden controls, and references to other elements. Check them when introducing",
    "custom controls or changing visibility and event handling.",
    "",
    "For example, a focusable element hidden from assistive technology is covered by",
    "[no-aria-hidden-on-focusable](./reference/a11y-no-aria-hidden-on-focusable.md).",
    "Compare its Bad/Good examples, use the shown configuration, then run `vp run lint`.",
    "The [complete accessibility guide](./accessibility.md) explains how these checks fit together.",
  ],
  integrity: [
    "Use these rules when a template's roles, focus order, labels, or element relationships disagree.",
    "They help review the consistency of markup after adding ARIA roles or changing an interactive component.",
    "",
    "For example, [role-has-required-aria-props](./reference/a11y-role-has-required-aria-props.md) shows",
    "which change to make when an assigned role is missing a required property. Compare the examples,",
    "use that rule's configuration, then run `vp run lint`.",
    "The [complete accessibility guide](./accessibility.md) links the other checks.",
  ],
};

export const japaneseAccessibilityIntroductions: Readonly<Record<string, readonly string[]>> = {
  core: [
    "このルール群では、画像の代替テキスト、リンクの内容、ARIA 属性、キーボード操作、フォームのラベルなど、",
    "Vue テンプレートの基本的なアクセシビリティ情報を確認します。",
    "画像やリンク、入力欄をコンポーネントに追加するときは、ここから確認してください。",
    "",
    "たとえば、入力欄を追加する前に[フォーム部品のラベル](./reference/a11y-form-control-has-label.md)を開き、",
    "悪い例と良い例を比較します。掲載された Vite+ の設定でルールを有効にし、`vp run lint` を実行してください。",
    "[アクセシビリティの全ルール](./accessibility.md)では、関連するルールも確認できます。",
  ],
};
