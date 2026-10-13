// Serialized by Playwright. Keep this read-only and entirely self-contained.
export function collectFailureCssom() {
  const startedAt = performance.now();
  const omissions = [];
  const sheets = [];
  const sheetIds = new Map();
  const elements = [];
  const elementIds = new Map();
  const styles = (element) => {
    const computed = getComputedStyle(element);
    return Object.fromEntries([...computed].map((name) => [name, computed.getPropertyValue(name)]));
  };
  const elementRef = (element) => {
    if (!element) return null;
    if (elementIds.has(element)) return elementIds.get(element);
    const id = elements.length;
    elementIds.set(element, id);
    const entry = {
      id,
      tag: element.tagName,
      attributes: [...element.attributes].map((attribute) => [attribute.name, attribute.value]),
      rect: element.getBoundingClientRect().toJSON(),
      computed: styles(element),
      parent: null,
    };
    elements.push(entry);
    entry.parent = elementRef(element.parentElement);
    return id;
  };
  const headingElements = [...document.querySelectorAll("h1,h2,h3,h4,h5,h6")];
  const headings = headingElements.map((element) => ({
    element: elementRef(element),
    outerHTML: element.outerHTML,
    textContent: element.textContent,
  }));
  const sheetRef = (sheet) => {
    if (!sheet) return null;
    if (sheetIds.has(sheet)) return sheetIds.get(sheet);
    const id = sheets.length;
    sheetIds.set(sheet, id);
    const entry = {
      id,
      href: sheet.href,
      disabled: sheet.disabled,
      media: sheet.media.mediaText,
      mediaMatches: matchMedia(sheet.media.mediaText || "all").matches,
      owner: sheet.ownerNode instanceof Element ? sheet.ownerNode.outerHTML : null,
      rules: [],
      complete: true,
    };
    sheets.push(entry);
    try {
      entry.rules = [...sheet.cssRules].map(safeRule);
    } catch (error) {
      entry.complete = false;
      entry.missing = String(error);
      omissions.push({ sheet: id, reason: String(error) });
    }
    return id;
  };
  const ruleValue = (rule) => {
    const entry = { type: rule.type, constructor: rule.constructor.name, cssText: rule.cssText };
    for (const key of ["selectorText", "conditionText", "name", "layerName", "supportsText"]) {
      if (key in rule) entry[key] = rule[key];
    }
    if (rule.media) entry.media = rule.media.mediaText;
    if (rule.style) {
      entry.declarations = [...rule.style].map((name) => ({
        name,
        value: rule.style.getPropertyValue(name),
        priority: rule.style.getPropertyPriority(name),
      }));
    }
    if (rule.selectorText) {
      try {
        entry.matchingHeadings = headings.flatMap((heading, index) =>
          headingElements[index].matches(rule.selectorText) ? [heading.element] : [],
        );
      } catch (error) {
        entry.selectorMatchingMissing = String(error);
      }
    }
    if ("cssRules" in rule) entry.rules = [...rule.cssRules].map(safeRule);
    if ("styleSheet" in rule) entry.importedSheet = sheetRef(rule.styleSheet);
    return entry;
  };
  const safeRule = (rule) => {
    try {
      return ruleValue(rule);
    } catch (error) {
      const reason = String(error);
      omissions.push({ rule: rule.cssText, reason });
      return { cssText: rule.cssText, complete: false, missing: reason };
    }
  };
  const documentSheets = [...document.styleSheets].map(sheetRef);
  const adoptedSheets = [...document.adoptedStyleSheets].map(sheetRef);
  const shadowRoots = [];
  const visitShadowRoots = (root) => {
    for (const element of root.querySelectorAll("*")) {
      if (!element.shadowRoot) continue;
      shadowRoots.push({
        host: element.outerHTML,
        documentHTML: element.shadowRoot.innerHTML,
        sheets: [...element.shadowRoot.styleSheets].map(sheetRef),
        adoptedSheets: [...element.shadowRoot.adoptedStyleSheets].map(sheetRef),
      });
      visitShadowRoots(element.shadowRoot);
    }
  };
  visitShadowRoots(document);
  return {
    schema: "vize.vuefes.post-matcher-failure-cssom",
    version: 1,
    complete: omissions.length === 0,
    boundary: "after original matcher failure; not atomic screenshot-time authority",
    startedAt,
    finishedAt: performance.now(),
    timeOrigin: performance.timeOrigin,
    url: location.href,
    navigation: performance.getEntriesByType("navigation").map((entry) => entry.toJSON()),
    documentHTML: document.documentElement.outerHTML,
    viewport: { width: innerWidth, height: innerHeight, devicePixelRatio },
    scroll: {
      x: scrollX,
      y: scrollY,
      width: document.body.scrollWidth,
      height: document.body.scrollHeight,
    },
    documentSheets,
    adoptedSheets,
    shadowRoots,
    sheets,
    elements,
    headings,
    omissions,
  };
}
