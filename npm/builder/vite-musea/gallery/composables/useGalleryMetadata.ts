import { ref } from "vue";

import type { ArtFileInfo, MuseaGalleryMetadataConfig } from "../../src/types/index.js";
import { fetchGalleryMetadata } from "../api";

const config = ref<MuseaGalleryMetadataConfig>({});
let loaded = false;

export function useGalleryMetadata() {
  async function load(): Promise<void> {
    if (loaded) return;
    config.value = await fetchGalleryMetadata();
    loaded = true;
  }

  function visibleTags(art: ArtFileInfo): string[] {
    return art.metadata.tags.filter((tag) => !config.value.tags?.[tag]?.hidden);
  }

  return { config, load, visibleTags };
}
