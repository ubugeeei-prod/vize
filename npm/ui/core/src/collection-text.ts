const ignoredTextTags = new Set(["NOSCRIPT", "SCRIPT", "STYLE", "TEMPLATE"]);

/**
 * Normalize authored or extracted typeahead text without applying a locale.
 *
 * Unicode is normalized to NFC and every whitespace run becomes one ASCII
 * space. Locale-sensitive equality remains the responsibility of a collator.
 */
export function normalizeCollectionTextValue(value: string): string {
  return value.normalize("NFC").trim().replace(/\s+/gu, " ");
}

/**
 * Extract a practical accessible text value from an item element.
 *
 * `aria-labelledby` and `aria-label` take precedence, hidden descendants are
 * excluded unless explicitly referenced, and image/input fallbacks are
 * included. Complex widgets should still provide `textValue` explicitly when
 * their typeahead label differs from their accessible name.
 */
export function extractCollectionTextValue(element: Element): string {
  return normalizeCollectionTextValue(readElementText(element, new Set(), false));
}

/** Whether a candidate text starts with a query under locale-aware comparison. */
export function collectionTextStartsWith(
  candidate: string,
  query: string,
  collator: Intl.Collator,
): boolean {
  let codeUnitLength = 0;
  for (const character of candidate) {
    codeUnitLength += character.length;
    if (collator.compare(candidate.slice(0, codeUnitLength), query) === 0) return true;
  }
  return false;
}

function readElementText(element: Element, visited: Set<Element>, referenced: boolean): string {
  if (visited.has(element)) return "";
  visited.add(element);
  if (!referenced && isCollectionTextHidden(element)) return "";

  const labelledBy = element.getAttribute("aria-labelledby");
  if (labelledBy !== null) {
    const root = element.getRootNode();
    const referenceRoot = hasGetElementById(root) ? root : element.ownerDocument;
    const labelledText = labelledBy
      .split(/\s+/u)
      .filter((id) => id.length > 0)
      .map((id) => referenceRoot.getElementById(id))
      .filter((label): label is Element => label !== null)
      .map((label) => readElementText(label, visited, true))
      .join(" ");
    if (normalizeCollectionTextValue(labelledText).length > 0) return labelledText;
  }

  const ariaLabel = element.getAttribute("aria-label");
  if (ariaLabel !== null && normalizeCollectionTextValue(ariaLabel).length > 0) return ariaLabel;

  const tagName = element.tagName.toUpperCase();
  if (tagName === "IMG" || tagName === "AREA") {
    const alternative = element.getAttribute("alt");
    if (alternative !== null) return alternative;
  }
  if (tagName === "INPUT") {
    const type = element.getAttribute("type")?.toLowerCase();
    if (type === "button" || type === "reset" || type === "submit") {
      const value = (element as HTMLInputElement).value;
      if (value.length > 0) return value;
    }
  }

  if (!ignoredTextTags.has(tagName)) {
    const fragments: string[] = [];
    for (const child of element.childNodes) {
      if (child.nodeType === 3) {
        fragments.push(child.nodeValue ?? "");
      } else if (child.nodeType === 1) {
        fragments.push(readElementText(child as Element, visited, referenced));
      }
    }
    const content = fragments.join("");
    if (normalizeCollectionTextValue(content).length > 0) return content;
  }

  return element.getAttribute("title") ?? "";
}

interface CollectionIdReferenceRoot extends Node {
  getElementById(id: string): Element | null;
}

function hasGetElementById(node: Node): node is CollectionIdReferenceRoot {
  return "getElementById" in node && typeof node.getElementById === "function";
}

function isCollectionTextHidden(element: Element): boolean {
  return (
    element.hasAttribute("hidden") ||
    element.hasAttribute("inert") ||
    element.getAttribute("aria-hidden")?.trim().toLowerCase() === "true"
  );
}
