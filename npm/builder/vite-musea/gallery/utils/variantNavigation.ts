interface VariantNavigationContext {
  activeTab: { readonly value: string };
  route: { readonly hash: string };
  artPath: { readonly value: string };
  selectedVariantName: { value: string };
  variantSectionIds: { readonly value: Record<string, string> };
  variantSectionElements: Map<string, Element>;
  nextTick: () => Promise<void>;
}

export function createVariantNavigation({
  activeTab,
  route,
  artPath,
  selectedVariantName,
  variantSectionIds,
  variantSectionElements,
  nextTick,
}: VariantNavigationContext) {
  let variantObserver: IntersectionObserver | null = null;
  let variantObserverGeneration = 0;

  function disconnectVariantObserver() {
    variantObserverGeneration += 1;
    variantObserver?.disconnect();
    variantObserver = null;
  }

  async function setupVariantObserver() {
    disconnectVariantObserver();
    const generation = variantObserverGeneration;

    if (activeTab.value !== "variants") {
      return;
    }

    await nextTick();

    if (
      generation !== variantObserverGeneration ||
      activeTab.value !== "variants" ||
      variantSectionElements.size === 0
    )
      return;

    if (typeof window === "undefined" || !("IntersectionObserver" in window)) {
      return;
    }

    variantObserver = new IntersectionObserver(
      (entries) => {
        if (generation !== variantObserverGeneration) return;

        const visibleEntries = entries
          .filter((entry) => entry.isIntersecting)
          .sort((a, b) => {
            if (Math.abs(a.intersectionRatio - b.intersectionRatio) > 0.05) {
              return b.intersectionRatio - a.intersectionRatio;
            }
            return Math.abs(a.boundingClientRect.top) - Math.abs(b.boundingClientRect.top);
          });

        const activeEntry = visibleEntries[0];
        const variantName =
          activeEntry?.target instanceof HTMLElement
            ? activeEntry.target.dataset.variantName
            : undefined;

        if (variantName && variantName !== selectedVariantName.value) {
          selectedVariantName.value = variantName;
        }
      },
      {
        rootMargin: "-18% 0px -52% 0px",
        threshold: [0.2, 0.4, 0.7],
      },
    );

    for (const element of variantSectionElements.values()) {
      variantObserver.observe(element);
    }
  }

  async function syncSelectionFromHash() {
    if (activeTab.value !== "variants" || !route.hash) {
      return;
    }

    const hash = route.hash;
    const path = artPath.value;
    const generation = variantObserverGeneration;
    await nextTick();

    if (
      activeTab.value !== "variants" ||
      route.hash !== hash ||
      artPath.value !== path ||
      generation !== variantObserverGeneration
    ) {
      return;
    }

    const targetId = decodeURIComponent(hash.slice(1));
    const targetEntry = Object.entries(variantSectionIds.value).find(([, id]) => id === targetId);
    const targetEl = document.getElementById(targetId);

    if (!targetEl) {
      return;
    }

    disconnectVariantObserver();
    if (targetEntry) {
      selectedVariantName.value = targetEntry[0];
    }

    targetEl.scrollIntoView({ behavior: "instant", block: "start" });
    void setupVariantObserver();
  }

  const handleVariantSelect = (variantName: string) => {
    disconnectVariantObserver();
    selectedVariantName.value = variantName;

    const targetId = variantSectionIds.value[variantName];
    const targetEl = targetId ? document.getElementById(targetId) : null;
    targetEl?.scrollIntoView({ behavior: "instant", block: "start" });
    void setupVariantObserver();
  };

  return {
    disconnectVariantObserver,
    setupVariantObserver,
    syncSelectionFromHash,
    handleVariantSelect,
  };
}
