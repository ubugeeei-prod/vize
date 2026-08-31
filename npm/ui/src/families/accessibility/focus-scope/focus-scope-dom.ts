import { comparePositiveTabindex } from "./focus-scope-internal.ts";
import type { FocusScopeAutoFocusEvent, FocusScopeMoveOptions } from "./focus-scope-types.ts";

const focusableSelector = [
  "a[href]",
  "area[href]",
  "audio[controls]",
  "button",
  "embed",
  "iframe",
  "input",
  "object",
  "select",
  "summary",
  "textarea",
  "video[controls]",
  "[contenteditable]",
  "[tabindex]",
].join(",");

function composedParent(element: Element): Element | null {
  if ((element as HTMLElement).assignedSlot) return (element as HTMLElement).assignedSlot;
  if (element.parentElement) return element.parentElement;
  const root = element.getRootNode() as Partial<ShadowRoot>;
  return root.host ?? null;
}

export function containsComposed(root: Element, element: Element | null): boolean {
  let current = element;
  while (current) {
    if (root === current || root.contains(current)) return true;
    current = composedParent(current);
  }
  return false;
}

export function deepActiveElement(document: Document): HTMLElement | null {
  let active = document.activeElement as HTMLElement | null;
  while (active?.shadowRoot?.activeElement) {
    active = active.shadowRoot.activeElement as HTMLElement;
  }
  return active;
}

function isHidden(
  element: HTMLElement,
  boundary: Element,
  checkComputedStyle: boolean,
  computedHidden = new Map<Element, boolean>(),
): boolean {
  let current: Element | null = element;
  while (current) {
    const html = current as HTMLElement;
    if (
      html.hidden ||
      html.getAttribute("aria-hidden") === "true" ||
      html.hasAttribute("inert") ||
      html.inert === true
    ) {
      return true;
    }
    if (html.localName === "details" && !html.hasAttribute("open")) {
      const summary = Array.from(html.children).find(({ localName }) => localName === "summary");
      if (!summary || (summary !== element && !summary.contains(element))) return true;
    }
    if (
      html.style.display === "none" ||
      html.style.visibility === "hidden" ||
      html.style.visibility === "collapse"
    ) {
      return true;
    }
    if (checkComputedStyle) {
      let hidden = computedHidden.get(current);
      if (hidden === undefined) {
        const style = html.ownerDocument.defaultView?.getComputedStyle(html);
        hidden =
          style?.display === "none" ||
          style?.visibility === "hidden" ||
          style?.visibility === "collapse";
        computedHidden.set(current, hidden);
      }
      if (hidden) return true;
    }
    if (current === boundary) break;
    current = composedParent(current);
  }
  return false;
}

function hasStyleSheets(element: Element): boolean {
  const scopes: Partial<DocumentOrShadowRoot>[] = [
    element.ownerDocument,
    element.getRootNode() as Partial<DocumentOrShadowRoot>,
  ];
  return scopes.some(
    (scope) => (scope.styleSheets?.length ?? 0) + (scope.adoptedStyleSheets?.length ?? 0) > 0,
  );
}

function isDisabled(element: HTMLElement): boolean {
  if ((element as HTMLInputElement).disabled === true) return true;

  let current = element.parentElement;
  while (current) {
    const html = current as HTMLElement;
    if (
      (html.localName === "optgroup" || html.localName === "select") &&
      (html as HTMLSelectElement).disabled === true
    ) {
      return true;
    }
    if (html.localName === "fieldset" && (html as HTMLFieldSetElement).disabled === true) {
      const firstLegend = Array.from(html.children).find(({ localName }) => localName === "legend");
      if (!firstLegend || (firstLegend !== element && !firstLegend.contains(element))) return true;
    }
    current = current.parentElement;
  }
  return false;
}

function isNativeFocusable(element: HTMLElement): boolean {
  const name = element.localName;
  if (name === "input") return (element as HTMLInputElement).type !== "hidden";
  if (name === "a" || name === "area") return element.hasAttribute("href");
  if (name === "audio" || name === "video") return element.hasAttribute("controls");
  if (
    name === "button" ||
    name === "embed" ||
    name === "iframe" ||
    name === "object" ||
    name === "select" ||
    name === "textarea"
  ) {
    return true;
  }
  if (name === "summary") {
    const details = element.parentElement;
    return (
      details?.localName === "details" &&
      Array.from(details.children).find(({ localName }) => localName === "summary") === element
    );
  }
  const contenteditable = element.getAttribute("contenteditable");
  return (
    element.isContentEditable ||
    contenteditable === "" ||
    contenteditable === "true" ||
    contenteditable === "plaintext-only"
  );
}

