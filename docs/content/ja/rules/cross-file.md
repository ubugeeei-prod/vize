---
title: ファイル間ルール
---

<!-- Generated translation; source: rules/cross-file.md -->

# クロスファイルルール

クロスファイル診断は、`vize lint --cross-file` によって発行されます。彼らは使用します
`vize:croquis/cf/*` 診断コードは、分離されたグラフではなくプロジェクト グラフを分析するためです。
SFC。これらのチェックは、ファイル間情報を必要とする Patina ルールの現在の公開表面です。
キーが次の場合、プロバイダーとインジェクターの値の型の不一致は TypeScript 診断に委ねられます。
`InjectionKey<T>` で宣言されます。

以下の各例は、小さな複数ファイルのフィクスチャとして書かれています。ファイル間の部分は次の関係です。
コンポーネントのインポート、テンプレートの使用、キーの提供/注入、または 1 つのファイルから移動するリアクティブな値
別のものに。 `v-for` 内の ID などのローカル回線を報告するルールは、引き続き次の文書に記載されています。
同じプロジェクト グラフ パス中に診断が発行されるため、この形状になります。

## `vize:croquis/cf/unmatched-inject`

キーが分析対象の到達可能な `provide()` と一致しない `inject()` をレポートします。
成分グラフ。

悪い：

```ts
// keys/theme.ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

```vue
<!-- App.vue -->
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

良い：

```vue
<!-- App.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

## `vize:croquis/cf/unused-provide`

グラフ内で到達可能であるが、一致するインジェクターがない `provide()` を報告します。

悪い：

```vue
<!-- App.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

```vue
<!-- Dashboard.vue -->
<template>
  <h1>Dashboard</h1>
</template>
```

良い：

```vue
<!-- App.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

```vue
<!-- Dashboard.vue -->
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

## `vize:croquis/cf/provide-without-symbol`

文字列キーを使用する `provide()` 呼び出しを報告します。シンボルはファイル間で 1 つのキー ID を保持し、
無関係なプロバイダーとインジェクターの間で偶発的に一致することを避けます。

悪い：

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

良い：

```ts
// keys/theme.ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

## `vize:croquis/cf/inject-without-symbol`

文字列キーを使用する `inject()` 呼び出しを報告します。

悪い：

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

良い：

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

## `vize:croquis/cf/non-reactive-provide`

レポートでは、反応的な値ではなく単純なスナップショットの値が提供されました。 `ref()` または
`computed()` なので、別のファイルのコンシューマーはプロバイダーからの更新を監視します。

悪い：

```ts
// keys/theme.ts
export const ThemeKey = Symbol("theme");
```

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = { color: "blue" };
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

良い：

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

良い：

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { computed, provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const color = ref("blue");
const theme = computed(() => ({ color: color.value }));
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

## `vize:croquis/cf/duplicate-id`

分析されたコンポーネント グラフ全体で重複する静的 ID をレポートします。ルールは、2 つの場合にこれを報告します。
異なるコンポーネントを一緒にレンダリングして、同じ DOM ID を生成できます。

悪い：

```vue
<!-- CheckoutForm.vue -->
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

```vue
<!-- ShippingAddress.vue -->
<template>
  <label for="postal-code">Shipping postal code</label>
  <input id="postal-code" />
</template>
```

```vue
<!-- BillingAddress.vue -->
<template>
  <label for="postal-code">Billing postal code</label>
  <input id="postal-code" />
</template>
```

良い：

```vue
<!-- ShippingAddress.vue -->
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Shipping postal code</label>
  <input :id="postalCodeId" />
</template>
```

```vue
<!-- BillingAddress.vue -->
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Billing postal code</label>
  <input :id="postalCodeId" />
</template>
```

## `vize:croquis/cf/non-unique-id`

繰り返されるテンプレート スコープ内の静的 ID をレポートします。問題のある行はローカルですが、ルールは実行されます
グラフ パス内で、ファイル間の重複 ID もチェックします。

悪い：

```vue
<!-- ResultsList.vue -->
<template>
  <article v-for="result in results" :key="result.id">
    <h2 id="result-title">{{ result.title }}</h2>
  </article>
</template>
```

良い：

```vue
<!-- ResultsList.vue -->
<template>
  <article v-for="result in results" :key="result.id">
    <h2 :id="`result-${result.id}-title`">{{ result.title }}</h2>
  </article>
</template>
```

## `vize:croquis/cf/spread-breaks-reactivity`

レポート オブジェクトは、コンポーネントの境界を越えた後、そのスナップショットの反応状態を拡散します。

悪い：

```vue
<!-- UserPage.vue -->
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
const props = defineProps<{ user: { name: string; role: string } }>();
const copiedUser = { ...props.user };
</script>
```

