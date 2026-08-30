import type { ScrollLockStrategy } from "./scroll-lock-types.ts";

interface StylePropertySnapshot {
  readonly name: string;
  readonly priority: string;
  readonly value: string;
}

interface ElementStyleSnapshot {
  readonly element: HTMLElement;
  readonly properties: readonly StylePropertySnapshot[];
}

export interface DocumentLockSnapshot {
  readonly attribute: string | null;
  readonly body: ElementStyleSnapshot;
  readonly root: ElementStyleSnapshot;
  readonly scrollX: number;
  readonly scrollY: number;
}

const rootProperties = [
  "--vize-scroll-lock-scrollbar-gap",
  "overflow",
  "overscroll-behavior",
  "padding-inline-end",
  "scrollbar-gutter",
] as const;
const bodyProperties = ["left", "position", "top", "width"] as const;

function captureStyle(element: HTMLElement, properties: readonly string[]): ElementStyleSnapshot {
  return {
    element,
    properties: properties.map((name) => ({
      name,
      priority: element.style.getPropertyPriority(name),
      value: element.style.getPropertyValue(name),
    })),
  };
}

function restoreStyle(snapshot: ElementStyleSnapshot): void {
  for (const { name, priority, value } of snapshot.properties) {
    if (value) snapshot.element.style.setProperty(name, value, priority);
    else snapshot.element.style.removeProperty(name);
  }
}

function setOwnedStyle(element: HTMLElement, name: string, value: string): void {
  element.style.setProperty(name, value, "important");
}

export function captureDocumentLock(document: Document): DocumentLockSnapshot | null {
  const root = document.documentElement as HTMLElement | null;
  const body = document.body;
  if (!root || !body) return null;
  const view = document.defaultView;
  return {
    attribute: root.getAttribute("data-vize-scroll-locked"),
    body: captureStyle(body, bodyProperties),
    root: captureStyle(root, rootProperties),
    scrollX: view?.scrollX ?? 0,
    scrollY: view?.scrollY ?? 0,
  };
}

export function measureScrollbarGap(document: Document): number {
  const view = document.defaultView;
  const clientWidth = document.documentElement?.clientWidth ?? 0;
  if (!view || clientWidth <= 0 || !Number.isFinite(view.innerWidth)) return 0;
  return Math.max(0, view.innerWidth - clientWidth);
}

export function resolveScrollLockStrategy(
  document: Document,
  strategy: ScrollLockStrategy,
): Exclude<ScrollLockStrategy, "auto"> {
  if (strategy !== "auto") return strategy;
  const navigator = document.defaultView?.navigator;
  const platform = navigator?.platform ?? "";
  const userAgent = navigator?.userAgent ?? "";
  const iOSLike =
    /^(iPad|iPhone|iPod)$/.test(platform) ||
    /\b(iPad|iPhone|iPod)\b/.test(userAgent) ||
    (platform === "MacIntel" && (navigator?.maxTouchPoints ?? 0) > 1);
  return iOSLike ? "fixed" : "overflow";
}

export function applyDocumentLock(
  snapshot: DocumentLockSnapshot,
  strategy: Exclude<ScrollLockStrategy, "auto">,
  gap: number,
  preserveGap: boolean,
): void {
  const root = snapshot.root.element;
  root.setAttribute("data-vize-scroll-locked", "");
  setOwnedStyle(root, "--vize-scroll-lock-scrollbar-gap", `${gap}px`);
  setOwnedStyle(root, "overflow", "hidden");
  setOwnedStyle(root, "overscroll-behavior", "none");
  if (preserveGap) {
    const supportsGutter = root.ownerDocument.defaultView?.CSS?.supports?.(
      "scrollbar-gutter",
      "stable",
    );
    if (supportsGutter) setOwnedStyle(root, "scrollbar-gutter", "stable");
    else if (gap > 0) {
      const computed = root.ownerDocument.defaultView?.getComputedStyle(root).paddingInlineEnd;
      const padding = Number.parseFloat(computed ?? "0");
      setOwnedStyle(
        root,
        "padding-inline-end",
        `${(Number.isFinite(padding) ? padding : 0) + gap}px`,
      );
    }
  }
  if (strategy === "fixed") {
    const body = snapshot.body.element;
    setOwnedStyle(body, "position", "fixed");
    setOwnedStyle(body, "top", `${-snapshot.scrollY}px`);
    setOwnedStyle(body, "left", `${-snapshot.scrollX}px`);
    setOwnedStyle(body, "width", "100%");
  }
}

export function restoreDocumentLock(snapshot: DocumentLockSnapshot): void {
  restoreStyle(snapshot.body);
  restoreStyle(snapshot.root);
  if (snapshot.attribute === null) snapshot.root.element.removeAttribute("data-vize-scroll-locked");
  else snapshot.root.element.setAttribute("data-vize-scroll-locked", snapshot.attribute);
}

export function restoreDocumentScroll(snapshot: DocumentLockSnapshot): void {
  const view = snapshot.root.element.ownerDocument.defaultView;
  if (!view || typeof view.scrollTo !== "function") return;
  try {
    view.scrollTo({ behavior: "instant", left: snapshot.scrollX, top: snapshot.scrollY });
  } catch {
    // Host environments may expose a non-functional scrollTo stub; styles must still restore.
  }
}
