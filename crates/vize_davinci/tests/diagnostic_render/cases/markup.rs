//! Template diagnostics on one line: inline and hanging labels, footers, and
//! both fix styles.

use super::{Case, Produced, after, diagnostic, fix, help, nth, primary, secondary, span, tr};
use vize_davinci::diagnostic::Severity;
use vize_s0::Span;
use vize_s0::i18n::Locale;

const V_FOR_KEY_SOURCE: &str = r#"<script setup lang="ts">
import { ref } from "vue"

const todos = ref([{ id: 1, title: "Write the talk" }])
</script>

<template>
  <ul>
    <li v-for="todo in todos">{{ todo.title }}</li>
  </ul>
</template>
"#;

pub const V_FOR_KEY: Case = Case {
    name: "v_for_key",
    path: "src/components/TodoList.vue",
    source: V_FOR_KEY_SOURCE,
    diagnostics: |locale| {
        let directive = span(V_FOR_KEY_SOURCE, r#"v-for="todo in todos""#);
        let message = tr(
            locale,
            "Elements in iteration expect to have `v-bind:key` directives.",
            "`v-for` で繰り返す要素には `v-bind:key` が必要です。",
            "使用 `v-for` 迭代的元素需要 `v-bind:key` 指令。",
        );
        let label = tr(
            locale,
            "each `<li>` rendered here needs a stable key",
            "ここで描画される `<li>` にはそれぞれ一意なキーが必要です",
            "此处渲染的每个 `<li>` 都需要稳定的 key",
        );
        let why = tr(
            locale,
            "without a key, Vue reuses DOM nodes in place and can mix up item state when the list changes",
            "キーがないと Vue は DOM をその場で使い回すため、一覧が変わったときに項目の状態が入れ替わることがあります",
            "没有 key 时，Vue 会就地复用 DOM 节点，列表变化时各项的状态可能会错乱",
        );
        let title = tr(
            locale,
            "key each item by its id",
            "各項目の `id` をキーにしてください",
            "用每一项的 `id` 作为 key",
        );
        let produced = diagnostic(Severity::Error, directive, message)
            .with_part(primary(directive, label))
            .with_part(help(directive, why))
            .with_part(help(directive, title))
            .with_part(fix(after(directive), r#" :key="todo.id""#));
        vec![(Some("vue/require-v-for-key"), produced)]
    },
};

const DUPLICATE_ATTRIBUTE_SOURCE: &str = r#"<template>
  <button class="btn" type="submit" class="btn-primary">Save</button>
</template>
"#;

pub const DUPLICATE_ATTRIBUTE: Case = Case {
    name: "duplicate_attribute",
    path: "src/components/SaveButton.vue",
    source: DUPLICATE_ATTRIBUTE_SOURCE,
    diagnostics: |locale| {
        let source = DUPLICATE_ATTRIBUTE_SOURCE;
        let first = span(source, r#"class="btn""#);
        let again = span(source, r#"class="btn-primary""#);
        let message = tr(
            locale,
            "Duplicate attribute `class`.",
            "属性 `class` が重複しています。",
            "属性 `class` 重复。",
        );
        let produced = diagnostic(Severity::Error, again, message)
            .with_part(primary(
                again,
                tr(
                    locale,
                    "`class` is set again here",
                    "ここで `class` をもう一度指定しています",
                    "此处再次设置了 `class`",
                ),
            ))
            .with_part(secondary(
                first,
                tr(
                    locale,
                    "first set here",
                    "最初の指定はここです",
                    "首次设置在此处",
                ),
            ))
            .with_part(help(
                again,
                tr(
                    locale,
                    "merge the values into one `class`",
                    "値を 1 つの `class` にまとめてください",
                    "将这些值合并到同一个 `class` 中",
                ),
            ))
            .with_part(fix(first, r#"class="btn btn-primary""#))
            .with_part(fix(Span::new(again.start - 1, again.end), ""));
        vec![(Some("vue/no-duplicate-attributes"), produced)]
    },
};

const V_IF_WITH_V_FOR_SOURCE: &str = r#"<script setup lang="ts">
defineProps<{ users: { id: number; name: string; active: boolean }[] }>()
</script>

<template>
  <ul>
    <li v-for="user in users" v-if="user.active" :key="user.id">
      {{ user.name }}
    </li>
  </ul>
</template>
"#;

pub const V_IF_WITH_V_FOR: Case = Case {
    name: "v_if_with_v_for",
    path: "src/components/ActiveUsers.vue",
    source: V_IF_WITH_V_FOR_SOURCE,
    diagnostics: |locale| {
        let source = V_IF_WITH_V_FOR_SOURCE;
        let v_for = span(source, r#"v-for="user in users""#);
        let v_if = span(source, r#"v-if="user.active""#);
        let message = tr(
            locale,
            "`v-if` cannot read `user`: it is evaluated before `v-for` declares it.",
            "`v-if` は `v-for` より先に評価されるため、`user` を参照できません。",
            "`v-if` 先于 `v-for` 求值，因此无法访问 `user`。",
        );
        let produced = diagnostic(Severity::Error, v_if, message)
            .with_part(secondary(
                v_for,
                tr(
                    locale,
                    "`user` is declared by this `v-for`…",
                    "`user` はこの `v-for` で宣言されますが…",
                    "`user` 由这个 `v-for` 声明……",
                ),
            ))
            .with_part(primary(
                v_if,
                tr(
                    locale,
                    "…but this `v-if` runs first",
                    "…この `v-if` のほうが先に評価されます",
                    "……但这个 `v-if` 会先求值",
                ),
            ))
            .with_part(help(
                v_if,
                tr(
                    locale,
                    "filter the list in a computed property, or wrap the item in `<template v-for>` and put `v-if` inside it",
                    "算出プロパティで一覧を絞り込むか、`<template v-for>` で囲んでその内側で `v-if` を使ってください",
                    "请在计算属性中过滤列表，或用 `<template v-for>` 包裹后在内部使用 `v-if`",
                ),
            ));
        vec![(Some("vue/no-use-v-if-with-v-for"), produced)]
    },
};

const SHORTHAND_SOURCE: &str = r#"<template>
  <form v-on:submit.prevent="save">
    <input v-model="title" v-on:keydown.enter="save">
  </form>
</template>
"#;

pub const SHORTHAND_FIX: Case = Case {
    name: "shorthand_fix",
    path: "src/components/TitleForm.vue",
    source: SHORTHAND_SOURCE,
    diagnostics: |locale| {
        let source = SHORTHAND_SOURCE;
        let submit = span(source, r#"v-on:submit.prevent="save""#);
        let keydown = span(source, r#"v-on:keydown.enter="save""#);
        let message = tr(
            locale,
            "Expected `@` instead of `v-on:`.",
            "`v-on:` ではなく `@` を使ってください。",
            "应使用 `@` 而不是 `v-on:`。",
        );
        let produced = diagnostic(Severity::Warning, submit, message)
            .with_part(primary(
                submit,
                tr(
                    locale,
                    "long-form `v-on:`",
                    "省略していない `v-on:` 記法です",
                    "完整写法的 `v-on:`",
                ),
            ))
            .with_part(secondary(
                keydown,
                tr(locale, "and here", "ここも同様です", "这里也是"),
            ))
            .with_part(help(
                submit,
                tr(
                    locale,
                    "use the `@` shorthand",
                    "省略記法の `@` に置き換えます",
                    "改用 `@` 简写",
                ),
            ))
            .with_part(fix(nth(source, "v-on:", 0), "@"))
            .with_part(fix(nth(source, "v-on:", 1), "@"));
        vec![(Some("vue/v-on-style"), produced)]
    },
};

const SEVERITIES_SOURCE: &str = r#"<script setup lang="ts">
const count = ref(0)
</script>

<template>
  <div  @click="count++">{{ count }}</div>
</template>
"#;

pub const SEVERITIES: Case = Case {
    name: "severities",
    path: "src/components/Counter.vue",
    source: SEVERITIES_SOURCE,
    diagnostics: |locale| severities(locale),
};

fn severities(locale: Locale) -> Produced {
    let source = SEVERITIES_SOURCE;
    let click = span(source, r#"@click="count++""#);
    let warning = diagnostic(
        Severity::Warning,
        click,
        tr(
            locale,
            "Visible, non-interactive elements with click handlers must have a keyboard handler.",
            "クリックハンドラーを持つ非インタラクティブな要素には、キーボード用のハンドラーも必要です。",
            "带有点击处理程序的非交互元素还必须有键盘事件处理程序。",
        ),
    )
    .with_part(help(
        click,
        tr(
            locale,
            "keyboard users cannot trigger this handler.\nprefer a `<button>`, or add `@keydown.enter` and `tabindex=\"0\"`",
            "キーボードの利用者はこのハンドラーを実行できません。\n`<button>` を使うか、`@keydown.enter` と `tabindex=\"0\"` を追加してください",
            "键盘用户无法触发此处理程序。\n请改用 `<button>`，或添加 `@keydown.enter` 和 `tabindex=\"0\"`",
        ),
    ));
    let spaces = span(source, "  @click");
    let spaces = Span::new(spaces.start, spaces.start + 2);
    let hint = diagnostic(
        Severity::Hint,
        spaces,
        tr(
            locale,
            "Multiple spaces found before `@click`.",
            "`@click` の前に空白が連続しています。",
            "`@click` 前有多个连续空格。",
        ),
    )
    .with_part(fix(spaces, " "));
    let reference = span(source, "ref");
    let info = diagnostic(
        Severity::Info,
        reference,
        tr(
            locale,
            "`ref` is used without an import.\nIt resolves through the auto-import of Vue APIs.",
            "`ref` はインポートせずに使われています。\nVue API の自動インポートによって解決されます。",
            "`ref` 未经导入即被使用。\n它通过 Vue API 的自动导入解析。",
        ),
    );
    vec![
        (Some("a11y/click-events-have-key-events"), warning),
        (Some("vue/no-multi-spaces"), hint),
        (None, info),
    ]
}
