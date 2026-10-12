import type { Page } from "playwright";
import { fileURLToPath } from "node:url";

interface ObserverRecord {
  id: number;
  callback: IntersectionObserverCallback;
  observer: IntersectionObserver;
  disconnected: boolean;
  deliveries: IntersectionObserverEntry[][];
}
interface Probe {
  records: ObserverRecord[];
  events: unknown[];
}
type ProbedWindow = { __museaNavigationProbe: Probe };

export async function instrumentObserver(page: Page) {
  await page.addInitScript({
    path: fileURLToPath(
      new URL("./variant-navigation-browser.instrumentation.js", import.meta.url),
    ),
  });
}

export async function observerCount(page: Page) {
  return page.evaluate(
    () => (window as unknown as ProbedWindow).__museaNavigationProbe.records.length,
  );
}

export async function waitForNewNativeDelivery(page: Page, previousCount: number) {
  await page.waitForFunction(
    (count) => {
      const records = (window as unknown as ProbedWindow).__museaNavigationProbe.records;
      return records
        .slice(count)
        .some((record) => !record.disconnected && record.deliveries.length > 0);
    },
    previousCount,
    { timeout: 8_000 },
  );
  await settle(page);
}

export async function settle(page: Page) {
  await page.evaluate(
    () =>
      new Promise<void>((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
      }),
  );
}

export async function replayFormerNativeDelivery(page: Page, name: string) {
  return page.evaluate((name) => {
    const probe = (window as unknown as ProbedWindow).__museaNavigationProbe;
    const retained = probe.records.findLast(
      (record) =>
        record.disconnected &&
        record.deliveries.some((entries) =>
          entries.some(
            (entry) =>
              entry.isIntersecting && (entry.target as HTMLElement).dataset.variantName === name,
          ),
        ),
    );
    if (!retained) throw new Error(`No genuine retained native delivery for ${name}`);
    const entries = retained.deliveries.findLast((entries) =>
      entries.some(
        (entry) =>
          entry.isIntersecting && (entry.target as HTMLElement).dataset.variantName === name,
      ),
    )!;
    // A deliberate causal control, separate from native-delivery observations. These are
    // the real formerly owned callback and unmodified native Entry objects, not fake bounds.
    retained.callback(entries, retained.observer);
    const event = {
      kind: "retained-callback-control",
      id: retained.id,
      time: performance.now(),
      names: entries.map((entry) => (entry.target as HTMLElement).dataset.variantName),
    };
    probe.events.push(event);
    return event;
  }, name);
}

export async function geometry(page: Page, name: string) {
  return page.evaluate((name) => {
    const item = [...document.querySelectorAll(".variant-toc-item")].find(
      (item) => item.querySelector(".variant-toc-name")?.textContent === name,
    );
    const id = item?.getAttribute("aria-controls");
    const section = id ? document.getElementById(id) : null;
    if (!item || !section) throw new Error(`Missing physical target ${name}`);
    const ancestors = [];
    for (let parent = section.parentElement; parent; parent = parent.parentElement) {
      if (parent.scrollHeight <= parent.clientHeight + 1) continue;
      const style = getComputedStyle(parent);
      ancestors.push({
        tag: parent.tagName,
        class: parent.className,
        overflow: style.overflowY,
        top: parent.getBoundingClientRect().top,
        clientHeight: parent.clientHeight,
        scrollHeight: parent.scrollHeight,
        scrollTop: parent.scrollTop,
      });
    }
    return {
      name,
      id,
      aria: item.getAttribute("aria-current"),
      rect: section.getBoundingClientRect().toJSON(),
      viewport: { width: innerWidth, height: innerHeight },
      ancestors,
      selected: [...document.querySelectorAll('.variant-toc-item[aria-current="true"]')].map(
        (item) => item.querySelector(".variant-toc-name")?.textContent,
      ),
      variantVector: [...document.querySelectorAll(".variant-toc-item")].map((item) => {
        const ariaControls = item.getAttribute("aria-controls");
        const target = ariaControls ? document.getElementById(ariaControls) : null;
        return {
          name: item.querySelector(".variant-toc-name")?.textContent,
          aria: item.getAttribute("aria-current"),
          ariaControls,
          id: target?.id,
          sectionName: target?.dataset.variantName,
          rect: target?.getBoundingClientRect().toJSON(),
        };
      }),
      time: performance.now(),
    };
  }, name);
}

export async function observerEvidence(page: Page) {
  return page.evaluate(() => {
    const probe = (window as unknown as ProbedWindow).__museaNavigationProbe;
    return {
      events: probe.events,
      records: probe.records.map((record) => ({
        id: record.id,
        disconnected: record.disconnected,
        nativeDeliveries: record.deliveries.length,
        targets: record.deliveries.map((entries) =>
          entries.map((entry) => ({
            name: (entry.target as HTMLElement).dataset.variantName,
            isIntersecting: entry.isIntersecting,
            ratio: entry.intersectionRatio,
            bounds: entry.boundingClientRect.toJSON(),
            root: entry.rootBounds?.toJSON(),
            intersection: entry.intersectionRect.toJSON(),
            time: entry.time,
          })),
        ),
      })),
    };
  });
}