良い：

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string; role: string } }>();
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/reassignment-breaks-reactivity`

状態がファイル境界を越えた後にプレーンな値に置き換えられるリアクティブな参照をレポートします。

悪い：

```vue
<!-- UserPage.vue -->
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
let user = toRef(props, "user");

user = props.user;
</script>
```

良い：

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/value-extraction-breaks-reactivity`

長期存続するプレーン バインディングにコピーされるリアクティブな値を報告します。ダイレクトリアクティブプロップ
分解は許可されます。問題は、その構造化されたバインディングを別のプレーンに代入することです
バインディング。

悪い：

```vue
<!-- UserPage.vue -->
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
const { item } = defineProps<{ item: { name: string } }>();
const itemSnapshot = item;
</script>
```

良い：

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { computed } from "vue";

const { item } = defineProps<{ item: { name: string } }>();
const itemView = computed(() => item);
</script>
```

## `vize:croquis/cf/destructuring-breaks-reactivity`

Vue のリアクティブ プロパティの destruction でカバーされていないリアクティブ オブジェクトの分割をレポートします。
変身する。

悪い：

```vue
<!-- UserPage.vue -->
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const { item } = props;
</script>
```

良い：

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

## `vize:croquis/cf/hydration-risk`

サーバーとクライアント間でレンダリングが異なる可能性がある値をレポートします。グラフはポイントを助けます
ルートまたは親コンポーネントから、非決定的な値をレンダリングするコンポーネントまで。

悪い：

```vue
<!-- App.vue -->
<script setup lang="ts">
import ClockBadge from "./ClockBadge.vue";
</script>

<template>
  <ClockBadge />
</template>
```

```vue
<!-- ClockBadge.vue -->
<template>
  <time>{{ new Date().toLocaleString() }}</time>
</template>
```

良い：

```vue
<!-- ClockBadge.vue -->
<script setup lang="ts">
const renderedAt = useState("rendered-at", () => new Date().toISOString());
</script>

<template>
  <time :datetime="renderedAt">{{ renderedAt }}</time>
</template>
```

## `vize:croquis/cf/async-boundary`

クリーンアップが登録されていない限り、読み取った状態を超えて存続する可能性がある非同期のリアクティブな作業をレポートします。

悪い：

```vue
<!-- SearchPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

```vue
<!-- SearchResults.vue -->
<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value) => {
    result.value = await load(value);
  },
);
</script>
```

良い：

```vue
<!-- SearchResults.vue -->
<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

## `vize:croquis/cf/watcheffect-async`

依存関係の収集と非同期作業を混在させる `watchEffect` コールバックをレポートします。明示的なものを使用する
ソースに `watch()` が設定されているため、無効化によって古いリクエストをキャンセルできます。

悪い：

```vue
<!-- SearchPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

```vue
<!-- SearchResults.vue -->
<script setup lang="ts">
import { ref, watchEffect } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watchEffect(async () => {
  result.value = await load(props.query);
});
</script>
```

良い：

```vue
<!-- SearchResults.vue -->
<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

## `vize:croquis/cf/injected-async-mutation-race`

プロバイダーまたは兄弟インジェクターと競合する可能性がある、注入された状態への非同期変異を報告します。しましょう
プロバイダーは共有ミューテーションを所有するか、明示的なイベント/アクションをプロバイダーに渡します。

悪い：

```ts
// keys/store.ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

```vue
<!-- StoreProvider.vue -->
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);
</script>

<template>
  <CountLoader />
  <CountSummary />
</template>
```

```vue
<!-- CountLoader.vue -->
<script setup lang="ts">
import { inject, ref, watch } from "vue";
import { StoreKey } from "./keys/store";

const store = inject(StoreKey)!;
const query = ref("");

watch(query, async (value) => {
  store.count = await loadCount(value);
});
</script>
```

良い：

```vue
<!-- StoreProvider.vue -->
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);

function applyLoadedCount(count: number) {
  store.count = count;
}
</script>

<template>
  <CountLoader @loaded="applyLoadedCount" />
  <CountSummary />
</template>
```

```vue
<!-- CountLoader.vue -->
<script setup lang="ts">
import { ref, watch } from "vue";

const emit = defineEmits<{ loaded: [count: number] }>();
const query = ref("");

watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;

  onCleanup(() => {
    active = false;
    controller.abort();
  });

  const count = await loadCount(value, { signal: controller.signal });
  if (active) emit("loaded", count);
});
</script>
```

## 実装の方向性

クロスファイル エンジンは、使用しているにもかかわらず、ルールとして意図的に文書化されています。
今日の診断コード。将来の作業では、必要に応じて、より多くの Patina ルールをこのレイヤーにプロモートできるようになります。
インポート、コンポーネントの関係、またはプロジェクト全体のシンボル ID を使用して、問題を正確に説明します。
