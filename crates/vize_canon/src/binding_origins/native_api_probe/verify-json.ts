/** Reject duplicate JSON keys and numeric identities JS cannot retain exactly. */
export function strictJson(source: string): unknown {
  let at = 0;
  const whitespace = () => {
    while (at < source.length && " \t\r\n".includes(source.charAt(at))) at++;
  };
  const requireCharacter = (character: string) => {
    whitespace();
    if (source.charAt(at++) !== character) throw new Error("invalid JSON structure");
  };
  const string = (): string => {
    const start = at;
    if (source.charAt(at++) !== '"') throw new Error("JSON key must be a string");
    while (at < source.length) {
      const character = source.charAt(at++);
      if (character === "\\") at++;
      else if (character === '"') {
        const result: unknown = JSON.parse(source.slice(start, at));
        if (typeof result !== "string") throw new Error("invalid JSON string");
        return result;
      }
    }
    throw new Error("truncated JSON string");
  };
  const value = (depth: number): void => {
    if (depth > 512) throw new Error("unsupported JSON nesting");
    whitespace();
    const character = source.charAt(at);
    if (character === '"') {
      string();
    } else if (character === "{") {
      at++;
      whitespace();
      const keys = new Set<string>();
      if (source.charAt(at) !== "}") {
        while (true) {
          whitespace();
          const key = string();
          if (keys.has(key)) throw new Error("duplicate JSON object key: " + key);
          keys.add(key);
          requireCharacter(":");
          value(depth + 1);
          whitespace();
          if (source.charAt(at) !== ",") break;
          at++;
        }
      }
      requireCharacter("}");
    } else if (character === "[") {
      at++;
      whitespace();
      if (source.charAt(at) !== "]") {
        while (true) {
          value(depth + 1);
          whitespace();
          if (source.charAt(at) !== ",") break;
          at++;
        }
      }
      requireCharacter("]");
    } else {
      const start = at;
      while (at < source.length && !" \t\r\n,]}".includes(source.charAt(at))) at++;
      if (start === at) throw new Error("missing JSON value");
      const scalar: unknown = JSON.parse(source.slice(start, at));
      if (
        typeof scalar === "number" &&
        (!Number.isFinite(scalar) || (Number.isInteger(scalar) && !Number.isSafeInteger(scalar)))
      ) {
        throw new Error("JSON number cannot be represented losslessly");
      }
    }
  };
  value(0);
  whitespace();
  if (at !== source.length) throw new Error("unexpected JSON suffix");
  const result: unknown = JSON.parse(source);
  return result;
}
