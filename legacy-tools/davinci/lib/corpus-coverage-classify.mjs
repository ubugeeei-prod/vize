// Tag and attribute classification for the corpus construct-coverage scan
// (Davinci P0-6). Every decision here is lexical: the scanners hand over a
// start-tag name or a raw attribute name and get back the taxonomy row the
// occurrence belongs to, with no scope or type information involved.

// SVG/MathML-namespace child tags that do not collide with an HTML element
// name; ambiguous names (a, title, style, script, set, ...) stay `native`.
const SVG_ONLY_TAGS = new Set([
  "animate",
  "animateMotion",
  "animateTransform",
  "circle",
  "clipPath",
  "defs",
  "desc",
  "ellipse",
  "feBlend",
  "feColorMatrix",
  "feComponentTransfer",
  "feComposite",
  "feConvolveMatrix",
  "feDiffuseLighting",
  "feDisplacementMap",
  "feDistantLight",
  "feDropShadow",
  "feFlood",
  "feFuncA",
  "feFuncB",
  "feFuncG",
  "feFuncR",
  "feGaussianBlur",
  "feImage",
  "feMerge",
  "feMergeNode",
  "feMorphology",
  "feOffset",
  "fePointLight",
  "feSpecularLighting",
  "feSpotLight",
  "feTile",
  "feTurbulence",
  "foreignObject",
  "g",
  "linearGradient",
  "marker",
  "mask",
  "metadata",
  "mpath",
  "path",
  "pattern",
  "polygon",
  "polyline",
  "radialGradient",
  "rect",
  "stop",
  "symbol",
  "text",
  "textPath",
  "tspan",
  "use",
  "view",
]);
const MATHML_ONLY_TAGS = new Set([
  "annotation",
  "annotation-xml",
  "maction",
  "merror",
  "mfrac",
  "mi",
  "mmultiscripts",
  "mn",
  "mo",
  "mover",
  "mpadded",
  "mphantom",
  "mprescripts",
  "mroot",
  "mrow",
  "ms",
  "mspace",
  "msqrt",
  "mstyle",
  "msub",
  "msubsup",
  "msup",
  "mtable",
  "mtd",
  "mtext",
  "mtr",
  "munder",
  "munderover",
  "semantics",
]);

export function classifyTag(tag) {
  if (tag === "slot") return "slot";
  if (tag === "template") return "template";
  if (tag === "svg" || SVG_ONLY_TAGS.has(tag)) return "svg";
  if (tag === "math" || MATHML_ONLY_TAGS.has(tag)) return "mathml";
  if (/^[A-Z]/.test(tag) || tag.includes("-")) return "component";
  return "native";
}

const BUILTIN_DIRECTIVES = new Set([
  "v-if",
  "v-else-if",
  "v-else",
  "v-for",
  "v-on",
  "v-bind",
  "v-model",
  "v-show",
  "v-html",
  "v-text",
  "v-once",
  "v-memo",
  "v-cloak",
  "v-pre",
]);
export const EVENT_MODIFIERS = new Set(["stop", "prevent", "capture", "self", "once", "passive"]);
export const KEY_MODIFIERS = new Set([
  "enter",
  "tab",
  "delete",
  "esc",
  "space",
  "up",
  "down",
  "left",
  "right",
  "ctrl",
  "alt",
  "shift",
  "meta",
  "exact",
]);
// Mouse-button modifiers are `left`/`right` (shared with the key class,
// disambiguated by event name below) plus `middle` (unambiguous).
export const MOUSE_EVENTS = new Set(["click", "dblclick", "mousedown", "mouseup", "contextmenu"]);
const BIND_MODIFIERS = new Set(["prop", "camel", "attr"]);
const MODEL_MODIFIERS = new Set(["lazy", "number", "trim"]);

/**
 * Classify one modifier token against the directive it modifies.
 *
 * `arg` is the directive argument (the event name for `v-on`), needed to
 * disambiguate `left`/`right`, which are key modifiers everywhere except on
 * mouse events.
 */
export function classifyModifier(directive, arg, modifier) {
  if (directive === "v-on") {
    if (EVENT_MODIFIERS.has(modifier)) return "event";
    if (modifier === "middle") return "mouse-button";
    if ((modifier === "left" || modifier === "right") && MOUSE_EVENTS.has(arg.toLowerCase())) {
      return "mouse-button";
    }
    if (KEY_MODIFIERS.has(modifier)) return "key";
    // Custom key aliases and unknown tokens are ignored (see "Skipped").
    return null;
  }
  if (directive === "v-bind" && BIND_MODIFIERS.has(modifier)) return "v-bind";
  if (directive === "v-model" && MODEL_MODIFIERS.has(modifier)) return "v-model";
  return null;
}

/**
 * Classify one attribute name into `{ directive, modifierClasses, vSlot }`.
 * `directive` is a taxonomy [[directive]] id or null; `vSlot` marks v-slot /
 * `#` shorthand occurrences, which have no taxonomy row.
 */
export function classifyAttribute(rawName) {
  let name = rawName;
  let directive = null;
  let vSlot = false;

  if (name.startsWith("@")) {
    directive = "v-on";
    name = name.slice(1);
  } else if (name.startsWith(":")) {
    directive = "v-bind";
    name = name.slice(1);
  } else if (name.startsWith("#")) {
    vSlot = true;
    name = name.slice(1);
  } else if (name.startsWith(".")) {
    // `.prop`-shorthand binding (`.innerHTML="x"`).
    return { directive: "v-bind", modifierClasses: ["v-bind"], vSlot: false };
  } else if (name.startsWith("v-")) {
    const base = name.split(":", 1)[0].split(".", 1)[0];
    if (base === "v-slot") {
      vSlot = true;
    } else if (BUILTIN_DIRECTIVES.has(base)) {
      directive = base;
    } else {
      directive = "custom";
    }
    name = name.slice(base.length);
    if (name.startsWith(":")) name = name.slice(1);
  } else {
    return null;
  }

  const segments = name.split(".");
  const arg = segments[0] ?? "";
  const modifierClasses = [];
  for (const modifier of segments.slice(1).filter((segment) => segment.length > 0)) {
    const modifierClass = classifyModifier(directive, arg, modifier);
    if (modifierClass) modifierClasses.push(modifierClass);
  }
  return { directive, modifierClasses, vSlot };
}
