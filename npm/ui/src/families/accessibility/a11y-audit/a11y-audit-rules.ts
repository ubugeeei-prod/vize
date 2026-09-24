import type { A11yAuditIssue, A11yAuditOptions, A11yAuditRule } from "./a11y-audit-types.ts";

/** Diagnostic prefix shared by auditor messages. */
export const a11yAuditDiagnostic = "VIZE_UI_A11Y_AUDIT";

const interactiveRoles = new Set([
  "button",
  "checkbox",
  "combobox",
  "link",
  "menuitem",
  "menuitemcheckbox",
  "menuitemradio",
  "option",
  "radio",
  "searchbox",
  "slider",
  "spinbutton",
  "switch",
  "tab",
  "textbox",
  "treeitem",
]);
const namedLandmarkRoles = new Set(["region", "form"]);
const repeatableLandmarkRoles = new Set(["complementary", "navigation", "search"]);
const idrefAttributes = [
  "aria-controls",
  "aria-describedby",
  "aria-labelledby",
  "aria-owns",
  "aria-activedescendant",
] as const;

function implicitRole(element: Element): string | null {
  const tag = element.tagName.toLowerCase();
  if (tag === "button") return "button";
  if (tag === "a" && element.hasAttribute("href")) return "link";
  if (tag === "select") return "combobox";
  if (tag === "textarea") return "textbox";
  if (tag === "nav") return "navigation";
  if (tag === "aside") return "complementary";
  if (tag === "section" && hasExplicitName(element)) return "region";
  if (tag === "dialog") return "dialog";
  if (tag === "input") {
    const type = (element.getAttribute("type") ?? "text").toLowerCase();
    if (type === "hidden") return null;
    if (type === "checkbox" || type === "radio") return type;
    if (type === "range") return "slider";
    if (type === "button" || type === "submit" || type === "reset") return "button";
    return "textbox";
  }
  return null;
}

function roleOf(element: Element): string | null {
  const explicit = element.getAttribute("role")?.trim().split(/\s+/)[0];
  return explicit && explicit.length > 0 ? explicit : implicitRole(element);
}

function hasExplicitName(element: Element): boolean {
  return (
    (element.getAttribute("aria-label")?.trim().length ?? 0) > 0 ||
    labelledbyText(element).length > 0
  );
}

function labelledbyText(element: Element): string {
  const ids = element.getAttribute("aria-labelledby")?.trim();
  if (!ids) return "";
  return ids
    .split(/\s+/)
    .map((id) => element.ownerDocument.getElementById(id)?.textContent ?? "")
    .join(" ")
    .trim();
}

function isHidden(element: Element): boolean {
  return element.closest("[hidden], [aria-hidden='true'], [inert]") !== null;
}

/** Approximate the accessible name from ARIA, labels, alt text, title, and content. */
export function accessibleNameOf(element: Element): string {
  const labelledby = labelledbyText(element);
  if (labelledby) return labelledby;
  const label = element.getAttribute("aria-label")?.trim();
  if (label) return label;
  const id = element.getAttribute("id");
  if (id) {
    for (const node of element.ownerDocument.querySelectorAll("label[for]")) {
      if (node.getAttribute("for") === id && node.textContent?.trim())
        return node.textContent.trim();
    }
  }
  const wrapping = element.closest("label");
  if (wrapping?.textContent?.trim()) return wrapping.textContent.trim();
  const alt = element.getAttribute("alt")?.trim();
  if (alt) return alt;
  const title = element.getAttribute("title")?.trim();
  if (title) return title;
  const tag = element.tagName.toLowerCase();
  if (tag === "input" || tag === "textarea" || tag === "select") {
    return element.getAttribute("placeholder")?.trim() ?? "";
  }
  const images = [...element.querySelectorAll("img[alt]")]
    .map((image) => image.getAttribute("alt") ?? "")
    .join(" ");
  return `${element.textContent ?? ""} ${images}`.trim();
}

function partOf(element: Element): string | null {
  return element.closest("[data-vize-ui]")?.getAttribute("data-vize-ui") ?? null;
}

function describe(element: Element): string {
  const part = partOf(element);
  const tag = element.tagName.toLowerCase();
  return part ? `<${tag}> in ${part}` : `<${tag}>`;
}

function issue(rule: A11yAuditRule, element: Element, message: string): A11yAuditIssue {
  return Object.freeze({ element, message, part: partOf(element), rule });
}

/**
 * Audit a rendered subtree for common accessibility mistakes.
 *
 * The audit reads the live DOM only; it never mutates it. It checks accessible
 * names on interactive parts, dialogs, and landmarks that need one, `alt` on
 * images, and ARIA id references that point at nothing.
 */
export function auditAccessibility(
  root: Element,
  options: A11yAuditOptions = {},
): readonly A11yAuditIssue[] {
  if (!(root instanceof Element)) {
    throw new TypeError(`${a11yAuditDiagnostic}: root must be an Element`);
  }
  const ignore = new Set(options.ignore ?? []);
  const onlyUiParts = options.onlyUiParts ?? true;
  const issues: A11yAuditIssue[] = [];
  const candidates = [root, ...root.querySelectorAll("*")];
  const landmarkCounts = new Map<string, number>();
  for (const element of candidates) {
    const role = roleOf(element);
    if (role && repeatableLandmarkRoles.has(role) && !isHidden(element)) {
      landmarkCounts.set(role, (landmarkCounts.get(role) ?? 0) + 1);
    }
  }

  for (const element of candidates) {
    if (onlyUiParts && partOf(element) === null) continue;
    if (isHidden(element)) continue;
    const role = roleOf(element);
    const add = (rule: A11yAuditRule, message: string) => {
      if (!ignore.has(rule)) issues.push(issue(rule, element, message));
    };

    if (role && interactiveRoles.has(role) && accessibleNameOf(element) === "") {
      add(
        "accessible-name",
        `${describe(element)} with role "${role}" has no accessible name; add visible text, aria-label, or aria-labelledby.`,
      );
    }
    if ((role === "dialog" || role === "alertdialog") && !hasExplicitName(element)) {
      add(
        "dialog-name",
        `${describe(element)} with role "${role}" needs aria-label or aria-labelledby (usually a title).`,
      );
    }
    if (
      role &&
      namedLandmarkRoles.has(role) &&
      element.hasAttribute("role") &&
      !hasExplicitName(element)
    ) {
      add("landmark-name", `${describe(element)} with role "${role}" must be labelled.`);
    }
    if (
      role &&
      repeatableLandmarkRoles.has(role) &&
      (landmarkCounts.get(role) ?? 0) > 1 &&
      !hasExplicitName(element)
    ) {
      add(
        "landmark-name",
        `${describe(element)} is one of several "${role}" landmarks; label each so they can be told apart.`,
      );
    }
    if (element.tagName.toLowerCase() === "img" && !element.hasAttribute("alt")) {
      add(
        "image-alt",
        `${describe(element)} has no alt attribute; use alt="" for decorative images.`,
      );
    }
    for (const attribute of idrefAttributes) {
      const value = element.getAttribute(attribute)?.trim();
      if (!value) continue;
      for (const id of value.split(/\s+/)) {
        if (element.ownerDocument.getElementById(id) === null) {
          add("dangling-idref", `${describe(element)} ${attribute} references missing id "${id}".`);
        }
      }
    }
  }
  return Object.freeze(issues);
}
