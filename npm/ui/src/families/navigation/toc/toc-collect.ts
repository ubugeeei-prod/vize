import type { TocCollectOptions, TocEntry } from "./toc-types.ts";

/**
 * Collect headings with ids from rendered content, in document order.
 *
 * This reads the DOM, so call it after mount (for example in `onMounted`) or
 * pass server-generated entries instead to keep SSR output complete.
 */
export function collectTocEntries(
  container: ParentNode,
  options: TocCollectOptions = {},
): readonly TocEntry[] {
  const ignore = options.ignoreAttribute ?? "data-toc-ignore";
  const entries: TocEntry[] = [];
  for (const heading of container.querySelectorAll<HTMLElement>(options.selector ?? "h2, h3")) {
    if (heading.id === "" || heading.hasAttribute(ignore)) continue;
    const match = /^h([1-6])$/i.exec(heading.tagName);
    const ariaLevel = Number(heading.getAttribute("aria-level"));
    const level =
      match?.[1] === undefined
        ? Number.isInteger(ariaLevel) && ariaLevel > 0
          ? ariaLevel
          : 2
        : Number(match[1]);
    const text = (heading.textContent ?? "").replaceAll(/\s+/g, " ").trim();
    entries.push(Object.freeze({ id: heading.id, level, text }));
  }
  return Object.freeze(entries);
}
