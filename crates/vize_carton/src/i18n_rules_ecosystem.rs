//! `<rule>.description` for the ecosystem (Nuxt, Vue Router, Pinia, vue-i18n,
//! Void, Vue Test Utils) and Nuxt config rules. Registered by
//! [`crate::i18n_rules_markup`], whose module docs state the contract.

/// `(key, en, ja, zh)`.
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "ecosystem/nuxt-prefer-nuxt-link.description",
        "Prefer NuxtLink for internal application links",
        "アプリ内部へのリンクには NuxtLink を推奨する",
        "应用内部链接优先使用 NuxtLink",
    ),
    (
        "ecosystem/pinia-prefer-store-to-refs.description",
        "Prefer storeToRefs() when destructuring Pinia stores",
        "Pinia ストアを分割代入するときは storeToRefs() を推奨する",
        "解构 Pinia store 时优先使用 storeToRefs()",
    ),
    (
        "ecosystem/router-link-require-to.description",
        "Require a `to` target on RouterLink and NuxtLink components",
        "RouterLink と NuxtLink コンポーネントに遷移先の `to` を必須にする",
        "要求 RouterLink 和 NuxtLink 组件提供 `to` 目标",
    ),
    (
        "ecosystem/void-link-require-href.description",
        "Require `href` on Void Vue Link components",
        "Void Vue の Link コンポーネントに `href` を必須にする",
        "要求 Void Vue 的 Link 组件提供 `href`",
    ),
    (
        "ecosystem/void-link-valid-method.description",
        "Validate static Void Vue Link method props",
        "Void Vue の Link コンポーネントに静的に渡した method を検証する",
        "校验 Void Vue Link 组件的静态 method 属性",
    ),
    (
        "ecosystem/vue-i18n-no-missing-key.description",
        "Report static vue-i18n keys that are absent from local SFC messages",
        "SFC ローカルのメッセージに存在しない静的な vue-i18n キーを報告する",
        "报告在 SFC 本地消息中不存在的静态 vue-i18n 键",
    ),
    (
        "ecosystem/vue-router-prefer-named-link.description",
        "Prefer named route objects over static path strings in RouterLink",
        "RouterLink では静的なパス文字列より名前付きルートのオブジェクトを推奨する",
        "RouterLink 中优先使用命名路由对象，而不是静态路径字符串",
    ),
    (
        "ecosystem/vue-router-prefer-named-push.description",
        "Prefer named route objects for Vue Router programmatic navigation",
        "Vue Router のプログラムによるナビゲーションでは名前付きルートのオブジェクトを推奨する",
        "Vue Router 编程式导航优先使用命名路由对象",
    ),
    (
        "ecosystem/vue-test-utils-no-html-snapshot.description",
        "Avoid snapshotting wrapper.html() in Vue Test Utils tests",
        "Vue Test Utils のテストで wrapper.html() をスナップショットに取ることを避ける",
        "避免在 Vue Test Utils 测试中对 wrapper.html() 做快照",
    ),
    (
        "nuxt/no-nuxt-config-test-key.description",
        "Disallow setting `test` key in Nuxt config",
        "Nuxt の設定で `test` キーを指定することを禁止する",
        "禁止在 Nuxt 配置中设置 `test` 键",
    ),
    (
        "nuxt/no-page-meta-runtime-values.description",
        "Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup",
        "`definePageMeta` の直下でランタイムの値を参照することを禁止する（ビルド時に別チャンクへ抽出され、コンポーネントの setup より前に実行されるため）",
        "禁止在 `definePageMeta` 顶层使用运行时上下文的值（它在构建时被提取到单独的 chunk，并在组件 setup 之前执行）",
    ),
    (
        "nuxt/nuxt-config-keys-order.description",
        "Prefer recommended order of Nuxt config properties",
        "Nuxt の設定プロパティを推奨の順序で書くことを推奨する",
        "推荐按照建议的顺序编写 Nuxt 配置属性",
    ),
    (
        "nuxt/prefer-import-meta.description",
        "Prefer using `import.meta.*` over `process.*`",
        "`process.*` より `import.meta.*` を推奨する",
        "优先使用 `import.meta.*`，而不是 `process.*`",
    ),
];
