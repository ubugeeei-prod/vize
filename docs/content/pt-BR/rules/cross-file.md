---
title: Regras de Arquivo Cruzado
---

<!-- Generated translation; source: rules/cross-file.md -->

# Regras de Arquivo Cruzado

Diagnósticos entre arquivos são emitidos por `vize lint --cross-file`. Eles usam códigos diagnósticos
`vize:croquis/cf/*` porque analisam um grafo de projeto, em vez de um isolado
SFC. Essas verificações são a superfície pública atual para as regras de Patina que precisam de informações em arquivos cruzados.
As incompatibilidades entre o valor do provedor e do injetor são deixadas para o diagnóstico TypeScript quando a chave é
declarada com `InjectionKey<T>`.

Cada exemplo abaixo é escrito como um pequeno fixture multi-arquivo. A parte de cruzamento de arquivos é a relação: importações de
componentes, uso de templates, chaves de fornecimento ou injeção ou valores reativos que se movem de um arquivo
para outro. Regras que reportam uma linha local, como um ID dentro de `v-for`, ainda são documentadas em
essa forma porque o diagnóstico é emitido durante a mesma passagem do grafo do projeto.

## `vize:croquis/cf/unmatched-inject`

Reporta um `inject()` cuja chave não pode ser associada a uma `provide()` acessível no grafo de componentes de
analisado.

Ruim:

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

Bom:

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

Reporta um `provide()` que está acessível no gráfico, mas não tem um injetor correspondente.

Ruim:

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

Bom:

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

Relata `provide()` chamadas que usam chaves de corda. Símbolos preservam uma identidade chave entre arquivos e
evitar correspondências acidentais entre provedores e injetores não relacionados.

Ruim:

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

Bom:

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

Relata `inject()` chamadas que usam chaves de corda.

Ruim:

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

Bom:

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

Relatórios forneciam valores que são snapshots simples em vez de valores reativos. Prefere `ref()` ou
`computed()` para que os usuários em outro arquivo vejam as atualizações do provedor.

Ruim:

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

Bom:

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

Bom:

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

Relata IDs estáticos duplicados ao longo do gráfico de componentes analisado. A regra informa isso quando dois
componentes diferentes podem ser renderizados juntos e produzir o mesmo ID DOM.

Ruim:

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

Bom:

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

Reporta IDs estáticos dentro de escopos de template repetidos. A linha problemática é local, mas a regra roda
dentro do passe de grafo que também verifica IDs duplicados entre arquivos.

Ruim:

```vue
<!-- ResultsList.vue -->
<template>
  <article v-for="result in results" :key="result.id">
    <h2 id="result-title">{{ result.title }}</h2>
  </article>
</template>
```

Bom:

```vue
<!-- ResultsList.vue -->
<template>
  <article v-for="result in results" :key="result.id">
    <h2 :id="`result-${result.id}-title`">{{ result.title }}</h2>
  </article>
</template>
```

## `vize:croquis/cf/spread-breaks-reactivity`

Reporta as propagações de objetos que capturam o estado reativo após cruzar um limite de componente.

Ruim:

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

Bom:

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string; role: string } }>();
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/reassignment-breaks-reactivity`

Reporta referências reativas que são substituídas por valores simples após o estado cruzar o limite de um arquivo.

Ruim:

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

Bom:

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/value-extraction-breaks-reactivity`

Relata um valor reativo que é copiado em uma encadernação simples de longa duração. Hélices reativas diretas
destructuração são permitidas; O problema é atribuir essa ligação desestruturada a outra ligação
simples.

Ruim:

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

Bom:

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { computed } from "vue";

const { item } = defineProps<{ item: { name: string } }>();
const itemView = computed(() => item);
</script>
```

## `vize:croquis/cf/destructuring-breaks-reactivity`

Relata desestruturação de objetos reativos que não são cobertos pela desestruturação
transformação de props reativos do Vue.

Ruim:

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

Bom:

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

## `vize:croquis/cf/hydration-risk`

Relatórios de valores que podem ser renderizados de forma diferente entre o servidor e o cliente. O grafo ajuda a apontar
do componente de rota ou pai para o componente que gera o valor não determinístico.

Ruim:

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

Bom:

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

Relata trabalho assíncrono reativo que pode durar além do estado que lê, a menos que a limpeza esteja registrada.

Ruim:

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

Bom:

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

Relatórios `watchEffect` callbacks que misturam coleta de dependências com trabalho assíncrono. Use uma fonte
explícita com `watch()` para que a invalidação possa cancelar solicitações obsoletas.

Ruim:

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

Bom:

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

Relata mutações assíncronas para o estado injetado que podem competir com o provedor ou injetores irmãos. Seja
o provedor possua a mutação compartilhada, ou passe um evento/ação explícita de volta para ela.

Ruim:

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

Bom:

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

## Direção de Implementação

O motor de arquivos cruzados é intencionalmente documentado como regras, mesmo que hoje utilize códigos diagnósticos
. Trabalhos futuros podem promover mais regras de Patina nessa camada quando precisarem de importações
, relacionamentos de componentes ou identidade de símbolo em todo o projeto para explicar um problema com precisão.