function isProgrammaticallyFocusable(element: HTMLElement): boolean {
  if (
    element.localName === "input" &&
    (element as HTMLInputElement).type.toLowerCase() === "hidden"
  ) {
    return false;
  }
  return (
    element.matches(focusableSelector) &&
    (element.hasAttribute("tabindex") || isNativeFocusable(element))
  );
}

function isRadioTabbable(element: HTMLElement, candidates: readonly HTMLElement[]): boolean {
  if (element.localName !== "input" || (element as HTMLInputElement).type !== "radio") return true;
  const radio = element as HTMLInputElement;
  if (!radio.name || radio.checked) return true;
  const group = candidates.filter((candidate) => {
    const input = candidate as HTMLInputElement;
    return (
      candidate.localName === "input" &&
      input.type === "radio" &&
      input.name === radio.name &&
      input.form === radio.form &&
      candidate.getRootNode() === radio.getRootNode()
    );
  }) as HTMLInputElement[];
  return !group.some(({ checked }) => checked) && group[0] === radio;
}

function collectElements(root: Element): HTMLElement[] {
  const elements: HTMLElement[] = [];
  const seen = new Set<Element>();
  const visitElement = (element: Element): void => {
    if (seen.has(element)) return;
    seen.add(element);
    if (
      element.namespaceURI === "http://www.w3.org/1999/xhtml" &&
      typeof (element as HTMLElement).focus === "function"
    ) {
      elements.push(element as HTMLElement);
    }
    const slot = element as HTMLSlotElement;
    if (slot.localName === "slot" && typeof slot.assignedElements === "function") {
      const assigned = slot.assignedElements({ flatten: true });
      if (assigned.length > 0) {
        for (const child of assigned) visitElement(child);
        return;
      }
    }
    const children = element.shadowRoot?.children ?? element.children;
    for (const child of children) visitElement(child);
  };
  const children = root.shadowRoot?.children ?? root.children;
  for (const child of children) {
    visitElement(child);
  }
  return elements;
}

export function focusableElements(
  root: Element,
  options: Pick<FocusScopeMoveOptions, "accept" | "includeProgrammatic"> = {},
): HTMLElement[] {
  const document = root.ownerDocument;
  const checkComputedStyle = hasStyleSheets(root);
  const visibilityBoundary = document.documentElement ?? root;
  const computedHidden = new Map<Element, boolean>();
  const candidates = collectElements(root).filter((element) => {
    if (
      !isProgrammaticallyFocusable(element) ||
      isDisabled(element) ||
      isHidden(element, visibilityBoundary, checkComputedStyle, computedHidden)
    ) {
      return false;
    }
    const tabindex = element.getAttribute("tabindex");
    const explicit = tabindex !== null && Number(tabindex) >= 0;
    const native = isNativeFocusable(element);
    const programmatic = tabindex !== null || native;
    const sequential = tabindex === null ? native || element.tabIndex >= 0 : explicit;
    const eligible = options.includeProgrammatic ? programmatic : sequential;
    return eligible && options.accept?.(element) !== false;
  });
  const radioFiltered = candidates.filter((element) =>
    options.includeProgrammatic ? true : isRadioTabbable(element, candidates),
  );
  return radioFiltered
    .map((element, order) => ({ element, order }))
    .sort(
      (left, right) =>
        comparePositiveTabindex(left.element, right.element) || left.order - right.order,
    )
    .map(({ element }) => element);
}

export function focusElement(element: HTMLElement, preventScroll = true): void {
  try {
    element.focus({ preventScroll });
  } catch {
    element.focus();
  }
}

export function isUsableTarget(element: HTMLElement | null): element is HTMLElement {
  if (
    element?.isConnected !== true ||
    element.nodeType !== 1 ||
    element.namespaceURI !== "http://www.w3.org/1999/xhtml" ||
    !isProgrammaticallyFocusable(element) ||
    isDisabled(element)
  ) {
    return false;
  }
  const boundary = element.ownerDocument.documentElement;
  return !boundary || !isHidden(element, boundary, hasStyleSheets(element));
}

export function createAutoFocusEvent(
  type: FocusScopeAutoFocusEvent["type"],
  target: HTMLElement | null,
): FocusScopeAutoFocusEvent {
  let prevented = false;
  return Object.freeze({
    type,
    target,
    get defaultPrevented() {
      return prevented;
    },
    preventDefault: () => {
      prevented = true;
    },
  });
}
