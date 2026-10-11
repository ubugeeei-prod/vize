import type { Page } from "playwright";

interface NavigationLocator<L> {
  first(): L;
  and(other: L): L;
  waitFor(): Promise<void>;
}

export function waitForFirstCurrentVariant<L extends NavigationLocator<L>>(items: L, current: L) {
  return items.first().and(current).waitFor();
}

export async function observeVariantNavigation(
  page: Page,
  observations: unknown[],
  labels: { phase: string; title?: string; capturePhase?: string },
) {
  try {
    const state = await page.locator(".variant-toc-item").evaluateAll((items) => {
      const read = (item: Element) => {
        const ariaControls = item.getAttribute("aria-controls");
        const section = ariaControls ? document.getElementById(ariaControls) : null;
        const rect = section?.getBoundingClientRect();
        return {
          name: item.querySelector(".variant-toc-name")?.textContent ?? null,
          ariaCurrent: item.getAttribute("aria-current"),
          ariaControls,
          sectionTop: rect?.top ?? null,
          sectionBottom: rect?.bottom ?? null,
          sectionParentScrollTop: section?.parentElement?.scrollTop ?? null,
        };
      };
      const bounded = items.slice(0, 3).map(read);
      return {
        wallTimestamp: new Date().toISOString(),
        monotonicMs: performance.now(),
        requestedFirst: items[0] ? read(items[0]) : null,
        requestedLast: items.length ? read(items[items.length - 1]) : null,
        tocItemCount: items.length,
        tocItems: bounded,
        activeItems: bounded.filter((item) => item.ariaCurrent === "true"),
        itemOverflow: items.length > 3,
        scrollX: window.scrollX,
        scrollY: window.scrollY,
        documentScrollTop: document.scrollingElement?.scrollTop ?? null,
        producerIntersectionObserverEvents: "unavailable: producer does not expose an event log",
      };
    });
    observations.push({ ...labels, ...state });
  } catch (error) {
    observations.push({
      ...labels,
      wallTimestamp: new Date().toISOString(),
      observationError: String(error),
    });
  }
}
