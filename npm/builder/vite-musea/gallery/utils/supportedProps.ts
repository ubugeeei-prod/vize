/** Retain raw editor values separately from props the component can apply. */
export function supportedProps(
  values: Record<string, unknown>,
  unsupported: ReadonlySet<string>,
): Record<string, unknown> {
  return Object.fromEntries(Object.entries(values).filter(([name]) => !unsupported.has(name)));
}

/** Ordinary JSON edits may retain or remove an unsupported value, but not apply a new one. */
export function unsupportedPropEdit(
  previous: Record<string, unknown>,
  next: Record<string, unknown>,
  unsupported: ReadonlySet<string>,
): string | undefined {
  for (const name of unsupported) {
    if (
      Object.hasOwn(next, name) &&
      (!Object.hasOwn(previous, name) ||
        JSON.stringify(next[name]) !== JSON.stringify(previous[name]))
    ) {
      return `${name} cannot be applied by Vue. Keep its retained value or remove it.`;
    }
  }
  return undefined;
}
