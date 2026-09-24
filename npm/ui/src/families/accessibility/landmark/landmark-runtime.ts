import {
  getCurrentInstance,
  getCurrentScope,
  onMounted,
  onScopeDispose,
  shallowReadonly,
  shallowRef,
  toValue,
  watch,
} from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { createContext } from "../../foundations/context/context.ts";
import type {
  LandmarkElementMap,
  LandmarkInfo,
  LandmarkKeyBinding,
  LandmarkNavigationController,
  LandmarkNavigationOptions,
  LandmarkRegistrationInput,
  LandmarkRole,
  NamedLandmarkRole,
} from "./landmark-types.ts";

const setupDiagnostic = "VIZE_UI_LANDMARK_SETUP";
const focusMarker = "data-vize-landmark-focus";

/** Native element rendered for each landmark role. */
export const landmarkElements: LandmarkElementMap = Object.freeze({
  banner: "header",
  complementary: "aside",
  contentinfo: "footer",
  form: "form",
  main: "main",
  navigation: "nav",
  region: "section",
  search: "search",
});

const landmarkRoles: ReadonlySet<string> = new Set(Object.keys(landmarkElements));
const namedLandmarkRoles: ReadonlySet<string> = new Set<NamedLandmarkRole>([
  "complementary",
  "form",
  "navigation",
  "region",
  "search",
]);
const sectioningAncestors = "article, aside, main, nav, section";
const discoverySelector = "main, nav, aside, header, footer, section, form, search, [role]";

/** Default key that moves to the next landmark. */
export const defaultLandmarkNextKey: LandmarkKeyBinding = Object.freeze({ key: "F6" });

/** Default key that moves to the previous landmark. */
export const defaultLandmarkPreviousKey: LandmarkKeyBinding = Object.freeze({
  key: "F6",
  shiftKey: true,
});

/** Whether a role must carry an accessible name to be distinguishable. */
export function isNamedLandmarkRole(role: LandmarkRole): role is NamedLandmarkRole {
  return namedLandmarkRoles.has(role);
}

function isLandmarkRole(value: string): value is LandmarkRole {
  return landmarkRoles.has(value);
}

/** Resolve the accessible name of a landmark from `aria-label` or `aria-labelledby`. */
export function landmarkLabelOf(element: Element): string | null {
  const label = element.getAttribute("aria-label")?.trim();
  if (label) return label;
  const labelledby = element.getAttribute("aria-labelledby")?.trim();
  if (!labelledby) return null;
  const text = labelledby
    .split(/\s+/)
    .map((id) => element.ownerDocument.getElementById(id)?.textContent?.trim() ?? "")
    .filter((part) => part.length > 0)
    .join(" ");
  return text.length > 0 ? text : null;
}

/** Resolve the landmark role an element exposes, or `null` when it is not a landmark. */
export function landmarkRoleOf(element: Element): LandmarkRole | null {
  const explicit = element.getAttribute("role")?.trim().split(/\s+/)[0];
  if (explicit) return isLandmarkRole(explicit) ? explicit : null;
  const tag = element.localName;
  if (tag === "main") return "main";
  if (tag === "nav") return "navigation";
  if (tag === "aside") return "complementary";
  if (tag === "search") return "search";
  if (tag === "header" || tag === "footer") {
    if (element.parentElement?.closest(sectioningAncestors)) return null;
    return tag === "header" ? "banner" : "contentinfo";
  }
  if (tag === "section" || tag === "form") {
    if (landmarkLabelOf(element) === null) return null;
    return tag === "section" ? "region" : "form";
  }
  return null;
}

function isHtmlElement(value: unknown): value is HTMLElement {
  return typeof HTMLElement !== "undefined" && value instanceof HTMLElement;
}

/** Whether a landmark can currently receive focus from landmark cycling. */
export function isLandmarkAvailable(element: HTMLElement): boolean {
  if (!element.isConnected) return false;
  if (element.closest("[hidden], [inert], [aria-hidden='true']")) return false;
  const view = element.ownerDocument.defaultView;
  return view?.getComputedStyle(element).display !== "none";
}

/**
 * Focus a landmark element. Elements outside the tab order temporarily get
 * `tabindex="-1"`, removed again when focus leaves.
 */
export function focusLandmarkElement(element: HTMLElement): boolean {
  if (!element.hasAttribute("tabindex")) {
    element.setAttribute("tabindex", "-1");
    element.setAttribute(focusMarker, "true");
    element.addEventListener(
      "blur",
      () => {
        if (element.getAttribute(focusMarker) !== "true") return;
        element.removeAttribute(focusMarker);
        element.removeAttribute("tabindex");
      },
      { once: true },
    );
  }
  element.focus();
  return element.ownerDocument.activeElement === element;
}

function toInfo(element: HTMLElement, role: LandmarkRole): LandmarkInfo {
  return Object.freeze({
    element,
    id: element.id || null,
    label: landmarkLabelOf(element),
    role,
  });
}

function compareDocumentOrder(left: LandmarkInfo, right: LandmarkInfo): number {
  if (left.element === right.element) return 0;
  const position = left.element.compareDocumentPosition(right.element);
  return position & 4 ? -1 : position & 2 ? 1 : 0;
}

