---
title: Règles croisées
---

<!-- Generated translation; source: rules/cross-file.md -->

# Règles croisées

Les diagnostics croisés sont émis par `vize lint --cross-file`. Ils utilisent
`vize:croquis/cf/*` codes de diagnostic car ils analysent un graphe de projet plutôt qu’un graphique isolé
SFC. Ces vérifications constituent actuellement la surface publique des règles de Patina qui nécessitent des informations à classement croisé.
Les incompatibilités de type de valeur fournisseur et injecteur sont laissées aux diagnostics TypeScript lorsque la clé est
déclarée avec `InjectionKey<T>`.

Chaque exemple ci-dessous est écrit sous forme d’un minuscule fixture multi-fichiers. La partie cross-file est la relation :
importations de composants, utilisation de modèles, clés de fournisseur/injectation, ou valeurs réactives qui passent d’un fichier
à un autre. Les règles qui rapportent une ligne locale, comme un ID à l’intérieur de `v-for`, sont toujours documentées dans
cette forme car le diagnostic est émis lors du même passage de graphe de projet.

## `vize:croquis/cf/unmatched-inject`

Rapporte un `inject()` dont la clé ne peut être associée à un `provide()` accessible dans le graphique
composant analysé.

Mauvais :

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

Bon :

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

Rapporte un `provide()` accessible dans le graphique mais sans injecteur correspondant.

Mauvais :

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

Bon :

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

Signale `provide()` appels utilisant des clés de cordes. Les symboles préservent une identité clé entre les fichiers et
éviter les correspondances accidentelles entre fournisseurs et injecteurs non apparentés.

Mauvais :

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

Bon :

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

Signale `inject()` appels utilisant des clés de cordes.

Mauvais :

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

Bon :

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

Les rapports fournissaient des valeurs qui sont de simples instantanés au lieu de valeurs réactives. Privilégiez `ref()` ou
`computed()` pour que les consommateurs d’un autre fichier suivent les mises à jour du fournisseur.

Mauvais :

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

Bon :

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

Bon :

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

Rapporte des identifiants statiques dupliqués sur le graphe des composants analysés. La règle indique cela lorsque deux composants
différents peuvent être rendus ensemble et produire le même ID DOM.

Mauvais :

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

Bon :

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

Signale des identifiants statiques à l’intérieur de champs de vision répétés. La ligne problématique est locale, mais la règle s’exécute
à l’intérieur du passage graphique qui vérifie aussi les identifiants doublés entre fichiers.

Mauvais :

```vue
<!-- ResultsList.vue -->
<template>
  <article v-for="result in results" :key="result.id">
    <h2 id="result-title">{{ result.title }}</h2>
  </article>
</template>
```

Bon :

```vue
<!-- ResultsList.vue -->
<template>
  <article v-for="result in results" :key="result.id">
    <h2 :id="`result-${result.id}-title`">{{ result.title }}</h2>
  </article>
</template>
```

## `vize:croquis/cf/spread-breaks-reactivity`

Signale les étalements de l’objet de cet état réactif instantané après avoir franchi une frontière de composant.

Mauvais :

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

Bon :

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string; role: string } }>();
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/reassignment-breaks-reactivity`

Signale des références réactives qui sont remplacées par des valeurs simples après que l’état a franchi la limite d’un fichier.

Mauvais :

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

Bon :

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/value-extraction-breaks-reactivity`

Rapporte une valeur réactive qui est copiée dans une reliure simple à longue durée. Les hélices réactives directes
déstructuration sont autorisées ; Le problème est d’assigner cette liaison déstructurée à une autre liaison
simple.

Mauvais :

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

Bon :

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { computed } from "vue";

const { item } = defineProps<{ item: { name: string } }>();
const itemView = computed(() => item);
</script>
```

## `vize:croquis/cf/destructuring-breaks-reactivity`

Rapporte la déstructuration d’objets réactifs non couverts par la déstructuration
transformation des hélices réactives de Vue.

Mauvais :

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

Bon :

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

## `vize:croquis/cf/hydration-risk`

Rapporte des valeurs qui peuvent être rendues différemment entre le serveur et le client. Le graphe aide à pointer
de la composante de route ou parent vers la composante qui rend la valeur non déterministe.

Mauvais :

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

Bon :

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

Signale un travail réactif asynchrone qui peut survivre à l’état qu’il lit à moins que le nettoyage ne soit enregistré.

Mauvais :

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

Bon :

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

Rapports `watchEffect` rappels qui mélangent la collecte de dépendances avec le travail asynchrone. Utilisez une source de
explicite avec `watch()` afin que l’invalidation puisse annuler les requêtes obsolètes.

Mauvais :

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

Bon :

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

Signale des mutations asynchrones à l’état injecté qui peuvent s’entraîner avec le fournisseur ou les injecteurs frères. Prenons
le fournisseur possède la mutation partagée, ou transmett-elle un événement/action explicite à celle-ci.

Mauvais :

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

Bon :

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

## Orientation de mise en œuvre

Le moteur cross-file est intentionnellement documenté sous forme de règles, même s’il utilise aujourd’hui
codes de diagnostic. Les travaux futurs peuvent promouvoir davantage de règles de patine dans cette couche lorsqu’elles nécessitent des importations
, des relations entre composants ou l’identité symbolique à l’échelle du projet pour expliquer un problème avec précision.
