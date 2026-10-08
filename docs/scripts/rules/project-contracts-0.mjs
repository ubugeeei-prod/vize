export const contract0Examples = {
  "array-mutation": {
    shared: {
      "main.ts":
        "import Vue from 'vue';\nimport App from './App.vue';\nnew Vue({ render: h => h(App) }).$mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        "<script lang=\"ts\">\nimport Vue from 'vue';\nimport { replaceFirst } from './replace-first';\nexport default Vue.extend({\n  data() { return { items: ['Before'] }; },\n  methods: { replace() { replaceFirst(this.items, 'After'); } },\n});\n</script>\n<template><section><p>{{ items[0] }}</p><button @click=\"replace\">Replace</button></section></template>\n",
    },
    bad: {
      "replace-first.ts":
        "export function replaceFirst(items: string[], next: string): void {\n  items[0] = next;\n}\n",
    },
    good: {
      "replace-first.ts":
        "export function replaceFirst(items: string[], next: string): void {\n  items.splice(0, 1, next);\n}\n",
    },
    badExplanation: {
      en: "In this historical Vue 2.7 project, `items[0] = next` changes the array without notifying Vue 2’s array observer, so the displayed first item need not update.",
      ja: "この旧 Vue 2.7 プロジェクトでは、`items[0] = next` が Vue 2 の配列 observer に通知せず配列を変えるため、表示中の最初の要素が更新されない場合があります。",
    },
    goodExplanation: {
      en: "`splice(0, 1, next)` uses the array mutation method observed by Vue 2, allowing the same replacement to update the view.",
      ja: "`splice(0, 1, next)` で Vue 2 が監視する配列の変更メソッドを使い、同じ置換を画面に反映します。",
    },
    note: {
      en: "Historical Vue 2.7 only: use matching Vue 2.7 and SFC compiler dependencies for this scenario. Vue 3 proxies track array index assignment, so `items[0] = next` is reactive in Vue 3 and is not a Vue 3 defect. This published code has no current producer.",
      ja: "旧 Vue 2.7 に限る例です。対応する Vue 2.7 と SFC コンパイラーを使うことを前提とします。Vue 3 の Proxy は配列のインデックス代入を追跡するため、`items[0] = next` は Vue 3 ではリアクティブであり、不具合ではありません。この公開コードには現在の生成元がありません。",
    },
  },
  "circular-dep": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        "<script setup lang=\"ts\">\nimport { aLabel } from './a';\n</script>\n\n<template>\n<p>{{ aLabel }}</p>\n</template>\n",
      "labels.ts": "export const aPrefix = 'A';\nexport const bPrefix = 'B';\n",
    },
    bad: {
      "a.ts": "import { bLabel } from './b';\nexport const aLabel = 'A' + bLabel;\n",
      "b.ts": "import { aLabel } from './a';\nexport const bLabel = 'B' + aLabel;\n",
    },
    good: {
      "a.ts": "import { bPrefix } from './labels';\nexport const aLabel = 'A' + bPrefix;\n",
      "b.ts": "import { aPrefix } from './labels';\nexport const bLabel = 'B' + aPrefix;\n",
    },
    badExplanation: {
      en: "`a.ts` imports `b.ts`, which imports `a.ts` back. Both eagerly initialize a constant from the other module’s still-uninitialized constant, creating a temporal-dead-zone failure.",
      ja: "`a.ts` が `b.ts` を読み、`b.ts` が `a.ts` を読み返します。両方が相手の未初期化の定数から即座に定数を作るため、初期化前のアクセスが発生します。",
    },
    goodExplanation: {
      en: "Both modules read initialized prefixes from the independent `labels.ts` module, removing the cycle and the eager cross-read.",
      ja: "両モジュールが独立した `labels.ts` の初期化済みの接頭辞を読み、循環と初期化前の相互参照を取り除きます。",
    },
    note: {
      en: "This illustrates a concrete eager-initialization cycle. A recursive Vue component or every circular import is not automatically erroneous. No current producer emits this contract code.",
      ja: "即時初期化による具体的な循環の例です。再帰する Vue コンポーネントや、あらゆる循環 import が必ず誤りになるわけではありません。現在この契約コードを生成する検査はありません。",
    },
  },
  "closure-captures-reactive": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        "<script setup lang=\"ts\">\nimport { computed, ref } from 'vue';\nimport { makeReader } from './reader';\nconst count = ref(0);\nconst read = makeReader(count);\nconst shown = computed(read);\n</script>\n\n<template>\n<button @click=\"count++\">Increment {{ count }}</button><p>{{ shown }}</p>\n</template>\n",
    },
    bad: {
      "reader.ts":
        "import type { Ref } from 'vue';\nexport function makeReader(count: Ref<number>): () => number {\n  const captured = count.value;\n  return () => captured;\n}\n",
    },
    good: {
      "reader.ts":
        "import type { Ref } from 'vue';\nexport function makeReader(count: Ref<number>): () => number {\n  return () => count.value;\n}\n",
    },
    badExplanation: {
      en: "`makeReader` copies `count.value` before creating the closure. The computed reader then returns that initial number without reading a reactive dependency.",
      ja: "`makeReader` がクロージャーを作る前に `count.value` をコピーします。computed の読取処理はリアクティブな依存を読まず、最初の数値を返し続けます。",
    },
    goodExplanation: {
      en: "The closure reads `count.value` when invoked, so the computed getter can track the ref and update `shown` after increments.",
      ja: "呼び出すたびにクロージャー内で `count.value` を読み、computed が ref を追跡して増加後の `shown` を更新できるようにします。",
    },
  },
  "composable-outside-setup": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        "<script setup lang=\"ts\">\nimport { useTitle } from './use-title';\nconst title = useTitle();\n</script>\n\n<template>\n<h1>{{ title }}</h1>\n</template>\n",
    },
    bad: {
      "use-title.ts":
        "import { onMounted, ref } from 'vue';\nconst title = ref('Before mount');\nonMounted(() => { title.value = 'Mounted'; });\nexport function useTitle() { return title; }\n",
    },
    good: {
      "use-title.ts":
        "import { onMounted, ref } from 'vue';\nexport function useTitle() {\n  const title = ref('Before mount');\n  onMounted(() => { title.value = 'Mounted'; });\n  return title;\n}\n",
    },
    badExplanation: {
      en: "Importing `use-title.ts` registers `onMounted` before a component setup is active. Calling its exported function later only returns that module-level ref; it cannot repair the missed lifecycle ownership.",
      ja: "`use-title.ts` の import 時点で、コンポーネントの setup が有効になる前に `onMounted` を登録します。後から公開関数を呼んでもモジュール直下の ref を返すだけで、ライフサイクルの所属は修復されません。",
    },
    goodExplanation: {
      en: "Both state creation and hook registration move into `useTitle`, which App calls synchronously inside setup. The mounted hook now belongs to that App instance.",
      ja: "状態作成とフック登録を `useTitle` の中へ移し、App の setup 内から同期的に呼びます。mounted フックがその App インスタンスに所属するようになります。",
    },
    note: {
      en: "The concern is this lifecycle-dependent composable, not a blanket ban on ordinary utility functions or all Composition API calls outside setup. This contract has no current producer.",
      ja: "対象はライフサイクルに依存するこの composable です。通常のユーティリティ関数や、setup 外のあらゆる Composition API 呼び出しを禁止する例ではありません。この契約には現在の生成元がありません。",
    },
  },
  "computed-side-effects": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        '<script setup lang="ts">\nimport { useDouble } from \'./use-double\';\nconst { count, doubled, lastCalculated } = useDouble();\n</script>\n\n<template>\n<button @click="count++">Increment {{ count }}</button><p>{{ doubled }} / {{ lastCalculated }}</p>\n</template>\n',
    },
    bad: {
      "use-double.ts":
        "import { computed, ref } from 'vue';\nexport function useDouble() {\n  const count = ref(0);\n  const lastCalculated = ref(0);\n  const doubled = computed(() => {\n    const next = count.value * 2;\n    lastCalculated.value = next;\n    return next;\n  });\n  return { count, doubled, lastCalculated };\n}\n",
    },
    good: {
      "use-double.ts":
        "import { computed, ref, watch } from 'vue';\nexport function useDouble() {\n  const count = ref(0);\n  const lastCalculated = ref(0);\n  const doubled = computed(() => count.value * 2);\n  watch(count, next => { lastCalculated.value = next * 2; }, { immediate: true });\n  return { count, doubled, lastCalculated };\n}\n",
    },
    badExplanation: {
      en: "Evaluating `doubled` writes `lastCalculated`, so reading a computed value also mutates separate state. That couples the side effect to when the lazy getter is read.",
      ja: "`doubled` の評価が `lastCalculated` を書き換え、computed の値を読む操作が別の状態を変更しています。副作用が遅延評価の getter を読むタイミングに依存します。",
    },
    goodExplanation: {
      en: "The getter only returns the derived number. A separate watcher owns the write to `lastCalculated` when `count` changes, including its initial value.",
      ja: "getter は導出した数値だけを返します。`count` の変更時に `lastCalculated` を書く処理は、初期値も含めて別の watcher に任せます。",
    },
  },
  "deep-import": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        "<script setup lang=\"ts\">\nimport { label } from './entry';\n</script>\n\n<template>\n<p>{{ label }}</p>\n</template>\n",
      "value.ts": "export const label = 'Notice';\n",
      "level-one.ts": "export { label } from './level-two';\n",
      "level-two.ts": "export { label } from './level-three';\n",
      "level-three.ts": "export { label } from './value';\n",
      "public-api.ts": "export { label } from './value';\n",
    },
    bad: {
      "entry.ts": "export { label } from './level-one';\n",
    },
    good: {
      "entry.ts": "export { label } from './public-api';\n",
    },
    badExplanation: {
      en: "The entry routes a simple value through `level-one`, `level-two`, and `level-three`, creating an unnecessarily deep import chain for a project that wants a shallow public boundary.",
      ja: "entry が単純な値を `level-one`、`level-two`、`level-three` 経由で読み、浅い公開境界を望むプロジェクトに不要な深い import の列を作っています。",
    },
    goodExplanation: {
      en: "The entry uses `public-api.ts`, which re-exports the value directly. The consumer keeps the same imported name while the chain becomes shorter.",
      ja: "値を直接再公開する `public-api.ts` を entry から使います。使用側の import 名を保ったまま、参照の列を短くします。",
    },
    note: {
      en: "This is an explicitly chosen project layout policy; it does not invent a supported depth threshold or option. There is no current diagnostic producer for this contract.",
      ja: "明示的に選んだプロジェクトの配置方針を示す例です。対応する深さのしきい値やオプションがあると主張するものではありません。この契約には現在の診断生成元がありません。",
    },
  },
  "di-outside-setup": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "theme.ts":
        "import { inject, provide } from 'vue';\nimport type { InjectionKey } from 'vue';\nexport const ThemeKey: InjectionKey<string> = Symbol('theme');\nexport function provideTheme() { provide(ThemeKey, 'dark'); }\nexport function useTheme() { return inject(ThemeKey, 'light'); }\n",
      "ThemedText.vue":
        "<script setup lang=\"ts\">\nimport { useTheme } from './theme';\nconst theme = useTheme();\n</script>\n\n<template>\n<p>{{ theme }}</p>\n</template>\n",
    },
    bad: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\nimport { provideTheme } from './theme';\nprovideTheme();\ncreateApp(App).mount('#app');\n",
      "App.vue":
        "<script setup lang=\"ts\">\nimport ThemedText from './ThemedText.vue';\n</script>\n\n<template>\n<ThemedText />\n</template>\n",
    },
    good: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport ThemedText from './ThemedText.vue';\nimport { provideTheme } from './theme';\nprovideTheme();\n</script>\n\n<template>\n<ThemedText />\n</template>\n",
    },
    badExplanation: {
      en: "`main.ts` calls component `provide` with no active component instance. The child’s `inject` therefore cannot receive this intended ancestor value and uses `light`.",
      ja: "`main.ts` が有効なコンポーネントインスタンスなしで component の `provide` を呼びます。子の `inject` は意図した祖先の値を受け取れず、`light` を使います。",
    },
    goodExplanation: {
      en: "App calls the provider from its setup before rendering the child. The child now inherits the `dark` value from its component ancestor.",
      ja: "App の setup で子を描画する前に provider を呼び、子が祖先コンポーネントの `dark` を受け取れるようにします。",
    },
    note: {
      en: "This example uses component provide/inject. `app.provide` and supported `app.runWithContext` injection are different valid ownership surfaces, not prohibited by this scenario. No current producer emits this contract code.",
      ja: "コンポーネントの provide/inject を使う例です。`app.provide` や対応する `app.runWithContext` 内の inject は別の有効な所属先であり、この例で禁止するものではありません。現在この契約コードの生成元はありません。",
    },
  },
};
