//! `<code>.description` for the cross-file setup-context and reactivity
//! codes. Registered by [`crate::i18n_cross_file`], whose module docs state
//! the contract.

/// `(key, en, ja, zh)`.
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "vize:croquis/cf/reactivity-outside-setup.description",
        "Report `ref`, `reactive` or `computed` called outside the setup context",
        "setup コンテキストの外での `ref`・`reactive`・`computed` の呼び出しを報告する",
        "报告在 setup 上下文之外调用 `ref`、`reactive` 或 `computed`",
    ),
    (
        "vize:croquis/cf/lifecycle-outside-setup.description",
        "Report lifecycle hooks registered outside synchronous setup",
        "setup の同期的な処理の外で登録されるライフサイクルフックを報告する",
        "报告在同步 setup 之外注册的生命周期钩子",
    ),
    (
        "vize:croquis/cf/lifecycle-without-cleanup.description",
        "Report a lifecycle hook that acquires resources without a matching cleanup hook",
        "リソースを確保しているのに対応する後始末のフックがないライフサイクルフックを報告する",
        "报告获取了资源却没有对应清理钩子的生命周期钩子",
    ),
    (
        "vize:croquis/cf/watcher-outside-setup.description",
        "Report `watch` or `watchEffect` called outside setup, where it is never stopped automatically",
        "自動では停止されない、setup の外での `watch`・`watchEffect` の呼び出しを報告する",
        "报告在 setup 之外调用、不会被自动停止的 `watch` 或 `watchEffect`",
    ),
    (
        "vize:croquis/cf/di-outside-setup.description",
        "Report `provide` or `inject` called outside setup",
        "setup の外での `provide`・`inject` の呼び出しを報告する",
        "报告在 setup 之外调用 `provide` 或 `inject`",
    ),
    (
        "vize:croquis/cf/composable-outside-setup.description",
        "Report a composable that uses Vue APIs called outside setup",
        "Vue の API を使うコンポーザブルが setup の外で呼ばれていることを報告する",
        "报告在 setup 之外调用了使用 Vue API 的组合式函数",
    ),
    (
        "vize:croquis/cf/spread-breaks-reactivity.description",
        "Report spreading a reactive object, which copies its values and drops reactivity",
        "値をコピーしてリアクティビティを失わせる、リアクティブオブジェクトのスプレッドを報告する",
        "报告展开响应式对象（会复制其值并丢失响应性）",
    ),
    (
        "vize:croquis/cf/reassignment-breaks-reactivity.description",
        "Report reassigning a reactive variable, which disconnects existing references",
        "既存の参照との繋がりを断ってしまう、リアクティブな変数の再代入を報告する",
        "报告重新赋值响应式变量（会断开已有的引用）",
    ),
    (
        "vize:croquis/cf/value-extraction-breaks-reactivity.description",
        "Report extracting a reactive value into a plain variable",
        "リアクティブな値をただの変数に取り出していることを報告する",
        "报告把响应式值提取到普通变量中",
    ),
    (
        "vize:croquis/cf/destructuring-breaks-reactivity.description",
        "Report destructuring a reactive object or props without `toRefs`",
        "`toRefs` を使わずにリアクティブオブジェクトや props を分割代入していることを報告する",
        "报告未使用 `toRefs` 就解构响应式对象或 props",
    ),
    (
        "vize:croquis/cf/reference-escapes-scope.description",
        "Report a reactive reference escaping its scope implicitly through a function parameter",
        "関数の引数を通じて、リアクティブな参照が暗黙にスコープの外へ出ていくことを報告する",
        "报告响应式引用通过函数参数隐式逃逸出其作用域",
    ),
    (
        "vize:croquis/cf/mutated-after-escape.description",
        "Report a reactive object mutated after being passed to an external function",
        "外部の関数に渡した後で変更されるリアクティブオブジェクトを報告する",
        "报告传给外部函数后又被修改的响应式对象",
    ),
    (
        "vize:croquis/cf/circular-reactive-dependency.description",
        "Report a circular reactive dependency that can loop updates forever",
        "更新が無限に繰り返されうる、リアクティブな依存の循環を報告する",
        "报告可能导致无限更新循环的响应式循环依赖",
    ),
    (
        "vize:croquis/cf/watch-can-be-computed.description",
        "Report a watcher that only derives one reactive value and could be a `computed`",
        "リアクティブな値を 1 つ導出するだけで `computed` に置き換えられるウォッチャーを報告する",
        "报告只派生一个响应式值、可以改用 `computed` 的侦听器",
    ),
    (
        "vize:croquis/cf/dom-access-without-next-tick.description",
        "Report DOM access outside lifecycle hooks or `nextTick`, before the DOM exists",
        "ライフサイクルフックや `nextTick` の外で、DOM ができる前にアクセスしていることを報告する",
        "报告在生命周期钩子或 `nextTick` 之外、DOM 尚不存在时访问 DOM",
    ),
    (
        "vize:croquis/cf/computed-side-effects.description",
        "Report side effects inside a computed getter",
        "computed ゲッター内の副作用を報告する",
        "报告计算属性 getter 中的副作用",
    ),
    (
        "vize:croquis/cf/module-scope-reactive.description",
        "Report reactive state at module scope, which SSR shares across requests",
        "SSR ではリクエスト間で共有されてしまう、モジュールスコープのリアクティブな状態を報告する",
        "报告位于模块作用域、在 SSR 中会被多个请求共享的响应式状态",
    ),
    (
        "vize:croquis/cf/template-ref-timing.description",
        "Report a template ref read during setup, before mounting fills it",
        "マウントで値が入る前の setup 中にテンプレート参照を読み取っていることを報告する",
        "报告在挂载赋值之前、于 setup 期间读取模板引用",
    ),
    (
        "vize:croquis/cf/async-boundary.description",
        "Report reactive state used across an `await`, after which the component may have unmounted or the value changed",
        "`await` をまたいでリアクティブな状態を使っていることを報告する（その間にコンポーネントがアンマウントされたり、値が変わったりしうるため）",
        "报告跨越 `await` 使用响应式状态（期间组件可能已卸载或值已改变）",
    ),
    (
        "vize:croquis/cf/injected-async-mutation-race.description",
        "Report a consumer asynchronously mutating injected provider state",
        "注入されたプロバイダーの状態を利用側が非同期に書き換えていることを報告する",
        "报告使用方异步修改注入的提供方状态",
    ),
    (
        "vize:croquis/cf/closure-captures-reactive.description",
        "Report a closure capturing reactive state implicitly",
        "クロージャーがリアクティブな状態を暗黙に捕捉していることを報告する",
        "报告闭包隐式捕获响应式状态",
    ),
    (
        "vize:croquis/cf/object-identity-comparison.description",
        "Report `===` between a reactive proxy and a raw object, whose identities differ",
        "同一性の異なるリアクティブプロキシと元のオブジェクトを `===` で比較していることを報告する",
        "报告用 `===` 比较身份不同的响应式代理与原始对象",
    ),
    (
        "vize:croquis/cf/reactive-export.description",
        "Report reactive state exported from a module as global mutable state",
        "グローバルな可変状態としてモジュールからエクスポートされるリアクティブな状態を報告する",
        "报告作为全局可变状态从模块导出的响应式状态",
    ),
    (
        "vize:croquis/cf/shallow-deep-access.description",
        "Report deep access on `shallowRef` or `shallowReactive`, whose nested changes do not trigger updates",
        "ネストした変更が更新を起こさない `shallowRef`・`shallowReactive` への深いアクセスを報告する",
        "报告对 `shallowRef` 或 `shallowReactive` 的深层访问（嵌套修改不会触发更新）",
    ),
    (
        "vize:croquis/cf/toraw-mutation.description",
        "Report mutating a `toRaw()` value, which bypasses reactivity",
        "リアクティビティを素通りする `toRaw()` の値の変更を報告する",
        "报告修改 `toRaw()` 的值（会绕过响应性）",
    ),
    (
        "vize:croquis/cf/event-listener-leak.description",
        "Report an event listener added without a matching removal",
        "対応する解除処理のないイベントリスナーの登録を報告する",
        "报告添加了事件监听器却没有对应的移除",
    ),
    (
        "vize:croquis/cf/array-mutation.description",
        "Report array mutations that do not trigger reactive updates",
        "リアクティブな更新を起こさない配列の変更を報告する",
        "报告不会触发响应式更新的数组修改",
    ),
    (
        "vize:croquis/cf/pinia-getter.description",
        "Report a Pinia getter read in setup without `storeToRefs`",
        "`storeToRefs` を使わずに setup で読み取っている Pinia のゲッターを報告する",
        "报告在 setup 中未经 `storeToRefs` 读取 Pinia getter",
    ),
    (
        "vize:croquis/cf/watcheffect-async.description",
        "Report async work inside `watchEffect`, which can race",
        "競合を起こしうる、`watchEffect` 内の非同期処理を報告する",
        "报告 `watchEffect` 中可能产生竞态的异步操作",
    ),
    (
        "vize:croquis/cf/setup-context-violation.description",
        "Report a Vue API called outside the setup context in a non-setup script",
        "setup ではない script で、setup コンテキストの外から Vue の API を呼んでいることを報告する",
        "报告在非 setup 的 script 中于 setup 上下文之外调用 Vue API",
    ),
];
