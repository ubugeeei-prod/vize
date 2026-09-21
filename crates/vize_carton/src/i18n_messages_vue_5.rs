//! Catalogue entries for Vue template and SFC lint rules (`vue/*`), part 5 of 5.
//! Registered by [`crate::i18n_messages`], whose module docs state the contract.

/// `(key, en, ja, zh)`.
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "vue/v-slot-style.message_v_slot",
        "Expected 'v-slot' instead of '{actual}'",
        "'{actual}' ではなく 'v-slot' を使用してください",
        "应使用 'v-slot' 而不是 '{actual}'",
    ),
    (
        "vue/v-slot-style.help",
        "**Shorthand:** `<template #header>` (default)\n**Longform:** `<template v-slot:header>`\n\nChoose one style and be consistent.",
        "**省略記法:** `<template #header>` (デフォルト)\n**完全記法:** `<template v-slot:header>`\n\n一貫したスタイルを選択してください。",
        "**简写:** `<template #header>` (默认)\n**完整形式:** `<template v-slot:header>`\n\n选择一种风格并保持一致。",
    ),
    (
        "vue/prop-name-casing.description",
        "Enforce a casing for declared prop names",
        "宣言されたprop名のケーシングを強制する",
        "强制声明的prop名使用指定的命名风格",
    ),
    (
        "vue/prop-name-casing.message",
        "Prop \"{name}\" is not in {casing}",
        "prop \"{name}\" は {casing} ではありません",
        "prop \"{name}\" 不是 {casing}",
    ),
    (
        "vue/prop-name-casing.help",
        "Declare props in camelCase; a template may still write them in kebab-case:\n```vue\n<!-- Bad -->\ndefineProps({ 'my-prop': String })\n\n<!-- Good -->\ndefineProps({ myProp: String })\n```",
        "propの宣言にはcamelCaseを使用してください（テンプレート側はkebab-caseのままで構いません）:\n```vue\n<!-- 悪い例 -->\ndefineProps({ 'my-prop': String })\n\n<!-- 良い例 -->\ndefineProps({ myProp: String })\n```",
        "声明prop时使用camelCase（模板中仍可写成kebab-case）:\n```vue\n<!-- 错误 -->\ndefineProps({ 'my-prop': String })\n\n<!-- 正确 -->\ndefineProps({ myProp: String })\n```",
    ),
    (
        "vue/html-quotes.description",
        "Enforce quotes style of HTML attributes",
        "HTML属性の引用符スタイルを強制する",
        "强制HTML属性引号风格",
    ),
    (
        "vue/html-quotes.message_double",
        "Expected double quotes but found single quotes",
        "シングルクォートではなくダブルクォートを使用してください",
        "应使用双引号而非单引号",
    ),
    (
        "vue/html-quotes.message_single",
        "Expected single quotes but found double quotes",
        "ダブルクォートではなくシングルクォートを使用してください",
        "应使用单引号而非双引号",
    ),
    (
        "vue/html-quotes.help",
        "Use consistent quote style for HTML attribute values.",
        "HTML属性値に一貫した引用符スタイルを使用してください。",
        "HTML属性值使用一致的引号风格。",
    ),
    (
        "vue/component-definition-name-casing.description",
        "Enforce PascalCase or kebab-case for component file names",
        "コンポーネントファイル名にPascalCaseまたはkebab-caseを強制する",
        "强制组件文件名使用PascalCase或kebab-case",
    ),
    (
        "vue/component-definition-name-casing.message",
        "Component file name '{name}' should be PascalCase or kebab-case",
        "コンポーネントファイル名'{name}'はPascalCaseまたはkebab-caseにすべきです",
        "组件文件名'{name}'应使用PascalCase或kebab-case",
    ),
    (
        "vue/component-definition-name-casing.help",
        "Rename the file to PascalCase or kebab-case:\n```\nmyComponent.vue  -> MyComponent.vue\nmy-Component.vue -> my-component.vue\n```",
        "ファイル名をPascalCaseまたはkebab-caseに変更してください:\n```\nmyComponent.vue  -> MyComponent.vue\nmy-Component.vue -> my-component.vue\n```",
        "将文件名改为PascalCase或kebab-case:\n```\nmyComponent.vue  -> MyComponent.vue\nmy-Component.vue -> my-component.vue\n```",
    ),
    (
        "vue/use-unique-element-ids.message_form",
        "Form element has static id=\"{value}\" — use useId() for unique IDs to ensure label/input associations work correctly",
        "フォーム要素に静的id=\"{value}\"が指定されています。ラベル/入力の関連付けが正しく機能するようuseId()を使用してください",
        "表单元素使用了静态id=\"{value}\"——请使用useId()确保标签/输入关联正常工作",
    ),
    (
        "vue/use-unique-element-ids.message_aria_ref",
        "Static id=\"{value}\" on element with ARIA references — use useId() to ensure ARIA associations remain unique",
        "ARIA参照を持つ要素に静的id=\"{value}\"が指定されています。ARIA関連付けの一意性を保つためuseId()を使用してください",
        "带有ARIA引用的元素使用了静态id=\"{value}\"——请使用useId()确保ARIA关联的唯一性",
    ),
    (
        "vue/permitted-contents.description",
        "Enforce HTML content model rules",
        "HTMLコンテンツモデルルールを強制",
        "强制执行HTML内容模型规则",
    ),
    (
        "vue/permitted-contents.block_in_inline",
        "<{child}> (block element) is not allowed inside <{parent}> (phrasing content only)",
        "<{child}>（ブロック要素）は<{parent}>（フレージングコンテンツのみ許可）内に配置できません",
        "<{child}>（块级元素）不允许出现在<{parent}>（仅允许短语内容）内",
    ),
    (
        "vue/permitted-contents.interactive_nesting",
        "<{tag}> is an interactive element and cannot be nested inside another interactive element",
        "<{tag}>はインタラクティブ要素であり、他のインタラクティブ要素内にネストできません",
        "<{tag}>是交互元素，不能嵌套在其他交互元素内",
    ),
    (
        "vue/permitted-contents.invalid_child",
        "<{child}> is not allowed as a direct child of <{parent}>",
        "<{child}>は<{parent}>の直接の子要素として許可されていません",
        "<{child}>不允许作为<{parent}>的直接子元素",
    ),
    (
        "vue/permitted-contents.help",
        "**Why:** HTML has content model rules that define which elements can be nested inside others. Violating these rules causes browsers to auto-correct the DOM, leading to unexpected rendering.\n\n**Common rules:**\n- `<p>`, `<span>`, `<a>` can only contain phrasing (inline) content\n- `<a>` and `<button>` cannot be nested inside each other\n- `<ul>/<ol>` direct children must be `<li>`\n- `<table>` direct children must be `<thead>`, `<tbody>`, `<tfoot>`, `<tr>`, or `<caption>`",
        "**理由:** HTMLにはコンテンツモデルルールがあり、どの要素をどの要素内にネストできるかが定義されています。これらのルールに違反すると、ブラウザがDOMを自動修正し、予期しない表示になります。\n\n**主なルール:**\n- `<p>`、`<span>`、`<a>`にはフレージング（インライン）コンテンツのみ配置可能\n- `<a>`と`<button>`は互いにネスト不可\n- `<ul>/<ol>`の直接の子は`<li>`のみ\n- `<table>`の直接の子は`<thead>`、`<tbody>`、`<tfoot>`、`<tr>`、`<caption>`のみ",
        "**原因:** HTML有内容模型规则，定义了哪些元素可以嵌套在其他元素内。违反这些规则会导致浏览器自动修正DOM，产生意外的渲染结果。\n\n**主要规则:**\n- `<p>`、`<span>`、`<a>`只能包含短语（内联）内容\n- `<a>`和`<button>`不能相互嵌套\n- `<ul>/<ol>`的直接子元素只能是`<li>`\n- `<table>`的直接子元素只能是`<thead>`、`<tbody>`、`<tfoot>`、`<tr>`或`<caption>`",
    ),
    (
        "vue/no-boolean-attr-value.message",
        "Boolean attribute \"{attr}\" should not have value \"{value}\"",
        "ブール属性\"{attr}\"に値\"{value}\"を指定すべきではありません",
        "布尔属性\"{attr}\"不应有值\"{value}\"",
    ),
    (
        "vue/no-boolean-attr-value.help",
        "Remove the value. Use just {attr} instead of {attr}=\"...\".",
        "値を削除してください。{attr}=\"...\"ではなく{attr}のみを使用してください。",
        "请移除值。使用{attr}而不是{attr}=\"...\"。",
    ),
];
