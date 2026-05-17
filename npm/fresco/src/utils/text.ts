/**
 * Unicode-aware text helpers shared by input and text components.
 */

const segmenter =
  typeof Intl !== "undefined" && "Segmenter" in Intl
    ? new Intl.Segmenter(undefined, { granularity: "grapheme" })
    : null;

export function graphemes(value: string): string[] {
  if (!value) return [];

  if (segmenter) {
    return Array.from(segmenter.segment(value), (segment) => segment.segment);
  }

  return Array.from(value);
}

export function graphemeLength(value: string): number {
  return graphemes(value).length;
}

export function sliceGraphemes(value: string, start: number, end?: number): string {
  return graphemes(value).slice(start, end).join("");
}

export function insertAtGrapheme(value: string, index: number, text: string): string {
  return `${sliceGraphemes(value, 0, index)}${text}${sliceGraphemes(value, index)}`;
}

export function deleteGraphemeBefore(value: string, index: number): string {
  if (index <= 0) return value;
  return `${sliceGraphemes(value, 0, index - 1)}${sliceGraphemes(value, index)}`;
}

export function deleteGraphemeAt(value: string, index: number): string {
  const length = graphemeLength(value);
  if (index < 0 || index >= length) return value;
  return `${sliceGraphemes(value, 0, index)}${sliceGraphemes(value, index + 1)}`;
}

export function stringifyChildren(children: unknown): string {
  if (children == null || typeof children === "boolean") return "";
  if (typeof children === "string" || typeof children === "number") return String(children);
  if (Array.isArray(children)) return children.map((child) => stringifyChildren(child)).join("");

  if (typeof children === "object") {
    const vnode = children as {
      children?: unknown;
      props?: Record<string, unknown> | null;
      type?: { name?: string } | string;
    };
    const typeName = typeof vnode.type === "string" ? vnode.type : vnode.type?.name;

    if (typeName === "Newline") {
      const count = typeof vnode.props?.count === "number" ? vnode.props.count : 1;
      return "\n".repeat(Math.max(0, count));
    }

    if (typeof vnode.props?.text === "string" || typeof vnode.props?.text === "number") {
      return String(vnode.props.text);
    }

    if (typeof vnode.props?.content === "string" || typeof vnode.props?.content === "number") {
      return String(vnode.props.content);
    }

    return stringifyChildren(vnode.children);
  }

  return "";
}
