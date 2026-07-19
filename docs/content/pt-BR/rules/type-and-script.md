---
title: Regras de Tipo e Script
---

<!-- Generated translation; source: rules/type-and-script.md -->

# Regras de Tipo e Script

Regras de tipo usam o verificador TypeScript quando informações semânticas são necessárias. O Vize lê a mesma forma
projeto que o TypeScript lê de `tsconfig.json`, então nomes de ambientes compartilhados devem vir de
`compilerOptions.types`, referências de projeto ou arquivos de declaração.

Regras de script são regras Patina para API de composição e código orientado a Vapor. Eles focam em padrões
que são difíceis de compilar de forma eficiente ou difíceis de raciocinar no modo Vapor.

O linting consciente do tipo é opt-in. Ative-o com `linter.typeAware: true`, `vize lint --type-aware`ou
ativando explicitamente uma regra `type/*`. `type/no-reactivity-loss` pode ser ativado diretamente com
`vize lint --strict-reactivity`. Se a Corsa não puder ser iniciada, Patina reporta `type/corsa-runtime` e
pula a passagem de regras com checker, em vez de silenciosamente descartar as regras configuradas.

`--type-aware` usa a mesma resolução executável Corsa que `vize check`; Configure
`typeChecker.corsaPath` quando o projeto precisar de um binário `tsgo` explícito ou Corsa. Os padrões permanecem
custo zero: Patina não analisa SFCs para linting com xadrez nem inicia Corsa a menos que a bandeira,
`linter.typeAware`, ou uma regra de `type/*` explicitamente ativada opte.

```ts
export default defineConfig({
  linter: { typeAware: true },
});
```

## `type/require-typed-props`

Requer que `defineProps` seja tipado em vez de usar uma declaração de array em tempo de execução.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<script setup lang="ts">
const props = defineProps(["label", "count"]);
</script>
```

Bom:

```vue
<script setup lang="ts">
const props = defineProps<{
  label: string;
  count: number;
}>();
</script>
```

## `type/require-typed-emits`

Requer `defineEmits` descreva as cargas úteis de eventos emitidas.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<script setup lang="ts">
const emit = defineEmits(["save"]);

emit("save", form.value);
</script>
```

Bom:

```vue
<script setup lang="ts">
const emit = defineEmits<{
  save: [payload: FormValue];
}>();

emit("save", form.value);
</script>
```

## `type/no-unsafe-template-binding`

Relatórios de templates bindings que resolvem valores inseguros, como `any`. A regra é backed por checker,
então ela segue os tipos importados e a configuração do projeto.

Gravidade padrão: `warning`
Presets: `nuxt`, `opinionated`

Ruim:

```vue
<script setup lang="ts">
const payload: any = await loadPayload();
</script>

<template>
  <p>{{ payload.title }}</p>
</template>
```

Bom:

```vue
<script setup lang="ts">
type Payload = { title: string };

const payload = await loadPayload<Payload>();
</script>

<template>
  <p>{{ payload.title }}</p>
</template>
```

## `type/no-floating-promises`

Relata promessas que são criadas, mas não aguardadas, devolvidas ou intencionalmente tratadas.
A verificação cobre tanto expressões `<script>` quanto modelos.

Gravidade padrão: `warning`
Presets: `nuxt`, `opinionated`

Ruim:

```vue
<script setup lang="ts">
function submit() {
  saveForm(form.value);
}
</script>

<template>
  <button @click="saveForm(form)">Save</button>
  <p>{{ loadPreview() }}</p>
</template>
```

Bom:

```vue
<script setup lang="ts">
type Preview = { title: string };

async function submit() {
  await saveForm(form.value);
}

const preview = ref<Preview | null>(null);

async function loadPreviewIntoState() {
  preview.value = await loadPreview();
}
</script>

<template>
  <button @click="void submit()">Save</button>
  <button @click="void loadPreviewIntoState()">Preview</button>
  <PreviewPanel v-if="preview" :preview="preview" />
</template>
```

## `type/no-reactivity-loss`

Reporta instantâneos simples dos valores reativos usados entre fluxos. A regra também é executada quando
`vize lint --strict-reactivity` está ativada.

Gravidade padrão: `warning`
Presets: `nuxt`, `opinionated`

Ruim:

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const item = props.item;
</script>
```

Bom:

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

## Configuração do Checker

As regras conscientes de tipos não precisam de um campo separado de `globals` Vize para nomes TypeScript. Prefira
configuração nativa do TypeScript:

Ruim:

```ts
export default {
  globals: ["definePageMeta", "process"],
};
```

Bom:

```json
{
  "compilerOptions": {
    "types": ["node", "nuxt/app"]
  }
}
```

## `script/no-options-api`

Relatórios Opções Definições de componentes da API em presets orientados ao Vapor.

Gravidade padrão: `error`
Presets: `nuxt`, `opinionated`

Ruim:

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

Bom:

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## `script/no-next-tick`

Relatórios `nextTick()` em componentes orientados a vapor. Prefira referências diretas, ganchos de ciclo de vida ou fluxo de estado
que não dependa do próximo flush do DOM.

Gravidade padrão: `error`
Presets: `nuxt`, `opinionated`

Ruim:

```vue
<script setup lang="ts" vapor>
await nextTick();
input.value?.focus();
</script>
```

Bom:

```vue
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>
```

## `script/no-get-current-instance`

Relatórios `getCurrentInstance()` em componentes orientados a vapor. Ele alcança internos de runtime que
Vapor não consegue otimizar com segurança.

Gravidade padrão: `error`
Presets: `nuxt`, `opinionated`

Ruim:

```vue
<script setup lang="ts" vapor>
const instance = getCurrentInstance();
const app = instance?.appContext.app;
</script>
```

Bom:

```vue
<script setup lang="ts" vapor>
const appConfig = useAppConfig();
</script>
```
