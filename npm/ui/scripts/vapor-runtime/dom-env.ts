/**
 * happy-dom browser globals for the Vapor lane: `window`, `document`,
 * `navigator`, and every DOM constructor Node lacks. Event constructors are
 * replaced as well, because events built with Node's classes cannot be
 * dispatched on happy-dom targets.
 */
import { Window } from "happy-dom";

const overridden = ["Event", "CustomEvent", "EventTarget", "MessageEvent", "ErrorEvent"] as const;
const browserOnly = ["window", "self", "document", "navigator"] as const;

/** Every global installed by {@link installDomGlobals} that Node lacks. */
const installedGlobals = new Set<string>(browserOnly);

/** Install the globals and return the window. */
export function installDomGlobals(): Window {
  const window = new Window({
    url: "https://vapor-conformance.vize.test/",
    width: 1024,
    height: 768,
  });
  const target: Record<string, unknown> = globalThis as unknown as Record<string, unknown>;
  for (const name of Object.getOwnPropertyNames(window)) {
    if (name in globalThis) continue;
    installedGlobals.add(name);
    const value: unknown = Reflect.get(window, name);
    target[name] = typeof value === "function" && !/^[A-Z]/.test(name) ? value.bind(window) : value;
  }
  for (const name of overridden) target[name] = Reflect.get(window, name);
  for (const name of browserOnly) {
    Object.defineProperty(globalThis, name, {
      configurable: true,
      writable: true,
      value: name === "window" || name === "self" ? window : Reflect.get(window, name),
    });
  }
  return window;
}

/**
 * Run `render` with every installed browser global removed (window, document,
 * navigator, and DOM constructors such as `HTMLElement`), so it sees exactly
 * Node's globals, as a real server render does.
 */
export async function withoutBrowserGlobals<Result>(
  render: () => Promise<Result>,
): Promise<Result> {
  const saved = new Map<string, PropertyDescriptor | undefined>();
  for (const name of installedGlobals) {
    saved.set(name, Object.getOwnPropertyDescriptor(globalThis, name));
    Reflect.deleteProperty(globalThis, name);
  }
  try {
    return await render();
  } finally {
    for (const [name, descriptor] of saved) {
      if (descriptor) Object.defineProperty(globalThis, name, descriptor);
    }
  }
}

const svgNamespace = "http://www.w3.org/2000/svg";

/** The HTML parser's "adjust SVG attributes" table: lowercase → SVG case. */
const svgAttributeCase = new Map(
  [
    "attributeName",
    "attributeType",
    "baseFrequency",
    "baseProfile",
    "calcMode",
    "clipPathUnits",
    "diffuseConstant",
    "edgeMode",
    "filterUnits",
    "glyphRef",
    "gradientTransform",
    "gradientUnits",
    "kernelMatrix",
    "kernelUnitLength",
    "keyPoints",
    "keySplines",
    "keyTimes",
    "lengthAdjust",
    "limitingConeAngle",
    "markerHeight",
    "markerUnits",
    "markerWidth",
    "maskContentUnits",
    "maskUnits",
    "numOctaves",
    "pathLength",
    "patternContentUnits",
    "patternTransform",
    "patternUnits",
    "pointsAtX",
    "pointsAtY",
    "pointsAtZ",
    "preserveAlpha",
    "preserveAspectRatio",
    "primitiveUnits",
    "refX",
    "refY",
    "repeatCount",
    "repeatDur",
    "requiredExtensions",
    "requiredFeatures",
    "specularConstant",
    "specularExponent",
    "spreadMethod",
    "startOffset",
    "stdDeviation",
    "stitchTiles",
    "surfaceScale",
    "systemLanguage",
    "tableValues",
    "targetX",
    "targetY",
    "textLength",
    "viewBox",
    "viewTarget",
    "xChannelSelector",
    "yChannelSelector",
    "zoomAndPan",
  ].map((name) => [name.toLowerCase(), name]),
);

/**
 * Restore SVG attribute case in markup parsed with `innerHTML`, as a browser's
 * HTML parser does and happy-dom does not. Vue's `ssrRenderAttrs` lowercases
 * attributes that fall through to an SVG root (`viewbox="0 0 1 1"`); a browser
 * still parses that as `viewBox`, which is what hydration then checks.
 */
export function adjustSvgAttributeCase(root: Element): void {
  for (const element of root.querySelectorAll("*")) {
    if (element.namespaceURI !== svgNamespace) continue;
    // A snapshot: renaming an attribute changes the live attribute map.
    for (const { name, value } of Array.from(element.attributes)) {
      const adjusted = svgAttributeCase.get(name);
      if (adjusted === undefined || adjusted === name) continue;
      element.removeAttribute(name);
      element.setAttribute(adjusted, value);
    }
  }
}
