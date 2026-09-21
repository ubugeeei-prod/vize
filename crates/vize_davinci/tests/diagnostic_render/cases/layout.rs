//! Spans across lines — opening at a line's content or mid-line, nested,
//! sharing a margin column, long enough to elide — and labels across lines.

use super::{Case, after, between, diagnostic, fix, help, primary, secondary, span, tr};
use vize_davinci::diagnostic::Severity;

const ELEMENT_SOURCE: &str = r#"<template>
  <header>
    <img
      src="/vize.svg"
      class="logo"
    >
  </header>
</template>
"#;

pub const MULTILINE_ELEMENT: Case = Case {
    name: "multiline_element",
    path: "src/components/AppHeader.vue",
    source: ELEMENT_SOURCE,
    diagnostics: |locale| {
        let image = between(ELEMENT_SOURCE, "<img", ">");
        let message = tr(
            locale,
            "`<img>` elements must have an `alt` attribute.",
            "`<img>` 要素には `alt` 属性が必要です。",
            "`<img>` 元素必须有 `alt` 属性。",
        );
        let produced = diagnostic(Severity::Error, image, message)
            .with_part(primary(
                image,
                tr(
                    locale,
                    "screen readers cannot describe this image",
                    "スクリーンリーダーがこの画像の内容を読み上げられません",
                    "屏幕阅读器无法描述这张图片",
                ),
            ))
            .with_part(help(
                image,
                tr(
                    locale,
                    "use `alt=\"\"` if the image is purely decorative",
                    "装飾のための画像なら `alt=\"\"` を指定してください",
                    "如果图片仅用于装饰，请使用 `alt=\"\"`",
                ),
            ))
            .with_part(help(
                image,
                tr(
                    locale,
                    "describe what the image shows",
                    "画像の内容を説明する代替テキストを指定してください",
                    "添加描述图片内容的替代文本",
                ),
            ))
            .with_part(fix(after(span(ELEMENT_SOURCE, "<img")), r#" alt="Vize""#));
        vec![(Some("a11y/img-alt"), produced)]
    },
};

const ATTRIBUTE_SOURCE: &str = r#"<template>
  <p :style="{
    color: 'crimson',
    fontWeight: 700,
  }">
    Static styles
  </p>
</template>
"#;

pub const MULTILINE_ATTRIBUTE: Case = Case {
    name: "multiline_attribute",
    path: "src/components/Notice.vue",
    source: ATTRIBUTE_SOURCE,
    diagnostics: |locale| {
        let binding = between(ATTRIBUTE_SOURCE, ":style", "}\"");
        let message = tr(
            locale,
            "Static inline styles are not allowed.",
            "静的なインラインスタイルは使用できません。",
            "不允许使用静态内联样式。",
        );
        let produced = diagnostic(Severity::Warning, binding, message)
            .with_part(primary(
                binding,
                tr(
                    locale,
                    "every value in this binding is a constant",
                    "このバインディングの値はすべて定数です",
                    "该绑定中的所有值都是常量",
                ),
            ))
            .with_part(help(
                binding,
                tr(
                    locale,
                    "move the declarations into a class in `<style scoped>`",
                    "宣言は `<style scoped>` のクラスに移してください",
                    "请将这些声明移到 `<style scoped>` 中的类里",
                ),
            ));
        vec![(Some("vue/no-static-inline-styles"), produced)]
    },
};

const NESTED_SOURCE: &str = r#"<template>
  <Transition name="fade">
    <p v-if="show">
      Hello
    </p>
    <span>
      world
    </span>
  </Transition>
</template>
"#;

pub const NESTED_MULTILINE: Case = Case {
    name: "nested_multiline",
    path: "src/components/Greeting.vue",
    source: NESTED_SOURCE,
    diagnostics: |locale| {
        let transition = between(NESTED_SOURCE, "<Transition", "</Transition>");
        let toggled = between(NESTED_SOURCE, "<p v-if", "</p>");
        let fixed = between(NESTED_SOURCE, "<span>", "</span>");
        let message = tr(
            locale,
            "The element inside `<Transition>` must toggle with `v-if` or `v-show`.",
            "`<Transition>` の中の要素は `v-if` か `v-show` で表示を切り替える必要があります。",
            "`<Transition>` 内的元素必须通过 `v-if` 或 `v-show` 切换显示。",
        );
        let produced = diagnostic(Severity::Error, fixed, message)
            .with_part(secondary(
                transition,
                tr(
                    locale,
                    "a transition only animates children that enter or leave",
                    "トランジションは、表示・非表示が切り替わる子要素にしか適用されません",
                    "过渡只会作用于进入或离开的子元素",
                ),
            ))
            .with_part(secondary(
                toggled,
                tr(
                    locale,
                    "this sibling toggles correctly",
                    "こちらの要素は正しく切り替わります",
                    "这个兄弟元素能正确切换",
                ),
            ))
            .with_part(primary(
                fixed,
                tr(
                    locale,
                    "this `<span>` never enters or leaves",
                    "この `<span>` は表示が切り替わりません",
                    "这个 `<span>` 永远不会进入或离开",
                ),
            ));
        vec![(Some("vue/require-toggle-inside-transition"), produced)]
    },
};

const LONG_SOURCE: &str = r#"<template>
  <p class="a">A</p>
</template>

<style scoped>
.a {
  color: red;
}
</style>

<style scoped>
.a {
  margin: 0;
  padding: 0;
  border: 0;
  font: inherit;
  vertical-align: baseline;
  line-height: 1.5;
  letter-spacing: 0;
  text-decoration: none;
  list-style: none;
}
</style>
"#;

pub const LONG_SPAN: Case = Case {
    name: "long_span",
    path: "src/components/Styled.vue",
    source: LONG_SOURCE,
    diagnostics: |locale| {
        let block = |nth: usize| {
            let open = super::nth(LONG_SOURCE, "<style scoped>", nth);
            let close = super::nth(LONG_SOURCE, "</style>", nth);
            vize_s0::Span::new(open.start, close.end)
        };
        let message = tr(
            locale,
            "Only one `<style scoped>` block is allowed per component.",
            "`<style scoped>` ブロックはコンポーネントごとに 1 つまでです。",
            "每个组件只允许一个 `<style scoped>` 块。",
        );
        let produced = diagnostic(Severity::Warning, block(1), message)
            .with_part(primary(
                block(1),
                tr(
                    locale,
                    "a second `<style scoped>` block",
                    "2 つ目の `<style scoped>` ブロックです",
                    "第二个 `<style scoped>` 块",
                ),
            ))
            .with_part(secondary(
                block(0),
                tr(
                    locale,
                    "the first one is here",
                    "1 つ目はここです",
                    "第一个在这里",
                ),
            ))
            .with_part(help(
                block(1),
                tr(
                    locale,
                    "merge the rules into a single block",
                    "ルールを 1 つのブロックにまとめてください",
                    "请将规则合并到同一个块中",
                ),
            ));
        vec![(Some("vue/single-style-block"), produced)]
    },
};

const LABELS_SOURCE: &str = r#"<template>
  <NameInput v-model="form.name" :model-value="name" @update:model-value="setName" />
</template>
"#;

pub const MULTI_LINE_LABELS: Case = Case {
    name: "multi_line_labels",
    path: "src/components/Profile.vue",
    source: LABELS_SOURCE,
    diagnostics: |locale| {
        let model = span(LABELS_SOURCE, r#"v-model="form.name""#);
        let value = span(LABELS_SOURCE, r#":model-value="name""#);
        let update = span(LABELS_SOURCE, r#"@update:model-value="setName""#);
        let message = tr(
            locale,
            "`v-model` is combined with the prop and listener it expands to.",
            "`v-model` と、その展開先であるプロパティとリスナーが同時に指定されています。",
            "`v-model` 与其展开后的属性和监听器同时使用。",
        );
        let produced = diagnostic(Severity::Error, model, message)
            .with_part(primary(
                model,
                tr(
                    locale,
                    "`v-model` expands to a `modelValue` prop\nand an `update:modelValue` listener…",
                    "`v-model` は `modelValue` プロパティと\n`update:modelValue` リスナーに展開されます…",
                    "`v-model` 会展开为 `modelValue` 属性\n和 `update:modelValue` 监听器……",
                ),
            ))
            .with_part(secondary(
                value,
                tr(
                    locale,
                    "…so this prop conflicts with it",
                    "…そのため、このプロパティと衝突します",
                    "……因此与这个属性冲突",
                ),
            ))
            .with_part(secondary(
                update,
                tr(
                    locale,
                    "…and this listener runs twice",
                    "…このリスナーも二重に実行されます",
                    "……这个监听器也会执行两次",
                ),
            ))
            .with_part(help(
                model,
                tr(
                    locale,
                    "keep either `v-model` or the explicit pair, not both",
                    "`v-model` と明示的な組み合わせのどちらか一方だけを残してください",
                    "`v-model` 与显式的属性和监听器只能保留其一",
                ),
            ));
        vec![(Some("vue/valid-v-model"), produced)]
    },
};
