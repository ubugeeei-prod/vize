export function mapToObject(value: unknown): unknown {
  if (value instanceof Map) {
    const obj: Record<string, unknown> = {};
    value.forEach((v, k) => {
      obj[String(k)] = mapToObject(v);
    });
    return obj;
  }
  if (Array.isArray(value)) {
    return value.map(mapToObject);
  }
  if (value !== null && typeof value === "object") {
    const obj: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(value)) {
      obj[k] = mapToObject(v);
    }
    return obj;
  }
  return value;
}

export function filterAstProperties(obj: unknown, hideLoc: boolean, hideSource: boolean): unknown {
  if (obj === null || typeof obj !== "object") return obj;
  if (Array.isArray(obj)) {
    return obj.map((item) => filterAstProperties(item, hideLoc, hideSource));
  }
  const result: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(obj)) {
    if (hideLoc && key === "loc") continue;
    if (hideSource && key === "source") continue;
    result[key] = filterAstProperties(value, hideLoc, hideSource);
  }
  return result;
}
