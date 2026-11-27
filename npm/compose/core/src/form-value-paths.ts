/**
 * Dotted-path access into form values.
 *
 * `useForm` addresses every field by a dotted path (`"address.city"`,
 * `"tags.0"`). Paths only walk plain objects and arrays, and a segment that
 * names the prototype chain makes the whole path unreadable and unwritable,
 * so a field path can never reach or pollute `Object.prototype`.
 */

type Container = Record<string, unknown> | unknown[];

/** Whether a value is a plain object or an array that a path may descend into. */
export function isContainer(value: unknown): value is Container {
  if (Array.isArray(value)) return true;
  if (typeof value !== "object" || value === null) return false;
  const prototype: unknown = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

/** Keys that reach an object's prototype chain instead of its own data. */
const prototypeKeys: ReadonlySet<string> = new Set(["__proto__", "constructor", "prototype"]);

/** Split a dotted path; undefined when a segment names the prototype chain. */
function splitPath(path: string): readonly string[] | undefined {
  if (path === "") return [];
  const keys = path.split(".");
  return keys.some((key) => prototypeKeys.has(key)) ? undefined : keys;
}

function readChild(container: Container, key: string): unknown {
  if (Array.isArray(container)) return container[Number(key)];
  // Read first so a reactive container tracks the key even while it is absent.
  const value = container[key];
  return Object.hasOwn(container, key) ? value : undefined;
}

function writeChild(container: Container, key: string, value: unknown): void {
  if (Array.isArray(container)) container[Number(key)] = value;
  else container[key] = value;
}

/** Read a dotted path. */
export function readPath(root: unknown, path: string): unknown {
  const keys = splitPath(path);
  if (keys === undefined) return undefined;
  let current = root;
  for (const key of keys) {
    if (!isContainer(current)) return undefined;
    current = readChild(current, key);
  }
  return current;
}

/** Write a dotted path, creating intermediate containers. */
export function writePath(root: unknown, path: string, value: unknown): void {
  const keys = splitPath(path);
  const last = keys?.at(-1);
  if (keys === undefined || last === undefined || !isContainer(root)) return;
  let current: Container = root;
  for (let index = 0; index < keys.length - 1; index += 1) {
    const key = keys[index] ?? "";
    let next = readChild(current, key);
    if (!isContainer(next)) {
      next = /^\d+$/.test(keys[index + 1] ?? "") ? [] : {};
      writeChild(current, key, next);
    }
    if (!isContainer(next)) return;
    current = next;
  }
  writeChild(current, last, value);
}
