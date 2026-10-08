export const contract1Examples = {
  "dom-access-without-next-tick": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "read-label.ts":
        "export function readLabel(node: HTMLElement | null): string {\n  return node?.textContent ?? '';\n}\n",
    },
    bad: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport { ref } from 'vue';\nimport { readLabel } from './read-label';\nconst count = ref(0);\nconst label = ref<HTMLElement | null>(null);\nconst sampled = ref('');\nfunction increment() {\n  count.value++;\n  sampled.value = readLabel(label.value);\n}\n</script>\n\n<template>\n<button @click=\"increment\">Increment</button><p ref=\"label\">{{ count }}</p><p>DOM sample: {{ sampled }}</p>\n</template>\n",
    },
    good: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport { nextTick, ref } from 'vue';\nimport { readLabel } from './read-label';\nconst count = ref(0);\nconst label = ref<HTMLElement | null>(null);\nconst sampled = ref('');\nasync function increment() {\n  count.value++;\n  await nextTick();\n  sampled.value = readLabel(label.value);\n}\n</script>\n\n<template>\n<button @click=\"increment\">Increment</button><p ref=\"label\">{{ count }}</p><p>DOM sample: {{ sampled }}</p>\n</template>\n",
    },
    badExplanation: {
      en: "The click handler increments `count` and immediately reads the rendered paragraph, before Vue flushes the scheduled DOM update. `sampled` can contain the previous count.",
      ja: "クリック処理が `count` を増やしてすぐ段落の DOM を読み、Vue による更新の反映を待っていません。`sampled` に前の count が入る可能性があります。",
    },
    goodExplanation: {
      en: "Awaiting `nextTick()` after the state write lets Vue update the paragraph before `readLabel` samples its text.",
      ja: "状態を書いた後に `nextTick()` を待ち、Vue が段落を更新してから `readLabel` でテキストを読みます。",
    },
  },
  "event-listener-leak": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        "<script setup lang=\"ts\">\nimport { useWidth } from './use-width';\nconst width = useWidth();\n</script>\n\n<template>\n<p>{{ width }}</p>\n</template>\n",
    },
    bad: {
      "use-width.ts":
        "import { onMounted, ref } from 'vue';\nexport function useWidth() {\n  const width = ref(0);\n  const resize = () => { width.value = window.innerWidth; };\n  onMounted(() => { resize(); window.addEventListener('resize', resize); });\n  return width;\n}\n",
    },
    good: {
      "use-width.ts":
        "import { onMounted, onUnmounted, ref } from 'vue';\nexport function useWidth() {\n  const width = ref(0);\n  const resize = () => { width.value = window.innerWidth; };\n  onMounted(() => { resize(); window.addEventListener('resize', resize); });\n  onUnmounted(() => { window.removeEventListener('resize', resize); });\n  return width;\n}\n",
    },
    badExplanation: {
      en: "Mounting adds a window resize listener that captures the component’s width ref, but unmounting never removes it. Repeated mounts can retain unused listeners and state.",
      ja: "マウント時に width の ref を参照する resize リスナーを window に追加しますが、アンマウント時に削除しません。再マウントを繰り返すと不要なリスナーや状態を保持し得ます。",
    },
    goodExplanation: {
      en: "`onUnmounted` removes the exact same `resize` function registered at mount, ending that instance’s external listener lifetime.",
      ja: "`onUnmounted` でマウント時に登録した同じ `resize` 関数を削除し、そのインスタンスの外部リスナーの寿命を終えます。",
    },
  },
  "lifecycle-outside-setup": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "install-title.ts":
        "import { onMounted } from 'vue';\nexport function installTitle() {\n  onMounted(() => { document.title = 'Mounted application'; });\n}\n",
    },
    bad: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\nimport { installTitle } from './install-title';\ninstallTitle();\ncreateApp(App).mount('#app');\n",
      "App.vue":
        '<script setup lang="ts">\n\n</script>\n\n<template>\n<p>Application</p>\n</template>\n',
    },
    good: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport { installTitle } from './install-title';\ninstallTitle();\n</script>\n\n<template>\n<p>Application</p>\n</template>\n",
    },
    badExplanation: {
      en: "The entry calls `installTitle()` before mounting an app, so `onMounted` is registered without an active component setup context.",
      ja: "entry がアプリのマウント前に `installTitle()` を呼び、有効なコンポーネントの setup なしで `onMounted` を登録しています。",
    },
    goodExplanation: {
      en: "Calling the same helper synchronously from App’s setup attaches the lifecycle callback to that instance’s mount.",
      ja: "同じヘルパーを App の setup から同期的に呼び、ライフサイクルのコールバックをそのインスタンスのマウントに結び付けます。",
    },
  },
  "missing-suspense": {
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
        "<script setup lang=\"ts\">\nimport AsyncCard from './AsyncCard.vue';\n</script>\n\n<template>\n<AsyncCard />\n</template>\n",
    },
    good: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport { Suspense } from 'vue';\nimport AsyncCard from './AsyncCard.vue';\n</script>\n\n<template>\n<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>\n</template>\n",
    },
    badExplanation: {
      en: "`AsyncCard` has top-level await, making its setup asynchronous, but App renders it without a Suspense boundary to coordinate that dependency.",
      ja: "`AsyncCard` の top-level await が setup を非同期にしますが、App はその依存を調整する Suspense の境界なしで描画しています。",
    },
    goodExplanation: {
      en: "App wraps the async child in `Suspense` and supplies a loading fallback until the child setup resolves.",
      ja: "App が非同期の子を `Suspense` で包み、子の setup が完了するまでローディングの fallback を表示します。",
    },
  },
  "module-scope-reactive": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        "<script setup lang=\"ts\">\nimport Counter from './Counter.vue';\n</script>\n\n<template>\n<Counter /><Counter />\n</template>\n",
      "Counter.vue":
        '<script setup lang="ts">\nimport { createCounter } from \'./counter\';\nconst { count } = createCounter();\n</script>\n\n<template>\n<button @click="count++">{{ count }}</button>\n</template>\n',
    },
    bad: {
      "counter.ts":
        "import { ref } from 'vue';\nconst count = ref(0);\nexport function createCounter() { return { count }; }\n",
    },
    good: {
      "counter.ts":
        "import { ref } from 'vue';\nexport function createCounter() {\n  const count = ref(0);\n  return { count };\n}\n",
    },
    badExplanation: {
      en: "The module initializes `count` once, and both Counter instances receive the same ref. Clicking one changes both counters even though this example intends independent instance state.",
      ja: "モジュールが `count` を一度だけ初期化し、二つの Counter が同じ ref を受け取ります。この例では独立した状態を意図していますが、一方をクリックすると両方が変わります。",
    },
    goodExplanation: {
      en: "Creating the ref inside `createCounter` gives each synchronous setup call a separate state object, so each button owns its counter.",
      ja: "`createCounter` の中で ref を作り、setup からの同期的な呼び出しごとに別の状態を渡します。それぞれのボタンが個別のカウンターを持つようになります。",
    },
    note: {
      en: "Module-scope reactive state is legal for intentional application stores. This example assumes component/request isolation; the published contract currently has no producer.",
      ja: "アプリケーション全体の store を意図する場合、モジュール直下のリアクティブ状態は有効です。この例はコンポーネントやリクエスト間の分離を前提とし、この公開契約には現在の生成元がありません。",
    },
  },
  "mutated-after-escape": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "archive.ts":
        "export interface Profile { name: string }\nconst records: Readonly<Profile>[] = [];\nexport function publish(profile: Readonly<Profile>): void { records.push(profile); }\nexport function latestName(): string { return records.at(-1)?.name ?? ''; }\n",
      "App.vue":
        "<script setup lang=\"ts\">\nimport { publishProfile } from './profile';\nimport { latestName } from './archive';\npublishProfile();\nconst archivedName = latestName();\n</script>\n\n<template>\n<p>Archived name: {{ archivedName }}</p>\n</template>\n",
    },
    bad: {
      "profile.ts":
        "import { reactive } from 'vue';\nimport { publish } from './archive';\nexport function publishProfile(): void {\n  const profile = reactive({ name: 'Ada' });\n  publish(profile);\n  profile.name = 'Grace';\n}\n",
    },
    good: {
      "profile.ts":
        "import { reactive } from 'vue';\nimport { publish } from './archive';\nexport function publishProfile(): void {\n  const profile = reactive({ name: 'Ada' });\n  publish({ ...profile });\n  profile.name = 'Grace';\n}\n",
    },
    badExplanation: {
      en: "The archive retains the same object passed to `publish`. The owner then changes its name, retroactively changing the supposedly historical record to Grace. TypeScript’s Readonly parameter does not copy the object.",
      ja: "archive は `publish` に渡した同じオブジェクトを保持します。その後で所有側が名前を変更し、過去の記録まで Grace に変わります。TypeScript の Readonly 引数はオブジェクトをコピーしません。",
    },
    goodExplanation: {
      en: "Publishing a plain copy separates the archived Ada record from later edits of the reactive profile. The archive’s snapshot policy is now maintained.",
      ja: "通常のコピーを公開し、保存した Ada の記録と後のリアクティブな profile の変更を分離します。archive のスナップショット方針を保てるようになります。",
    },
    note: {
      en: "This is an explicit immutable-history ownership policy, not a general prohibition on passing or later mutating reactive objects. No current producer emits this contract.",
      ja: "変更しない履歴という明示的な所有方針を示す例です。リアクティブなオブジェクトを渡したり、後で変更したりすることを一般に禁止するものではありません。現在この契約の生成元はありません。",
    },
  },
  "object-identity-comparison": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "user.ts":
        "import { reactive } from 'vue';\nexport function makeUser() {\n  const raw = { id: 7, name: 'Ada' };\n  return { raw, proxy: reactive(raw) };\n}\n",
    },
    bad: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport { makeUser } from './user';\nconst { raw, proxy } = makeUser();\nconst sameRecord = proxy === raw;\n</script>\n\n<template>\n<p>Same record: {{ sameRecord }}</p>\n</template>\n",
    },
    good: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport { makeUser } from './user';\nconst { raw, proxy } = makeUser();\nconst sameRecord = proxy.id === raw.id;\n</script>\n\n<template>\n<p>Same record: {{ sameRecord }}</p>\n</template>\n",
    },
    badExplanation: {
      en: "`proxy === raw` compares wrapper identity, so it is false even though both represent the same user record. The application intended record identity, not object-wrapper identity.",
      ja: "`proxy === raw` はラッパーの同一性を比較するため、同じユーザーを表していても false になります。アプリが意図するレコードの同一性と、オブジェクトの同一性が異なっています。",
    },
    goodExplanation: {
      en: "Comparing the stable record `id` answers the intended question without depending on whether the object is raw or proxied.",
      ja: "安定したレコードの `id` を比較し、raw か Proxy かに依存せず意図した同一性を判定します。",
    },
    note: {
      en: "The example assumes IDs uniquely identify records. Comparing two references to the same reactive proxy remains valid; this contract has no current producer.",
      ja: "ID がレコードを一意に識別することを前提とします。同じリアクティブ Proxy の参照同士を比較することは有効であり、この契約には現在の生成元がありません。",
    },
  },
};
