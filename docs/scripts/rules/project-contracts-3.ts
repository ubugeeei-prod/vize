export const contract3Examples = {
  "toraw-mutation": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        '<script setup lang="ts">\nimport { makeProfile, rename } from \'./profile\';\nconst profile = makeProfile();\n</script>\n\n<template>\n<p>{{ profile.name }}</p><button @click="rename(profile)">Rename</button>\n</template>\n',
    },
    bad: {
      "profile.ts":
        "import { reactive, toRaw } from 'vue';\nexport function makeProfile() { return reactive({ name: 'Ada' }); }\nexport function rename(profile: { name: string }): void {\n  const raw = toRaw(profile);\n  raw.name = 'Grace';\n}\n",
    },
    good: {
      "profile.ts":
        "import { reactive } from 'vue';\nexport function makeProfile() { return reactive({ name: 'Ada' }); }\nexport function rename(profile: { name: string }): void {\n  profile.name = 'Grace';\n}\n",
    },
    badExplanation: {
      en: "`rename` obtains the raw target and writes `raw.name`, bypassing the proxy setter that would notify the displayed reactive name.",
      ja: "`rename` が raw の対象を取り出して `raw.name` に代入し、表示中の名前を通知する Proxy の setter を通していません。",
    },
    goodExplanation: {
      en: "Writing `profile.name` through the passed reactive proxy preserves the same rename while notifying its dependents.",
      ja: "渡されたリアクティブ Proxy の `profile.name` を書き、同じ名前の変更を依存先に通知します。",
    },
  },
  "undefined-slot": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "Card.vue":
        '<script setup lang="ts">\ndefineSlots<{ header(): unknown }>();\n</script>\n\n<template>\n<article><header><slot name="header" /></header></article>\n</template>\n',
    },
    bad: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport Card from './Card.vue';\n</script>\n\n<template>\n<Card><template #footer>Notice</template></Card>\n</template>\n",
    },
    good: {
      "App.vue":
        "<script setup lang=\"ts\">\nimport Card from './Card.vue';\n</script>\n\n<template>\n<Card><template #header>Notice</template></Card>\n</template>\n",
    },
    badExplanation: {
      en: "App supplies a `footer` slot, but Card declares and renders only `header`. The supplied Notice content has no matching slot outlet in this child.",
      ja: "App が `footer` slot を渡しますが、Card が宣言して描画するのは `header` だけです。渡した Notice に対応する子の slot 出口がありません。",
    },
    goodExplanation: {
      en: "App supplies `header`, matching both the child’s typed slot declaration and its rendered outlet, so Notice appears there.",
      ja: "子の型付き slot 宣言と描画する出口に合わせて `header` を渡し、Notice をそこへ表示できるようにします。",
    },
  },
  "watch-can-be-computed": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        '<script setup lang="ts">\nimport { useDouble } from \'./use-double\';\nconst { count, doubled } = useDouble();\n</script>\n\n<template>\n<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>\n</template>\n',
    },
    bad: {
      "use-double.ts":
        "import { ref, watch } from 'vue';\nexport function useDouble() {\n  const count = ref(0);\n  const doubled = ref(0);\n  watch(count, next => { doubled.value = next * 2; }, { immediate: true });\n  return { count, doubled };\n}\n",
    },
    good: {
      "use-double.ts":
        "import { computed, ref } from 'vue';\nexport function useDouble() {\n  const count = ref(0);\n  const doubled = computed(() => count.value * 2);\n  return { count, doubled };\n}\n",
    },
    badExplanation: {
      en: "The watcher performs no external effect; it only keeps a second writable ref synchronized with twice `count`. This example has no independent writes to that derived value.",
      ja: "watcher は外部への副作用を行わず、二つ目の書き込み可能な ref を `count` の二倍へ同期するだけです。この例に派生値への独立した書き込みはありません。",
    },
    goodExplanation: {
      en: "A computed getter expresses the same derivation directly and removes the manual synchronization and extra writable state.",
      ja: "computed の getter で同じ導出を直接表し、手作業の同期と追加の書き込み可能な状態を取り除きます。",
    },
    note: {
      en: "This illustrates the published preference for purely derived state. Watchers remain appropriate for external effects or independently writable state; no current producer emits this contract.",
      ja: "純粋な派生状態についての公開された推奨を示す例です。外部への副作用や独立して書き込む状態には watcher が適しており、現在この契約の生成元はありません。",
    },
  },
  "watcher-outside-setup": {
    shared: {
      "main.ts":
        "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n",
      "index.html":
        '<!doctype html>\n<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>\n<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>\n',
      "App.vue":
        "<script setup lang=\"ts\">\nimport Observer from './Observer.vue';\n</script>\n\n<template>\n<Observer /><Observer />\n</template>\n",
      "Observer.vue":
        '<script setup lang="ts">\nimport { useObserver } from \'./use-observer\';\nconst { count, observed } = useObserver();\n</script>\n\n<template>\n<button @click="count++">{{ count }}</button><p>{{ observed }}</p>\n</template>\n',
    },
    bad: {
      "use-observer.ts":
        "import { ref, watch } from 'vue';\nconst count = ref(0);\nconst observed = ref(0);\nwatch(count, next => { observed.value = next; });\nexport function useObserver() { return { count, observed }; }\n",
    },
    good: {
      "use-observer.ts":
        "import { ref, watch } from 'vue';\nexport function useObserver() {\n  const count = ref(0);\n  const observed = ref(0);\n  watch(count, next => { observed.value = next; });\n  return { count, observed };\n}\n",
    },
    badExplanation: {
      en: "The watcher is created at module load, outside either Observer’s setup, and both instances share its refs. It is not automatically stopped when a particular Observer unmounts.",
      ja: "watcher をいずれの Observer の setup 内でもなくモジュールの読み込み時に作り、両インスタンスが ref を共有します。個々の Observer のアンマウントでは自動停止されません。",
    },
    goodExplanation: {
      en: "Each synchronous setup call creates its own refs and watcher inside `useObserver`. Vue associates that watcher with the calling component’s lifetime.",
      ja: "setup からの同期的な呼び出しごとに、`useObserver` 内で個別の ref と watcher を作ります。Vue が watcher を呼び出し元のコンポーネントの寿命に結び付けます。",
    },
    note: {
      en: "Module-scope watchers are valid when their owner keeps and calls a stop handle or intentionally gives them application lifetime. This example requires component-owned lifetimes; the contract has no current producer.",
      ja: "所有側が stop 関数を保持して呼んだり、アプリケーション全体の寿命を意図したりする場合、モジュール直下の watcher は有効です。この例はコンポーネントが寿命を持つことを前提とし、この契約には現在の生成元がありません。",
    },
  },
};
