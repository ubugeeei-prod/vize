import { ref } from "vue";
import type { TokenUsageMap, TokenUsageEntry } from "../api";
import { fetchTokenUsage } from "../api";

const usageMap = ref<TokenUsageMap>({});
const loading = ref(false);
const error = ref<string | null>(null);

let loaded = false;

export function useTokenUsage() {
  async function load() {
    if (loaded) return;
    loading.value = true;
    error.value = null;
    try {
      usageMap.value = await fetchTokenUsage();
      loaded = true;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  async function reload() {
    loaded = false;
    await load();
  }

  function getUsage(tokenPath: string): TokenUsageEntry[] {
    return usageMap.value[tokenPath] ?? [];
  }

  function getUsageCount(tokenPath: string): number {
    const entries = usageMap.value[tokenPath];
    if (!entries) return 0;
    return entries.reduce((sum, entry) => sum + entry.matches.length, 0);
  }

  return {
    usageMap,
    loading,
    error,
    load,
    reload,
    getUsage,
    getUsageCount,
  };
}
