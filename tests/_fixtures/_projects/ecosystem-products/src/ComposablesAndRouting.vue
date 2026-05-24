<script setup lang="ts">
import type {} from "./shims";
import { computed, ref, watch } from "vue";
import { useDark, useDebounceFn, useToggle } from "@vueuse/core";
import { useI18n } from "vue-i18n";
import { RouterLink, useRouter } from "vue-router";
import { useField, useForm } from "vee-validate";
import { useQuery as useTanStackQuery } from "@tanstack/vue-query";
import { useQuery as useApolloQuery } from "@vue/apollo-composable";
import { gql } from "@apollo/client/core";
import VSelect from "./vendors/vue-select";
import "vue-select/dist/vue-select.css";
import type { ApolloProject, ProductOption } from "./types";

interface ProductForm {
  email: string;
  library: string;
}

interface ProductsQueryResult {
  products: ApolloProject[];
}

const router = useRouter();
const isDark = useDark({ storageKey: "vize-ecosystem-dark" });
const toggleDark = useToggle(isDark);
const search = ref("vue");
const debouncedSearch = ref(search.value);
const selectedProduct = ref("vue-router");

const productOptions = [
  { label: "Vue Router", value: "vue-router" },
  { label: "VueUse", value: "vueuse" },
  { label: "Vue I18n", value: "vue-i18n" },
  { label: "Vee Validate", value: "vee-validate" },
  { label: "TanStack Query Vue", value: "tanstack-query-vue" },
  { label: "Vue Apollo", value: "vue-apollo" },
  { label: "Vue Select", value: "vue-select" },
] satisfies ProductOption[];

const updateDebouncedSearch = useDebounceFn((value: string) => {
  debouncedSearch.value = value;
}, 10);

watch(search, (value) => updateDebouncedSearch(value), { immediate: true });

const { t, locale } = useI18n({
  legacy: false,
  locale: "en",
  messages: {
    en: {
      heading: "Vue ecosystem search",
      submit: "Submit",
    },
    ja: {
      heading: "Vue ecosystem search",
      submit: "Submit",
    },
  },
});

const { handleSubmit, errors, meta } = useForm<ProductForm>({
  initialValues: {
    email: "team@example.com",
    library: selectedProduct.value,
  },
  validationSchema: {
    email(value: string) {
      return value.includes("@") || "Email must contain @";
    },
    library(value: string) {
      return value.length > 0 || "Pick a library";
    },
  },
});

const { value: email } = useField<string>("email");
const { value: library } = useField<string>("library");

const tanStackQuery = useTanStackQuery({
  queryKey: computed(() => ["ecosystem-products", debouncedSearch.value] as const),
  queryFn: async () => {
    const needle = debouncedSearch.value.toLowerCase();
    return productOptions.filter((option) => option.label.toLowerCase().includes(needle));
  },
  staleTime: 30_000,
});

const productsDocument = gql`
  query Products($search: String!) {
    products(search: $search) {
      id
      name
    }
  }
`;

const apolloVariables = computed(() => ({ search: debouncedSearch.value }));
const apolloQuery = useApolloQuery<ProductsQueryResult, { search: string }>(
  productsDocument,
  apolloVariables,
  { enabled: false },
);

const resolvedHref = computed(() => {
  return router.resolve({ path: "/ecosystem", query: { q: debouncedSearch.value } }).href;
});

const submitted = ref<ProductForm | null>(null);
const onSubmit = handleSubmit((values) => {
  submitted.value = values;
  selectedProduct.value = values.library;
});

function reduceProduct(option: ProductOption): string {
  return option.value;
}
</script>

<template>
  <form class="composable-panel" @submit.prevent="onSubmit">
    <header>
      <h2>{{ t("heading") }}</h2>
      <button type="button" @click="toggleDark()">Dark mode: {{ isDark ? "on" : "off" }}</button>
      <select v-model="locale" aria-label="Locale">
        <option value="en">English</option>
        <option value="ja">Japanese</option>
      </select>
    </header>

    <label>
      Search
      <input v-model="search" name="search" />
    </label>

    <label>
      Email
      <input v-model="email" name="email" :aria-invalid="Boolean(errors.email)" />
      <span v-if="errors.email">{{ errors.email }}</span>
    </label>

    <label>
      Library
      <VSelect
        v-model="library"
        :options="productOptions"
        label="label"
        :reduce="reduceProduct"
      />
      <span v-if="errors.library">{{ errors.library }}</span>
    </label>

    <RouterLink :to="{ path: '/ecosystem', query: { q: debouncedSearch } }" custom v-slot="{ href, navigate }">
      <a :href="href" @click.prevent="navigate">RouterLink target: {{ resolvedHref }}</a>
    </RouterLink>

    <ul>
      <li v-for="option in tanStackQuery.data.value ?? []" :key="option.value">
        {{ option.label }}
      </li>
    </ul>

    <p>Apollo loading: {{ apolloQuery.loading.value }}</p>
    <p>Form dirty: {{ meta.dirty }}</p>
    <p v-if="submitted">Submitted {{ submitted.email }} for {{ submitted.library }}</p>

    <button type="submit">{{ t("submit") }}</button>
  </form>
</template>

<style scoped>
.composable-panel {
  display: grid;
  gap: 12px;
  max-width: 560px;
}

header,
label {
  display: grid;
  gap: 6px;
}

button,
input,
select {
  min-height: 34px;
}
</style>
