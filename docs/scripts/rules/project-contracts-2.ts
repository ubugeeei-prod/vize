import type { ContractExample } from "./types.ts";
export const contract2Examples: Record<string, ContractExample> = {
  "pinia-getter": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport { createPinia } from 'pinia';\nimport App from './App.vue';\ncreateApp(App).use(createPinia()).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "counter-store.ts":
        "import { defineStore } from 'pinia';\nexport const useCounterStore = defineStore('counter', {\n  state: () => ({ count: 0 }),\n  getters: { doubled: state => state.count * 2 },\n});\n",
    },
    bad: {
      "App.vue":
        '<script setup lang="ts">\nimport { useCounterStore } from \'./counter-store\';\nconst store = useCounterStore();\nconst doubled = store.doubled;\n</script>\n\n<template>\n<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>\n</template>\n',
    },
    good: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport { storeToRefs } from 'pinia';\nimport { useCounterStore } from './counter-store';\nconst store = useCounterStore();\nconst { doubled } = storeToRefs(store);\n</script>\n\n<template>\n<button @click=\"store.count++\">{{ store.count }}</button><p>{{ doubled }}</p>\n</template>\n",
    },
    badExplanation: {
      en: "`const doubled = store.doubled` copies the getter’s current number during setup. The copied number does not follow later `store.count` updates.",
      ja: "`const doubled = store.doubled` が setup 時点の getter の数値をコピーし、後の `store.count` 更新に追従しなくなります。",
    },
    goodExplanation: {
      en: "`storeToRefs(store)` supplies a reactive getter ref that can be destructured and unwrapped by the template while staying connected to the store.",
      ja: "`storeToRefs(store)` からリアクティブな getter の ref を受け取り、分割代入とテンプレートの unwrap 後も store とのつながりを保ちます。",
    },
    note: {
      en: "Pinia must be installed, and main.ts installs its plugin before mounting. Reading `store.doubled` directly inside a tracked computation or template is valid; the defect here is taking a plain snapshot. This contract currently has no producer.",
      ja: "Pinia のインストールを前提とし、main.ts がマウント前に plugin を登録します。追跡される計算やテンプレートの中で `store.doubled` を直接読むことは有効です。ここでの問題は通常の値としてスナップショットを取ることであり、現在この契約の生成元はありません。",
    },
  },
  "reactive-export": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
    },
    bad: {
      "state.ts": "import { reactive } from 'vue';\nexport const state = reactive({ count: 0 });\n",
      "App.vue":
        '<script setup lang="ts">\nimport { state } from \'./state\';\n</script>\n\n<template>\n<button @click="state.count++">{{ state.count }}</button>\n</template>\n',
    },
    good: {
      "state.ts":
        "import { reactive } from 'vue';\nexport function createState() { return reactive({ count: 0 }); }\n",
      "App.vue":
        '<script setup lang="ts">\nimport { createState } from \'./state\';\nconst state = createState();\n</script>\n\n<template>\n<button @click="state.count++">{{ state.count }}</button>\n</template>\n',
    },
    badExplanation: {
      en: "The module exports one initialized reactive object, so every importer receives the same count. In an SSR module shared between requests, this defeats the example’s per-instance/request state isolation.",
      ja: "モジュールが初期化済みのリアクティブなオブジェクトを一つだけ公開し、すべての import 元が同じ count を受け取ります。リクエスト間でモジュールを共有する SSR では、この例のインスタンスやリクエストごとの分離を保てません。",
    },
    goodExplanation: {
      en: "The module exports a factory, and App invokes it inside setup. Each instance obtains a fresh reactive count rather than the exported singleton.",
      ja: "モジュールから factory を公開し、App が setup 内で呼びます。公開された singleton ではなく、インスタンスごとに新しいリアクティブな count を得ます。",
    },
    note: {
      en: "Intentional shared application stores may export reactive state. This scenario requires isolated state and does not claim every reactive export is invalid. No current producer emits this contract.",
      ja: "共有するアプリケーションの store を意図する場合、リアクティブな状態の export は有効です。この例は状態の分離を必要とするもので、あらゆる export が不正という意味ではありません。現在この契約の生成元はありません。",
    },
  },
  "reactivity-outside-setup": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        "<script setup lang=\"ts\">\nimport Counter from './Counter.vue';\n</script>\n\n<template>\n<Counter /><Counter />\n</template>\n",
      "Counter.vue":
        '<script setup lang="ts">\nimport { useCounter } from \'./use-counter\';\nconst { count, doubled } = useCounter();\n</script>\n\n<template>\n<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>\n</template>\n',
    },
    bad: {
      "use-counter.ts":
        "import { computed, ref } from 'vue';\nconst count = ref(0);\nconst doubled = computed(() => count.value * 2);\nexport function useCounter() { return { count, doubled }; }\n",
    },
    good: {
      "use-counter.ts":
        "import { computed, ref } from 'vue';\nexport function useCounter() {\n  const count = ref(0);\n  const doubled = computed(() => count.value * 2);\n  return { count, doubled };\n}\n",
    },
    badExplanation: {
      en: "Both reactive APIs run while the module loads. The two Counter instances therefore share one ref and computed value despite the intended independent counters.",
      ja: "二つのリアクティブ API がモジュールの読み込み時に実行されます。独立したカウンターを意図していても、二つの Counter が一つの ref と computed を共有します。",
    },
    goodExplanation: {
      en: "`useCounter` creates the ref and computed synchronously inside each component setup call, giving each widget its own state and tracked derivation.",
      ja: "各コンポーネントの setup 呼び出しの中で、`useCounter` が ref と computed を同期的に作り、各 widget に個別の状態と追跡される導出を与えます。",
    },
    note: {
      en: "Vue permits ref/reactive/computed outside component setup. The risk here is unwanted ownership/sharing under an explicit instance-isolation policy, not API illegality. This contract has no current producer.",
      ja: "Vue では component setup の外でも ref・reactive・computed を使えます。ここでのリスクは API の禁止ではなく、インスタンスを分離する明示的な方針に反した共有や所属です。この契約には現在の生成元がありません。",
    },
  },
  "reference-escapes-scope": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "saved.ts":
        "import type { Ref } from 'vue';\nlet saved: Ref<number> | number | undefined;\nexport function remember(value: Ref<number> | number): void { saved = value; }\nexport function remembered(): Ref<number> | number | undefined { return saved; }\n",
    },
    bad: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport { ref } from 'vue';\nimport { remember } from './saved';\nconst count = ref(0);\nremember(count);\n</script>\n\n<template>\n<button @click=\"count++\">{{ count }}</button>\n</template>\n",
    },
    good: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport { ref } from 'vue';\nimport { remember } from './saved';\nconst count = ref(0);\nremember(count.value);\n</script>\n\n<template>\n<button @click=\"count++\">{{ count }}</button>\n</template>\n",
    },
    badExplanation: {
      en: "The process-level cache retains the component’s live count ref. It can keep that instance state reachable after unmount and can observe later edits, although this cache is intended to store a snapshot.",
      ja: "プロセス全体の cache がコンポーネントの count ref をそのまま保持し、アンマウント後もインスタンスの状態を参照できるようにします。スナップショットを保存する意図にもかかわらず、後の変更も観測できます。",
    },
    goodExplanation: {
      en: "The cache receives the current plain number, so it keeps a snapshot without retaining the component-owned ref.",
      ja: "cache に現在の通常の数値を渡し、コンポーネントが持つ ref を保持せずスナップショットを保存します。",
    },
    note: {
      en: "Refs may legitimately be returned from composables or shared across scopes. This example explicitly requires a snapshot cache; it does not claim that unmount invalidates a ref. No current producer emits this contract.",
      ja: "ref を composable から返したり、スコープをまたいで共有したりすることは有効です。この例では cache にスナップショットを求めており、アンマウントで ref 自体が無効になると主張するものではありません。現在この契約の生成元はありません。",
    },
  },
  "shallow-deep-access": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        "<script setup lang=\"ts\">\nimport { makeProfile } from './profile';\nconst profile = makeProfile();\n</script>\n\n<template>\n<p>{{ profile.user.name }}</p><button @click=\"profile.user.name = 'Grace'\">Rename</button>\n</template>\n",
    },
    bad: {
      "profile.ts":
        "import { shallowReactive } from 'vue';\nexport function makeProfile() { return shallowReactive({ user: { name: 'Ada' } }); }\n",
    },
    good: {
      "profile.ts":
        "import { reactive } from 'vue';\nexport function makeProfile() { return reactive({ user: { name: 'Ada' } }); }\n",
    },
    badExplanation: {
      en: "`shallowReactive` tracks the root `user` property but leaves the nested object raw. Changing `profile.user.name` does not notify the template as a tracked deep mutation.",
      ja: "`shallowReactive` が追跡するのはルートの `user` プロパティであり、内側のオブジェクトは raw のままです。`profile.user.name` の変更は、追跡された深い変更としてテンプレートへ通知されません。",
    },
    goodExplanation: {
      en: "Deep `reactive` wraps the nested user object, so the same name assignment can trigger the displayed name’s update.",
      ja: "深い `reactive` で内側の user オブジェクトも包み、同じ名前の代入を表示の更新につなげます。",
    },
  },
  "suspense-no-fallback": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "AsyncCard.vue":
        "<script setup lang=\"ts\">\nconst message = await Promise.resolve('Ready');\n</script>\n\n<template>\n<p>{{ message }}</p>\n</template>\n",
    },
    bad: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport { Suspense } from 'vue';\nimport AsyncCard from './AsyncCard.vue';\n</script>\n\n<template>\n<Suspense><AsyncCard /></Suspense>\n</template>\n",
    },
    good: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport { Suspense } from 'vue';\nimport AsyncCard from './AsyncCard.vue';\n</script>\n\n<template>\n<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>\n</template>\n",
    },
    badExplanation: {
      en: "The Suspense boundary has an async child but no fallback content, leaving no loading content for this example’s pending state.",
      ja: "Suspense の境界に非同期の子がありますが fallback がなく、この例の待機中の状態に表示する内容がありません。",
    },
    goodExplanation: {
      en: "The `#fallback` slot supplies an explicit loading paragraph until the async child resolves.",
      ja: "`#fallback` slot にローディングの段落を指定し、非同期の子が完了するまで表示します。",
    },
    note: {
      en: "Suspense without a fallback is valid Vue syntax. This is a chosen loading-UI convention, not a compiler error; the published contract has no current producer.",
      ja: "fallback のない Suspense は有効な Vue の構文です。ローディング UI を置くという方針の例であり、コンパイルエラーではありません。この公開契約には現在の生成元がありません。",
    },
  },
  "template-ref-timing": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "focus-input.ts":
        "export function focusInput(input: HTMLInputElement | null): void { input?.focus(); }\n",
    },
    bad: {
      "App.vue":
        '<script setup lang="ts">\nimport { ref } from \'vue\';\nimport { focusInput } from \'./focus-input\';\nconst input = ref<HTMLInputElement | null>(null);\nfocusInput(input.value);\n</script>\n\n<template>\n<input ref="input" aria-label="Name" />\n</template>\n',
    },
    good: {
      "App.vue":
        '<script setup lang="ts">\nimport { onMounted, ref } from \'vue\';\nimport { focusInput } from \'./focus-input\';\nconst input = ref<HTMLInputElement | null>(null);\nonMounted(() => { focusInput(input.value); });\n</script>\n\n<template>\n<input ref="input" aria-label="Name" />\n</template>\n',
    },
    badExplanation: {
      en: "Setup reads the template ref before mounting, when its value is still null. The optional focus call therefore performs no focus action.",
      ja: "マウント前の setup でテンプレート ref を読み、値が null のままです。任意の focus 呼び出しは何もフォーカスしません。",
    },
    goodExplanation: {
      en: "`onMounted` defers the read until Vue has assigned the input element to the template ref, allowing the focus helper to act on it.",
      ja: "`onMounted` で Vue が input 要素をテンプレート ref に設定するまで参照を遅らせ、ヘルパーが実際の要素をフォーカスできるようにします。",
    },
  },
};
