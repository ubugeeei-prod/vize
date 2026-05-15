import { ref, computed } from "vue";
import type { ArtFileInfo } from "../../src/types/index.js";
import { fetchArts } from "../api";

const arts = ref<ArtFileInfo[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

let loaded = false;

export function useArts() {
  const categories = computed(() => {
    const map = new Map<string, ArtFileInfo[]>();
    for (const art of arts.value) {
      const cat = art.metadata.category || "Components";
      if (!map.has(cat)) map.set(cat, []);
      map.get(cat)!.push(art);
    }
    return map;
  });

  async function load() {
    if (loaded) return;
    loading.value = true;
    error.value = null;
    try {
      arts.value = await fetchArts();
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

  function getArt(path: string): ArtFileInfo | undefined {
    return arts.value.find((a) => a.path === path);
  }

  return {
    arts,
    categories,
    loading,
    error,
    load,
    reload,
    getArt,
  };
}
