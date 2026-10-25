// The runtime is an independent package module, so a stylesheet invalidation
// cannot recreate its records. Weak keys allow disposed components to be freed.
const records = new WeakMap<object, Map<string, object>>();

/** @internal One new token for each actual component module evaluation. */
export function remember(component: unknown, identity: string): void {
  if ((typeof component !== "object" || component === null) && typeof component !== "function")
    return;
  let modules = records.get(component);
  if (!modules) records.set(component, (modules = new Map()));
  modules.set(identity, {});
}

/** @internal Capture the token before an update can replace it. */
export function read(component: unknown, identity: string): object | undefined {
  if ((typeof component !== "object" || component === null) && typeof component !== "function")
    return;
  return records.get(component)?.get(identity);
}
