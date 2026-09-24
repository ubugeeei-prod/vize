<script setup lang="ts">
import { computed } from "vue";
import type { TokenCategory, DesignToken, TokenUsageMap } from "../../api";
import TokenCard from "./TokenCard.vue";

const props = withDefaults(
  defineProps<{
    category: TokenCategory;
    level?: number;
    parentPath?: string;
    usageMap?: TokenUsageMap;
    tokenMap?: Record<string, DesignToken>;
  }>(),
  {
    usageMap: () => ({}),
    tokenMap: () => ({}),
  },
);

const emit = defineEmits<{
  edit: [path: string, token: DesignToken];
  delete: [path: string, token: DesignToken];
  showUsage: [tokenPath: string];
}>();

const headingLevel = computed(() => Math.min(props.level ?? 2, 6));
const tokenEntries = computed(() =>
  Object.entries(props.category.tokens).map(([name, token]) => ({ name, token })),
);

function getCategoryPath(): string {
  const catKey = props.category.name.toLowerCase().replace(/\s+/g, "-");
  return props.parentPath ? `${props.parentPath}.${catKey}` : catKey;
}

function getTokenPath(name: string): string {
  return `${getCategoryPath()}.${name}`;
}

function getUsageCount(name: string): number {
  const tokenPath = getTokenPath(name);
  const entries = props.usageMap[tokenPath];
  if (!entries) return 0;
  return entries.reduce((sum, entry) => sum + entry.matches.length, 0);
}
</script>

<template>
  <div class="token-category" :class="{ 'token-subcategory': level && level > 2 }">
    <h2 v-if="headingLevel === 2" class="category-title category-title--h2">{{ category.name }}</h2>
    <h3 v-else-if="headingLevel === 3" class="category-title category-title--h3">
      {{ category.name }}
    </h3>
    <h4 v-else-if="headingLevel === 4" class="category-title category-title--h4">
      {{ category.name }}
    </h4>
    <h5 v-else-if="headingLevel === 5" class="category-title category-title--h5">
      {{ category.name }}
    </h5>
    <h6 v-else class="category-title category-title--h6">{{ category.name }}</h6>

    <div v-if="tokenEntries.length > 0" class="tokens-grid">
      <TokenCard
        v-for="entry in tokenEntries"
        :key="getTokenPath(entry.name)"
        :name="entry.name"
        :token="entry.token"
        :token-path="getTokenPath(entry.name)"
        :token-map
        :usage-count="getUsageCount(entry.name)"
        @edit="() => emit('edit', getTokenPath(entry.name), entry.token)"
        @delete="() => emit('delete', getTokenPath(entry.name), entry.token)"
        @show-usage="() => emit('showUsage', getTokenPath(entry.name))"
      />
    </div>

    <template v-if="category.subcategories">
      <TokenCategorySection
        v-for="sub in category.subcategories"
        :key="sub.name"
        :category="sub"
        :level="(level ?? 2) + 1"
        :parent-path="getCategoryPath()"
        :usage-map
        :token-map
        @edit="(path, token) => emit('edit', path, token)"
        @delete="(path, token) => emit('delete', path, token)"
        @show-usage="(tokenPath) => emit('showUsage', tokenPath)"
      />
    </template>
  </div>
</template>

<style scoped>
.token-category {
  margin-bottom: 2.5rem;
}

.token-subcategory {
  margin-top: 1.5rem;
  margin-inline-start: 1rem;
  margin-bottom: 0;
}

.category-title {
  font-weight: 600;
  margin-bottom: 1rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--musea-border);
}

.category-title--h2 {
  font-size: 1.125rem;
}

.category-title--h3 {
  font-size: 0.9375rem;
  color: var(--musea-text-secondary);
  border-bottom: none;
  padding-bottom: 0;
}

.category-title--h4,
.category-title--h5,
.category-title--h6 {
  font-size: 0.875rem;
  color: var(--musea-text-muted);
  border-bottom: none;
  padding-bottom: 0;
}

.tokens-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 1rem;
}
</style>
