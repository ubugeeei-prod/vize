import { deepActiveElement, focusElement } from "./focus-scope-dom.ts";
import { resolveMoveOptions } from "./focus-scope-internal.ts";
import type { FocusScopeManager, FocusScopeMoveOptions } from "./focus-scope-types.ts";

interface FocusScopeManagerConfig {
  readonly assertAlive: () => void;
  readonly list: (root: Element, options?: FocusScopeMoveOptions) => HTMLElement[];
  readonly root: () => Element | null;
}

/** Create stable traversal methods while keeping controller lifecycle orchestration focused. */
export function createFocusScopeManager({
  assertAlive,
  list,
  root,
}: FocusScopeManagerConfig): FocusScopeManager {
  const move = (
    position: "first" | "last" | "next" | "previous",
    values?: FocusScopeMoveOptions,
  ): HTMLElement | null => {
    assertAlive();
    const scopeRoot = root();
    if (!scopeRoot) return null;
    const options = resolveMoveOptions(values);
    const candidates = list(scopeRoot, options);
    let target: HTMLElement | undefined;
    if (position === "first") target = candidates[0];
    else if (position === "last") target = candidates.at(-1);
    else {
      const from = "from" in options ? options.from : deepActiveElement(scopeRoot.ownerDocument);
      const index = from ? candidates.indexOf(from as HTMLElement) : -1;
      if (position === "next") {
        target = candidates[index + 1] ?? (options.wrap ? candidates[0] : undefined);
      } else {
        target =
          index < 0
            ? candidates.at(-1)
            : (candidates[index - 1] ?? (options.wrap ? candidates.at(-1) : undefined));
      }
    }
    if (target) focusElement(target, options.preventScroll);
    return target ?? null;
  };

  return Object.freeze({
    focusFirst: (options: FocusScopeMoveOptions | undefined) => move("first", options),
    focusLast: (options: FocusScopeMoveOptions | undefined) => move("last", options),
    focusNext: (options: FocusScopeMoveOptions | undefined) => move("next", options),
    focusPrevious: (options: FocusScopeMoveOptions | undefined) => move("previous", options),
  });
}