function matchesBinding(event: KeyboardEvent, binding: LandmarkKeyBinding | null): boolean {
  if (binding === null) return false;
  return (
    event.key === binding.key &&
    event.shiftKey === (binding.shiftKey ?? false) &&
    event.altKey === (binding.altKey ?? false) &&
    event.ctrlKey === (binding.ctrlKey ?? false) &&
    event.metaKey === (binding.metaKey ?? false)
  );
}

/** Create a keyboard landmark navigator. SSR-safe: nothing touches the DOM until used. */
export function createLandmarkNavigation(
  options: LandmarkNavigationOptions = {},
): LandmarkNavigationController {
  const registry = createCollectionRegistry<string, LandmarkRole>();
  const landmarks = shallowRef<readonly LandmarkInfo[]>(Object.freeze([]));
  const nextKey = (): LandmarkKeyBinding | null => {
    const value = toValue(options.nextKey);
    return value === undefined ? defaultLandmarkNextKey : value;
  };
  const previousKey = (): LandmarkKeyBinding | null => {
    const value = toValue(options.previousKey);
    return value === undefined ? defaultLandmarkPreviousKey : value;
  };
  let listeningDocument: Document | null = null;

  function resolveRoot(): HTMLElement | null {
    const root = toValue(options.root);
    if (root) return root;
    return typeof document === "undefined" ? null : document.body;
  }

  function refresh(): readonly LandmarkInfo[] {
    const byElement = new Map<HTMLElement, LandmarkInfo>();
    for (const item of registry.items.value) {
      if (isHtmlElement(item.element)) {
        byElement.set(item.element, toInfo(item.element, item.value));
      }
    }
    const root = resolveRoot();
    if (toValue(options.discover) === true && root) {
      const candidates = [root, ...root.querySelectorAll(discoverySelector)];
      for (const candidate of candidates) {
        if (!isHtmlElement(candidate) || byElement.has(candidate)) continue;
        const role = landmarkRoleOf(candidate);
        if (role !== null) byElement.set(candidate, toInfo(candidate, role));
      }
    }
    landmarks.value = Object.freeze([...byElement.values()].sort(compareDocumentOrder));
    return landmarks.value;
  }

  function available(): readonly LandmarkInfo[] {
    return refresh().filter((landmark) => isLandmarkAvailable(landmark.element));
  }

  function currentIndex(list: readonly LandmarkInfo[]): number {
    const active = typeof document === "undefined" ? null : document.activeElement;
    if (!active) return -1;
    let index = -1;
    list.forEach((landmark, candidate) => {
      if (landmark.element.contains(active)) index = candidate;
    });
    return index;
  }

  function move(step: 1 | -1, event: KeyboardEvent | null): LandmarkInfo | null {
    const list = available();
    if (list.length === 0) return null;
    const index = currentIndex(list);
    const nextIndex =
      index === -1
        ? step === 1
          ? 0
          : list.length - 1
        : (index + step + list.length) % list.length;
    const target = list[nextIndex];
    if (!target) return null;
    focusLandmarkElement(target.element);
    options.onNavigate?.(target, event);
    return target;
  }

  function focusLandmark(idOrRole: string): LandmarkInfo | null {
    const target =
      available().find((landmark) => landmark.id === idOrRole) ??
      available().find((landmark) => landmark.role === idOrRole) ??
      null;
    if (target === null) return null;
    focusLandmarkElement(target.element);
    options.onNavigate?.(target, null);
    return target;
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.defaultPrevented || toValue(options.enabled) === false) return;
    const step = matchesBinding(event, nextKey())
      ? 1
      : matchesBinding(event, previousKey())
        ? -1
        : 0;
    if (step === 0) return;
    if (move(step, event) !== null) event.preventDefault();
  }

  function attach(): void {
    const root = resolveRoot();
    const ownerDocument = root?.ownerDocument ?? null;
    if (ownerDocument === listeningDocument) return;
    detach();
    listeningDocument = ownerDocument;
    listeningDocument?.addEventListener("keydown", handleKeydown);
    refresh();
  }

  function detach(): void {
    listeningDocument?.removeEventListener("keydown", handleKeydown);
    listeningDocument = null;
  }

  function register(input: LandmarkRegistrationInput): () => void {
    const registration = registry.register({
      element: input.element,
      key: input.id,
      value: input.role,
    });
    return () => {
      registration.unregister();
    };
  }

  const stopItems = watch(registry.items, () => {
    if (listeningDocument !== null) refresh();
  });

  return Object.freeze({
    attach,
    detach,
    dispose: () => {
      detach();
      stopItems();
    },
    focusLandmark,
    focusNext: (event: KeyboardEvent | null = null) => move(1, event),
    focusPrevious: (event: KeyboardEvent | null = null) => move(-1, event),
    handleKeydown,
    landmarks: shallowReadonly(landmarks),
    refresh,
    register,
  });
}

/**
 * Create a landmark navigator bound to the current Vue scope. The keyboard
 * listener attaches after mount inside components, or immediately in a
 * client-side effect scope, and is removed with the scope.
 */
export function useLandmarkNavigation(
  options: LandmarkNavigationOptions = {},
): LandmarkNavigationController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createLandmarkNavigation(options);
  if (getCurrentInstance()) onMounted(controller.attach);
  else if (typeof document !== "undefined") controller.attach();
  onScopeDispose(controller.dispose);
  return controller;
}

/** Landmark registry published by LandmarkProvider. */
export const landmarkContext = createContext<LandmarkNavigationController>("LandmarkProvider");
