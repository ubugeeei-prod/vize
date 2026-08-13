export function normalizeStyleVirtualId(id: string): string {
  const withoutPrefix = id.startsWith("\0") ? id.slice(1) : id;
  if (!withoutPrefix.includes("?vue")) {
    return id;
  }

  return withoutPrefix.replace(/\.module\.\w+$/, "").replace(/\.\w+$/, "");
}
