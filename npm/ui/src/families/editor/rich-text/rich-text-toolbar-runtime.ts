import { onScopeDispose } from "vue";
import type { ShallowRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import type { RichTextContextValue, RichTextToolbarContextValue } from "./rich-text-context.ts";
import { richTextToolbarContext } from "./rich-text-context.ts";

/** Roving focus for a toolbar-like container of RichTextToolbarButtons. */
export function useRichTextToolbar(
  editor: RichTextContextValue,
  element: Readonly<ShallowRef<HTMLElement | null>>,
): {
  readonly context: RichTextToolbarContextValue;
  readonly onKeydown: (event: KeyboardEvent) => void;
} {
  const registry = createCollectionRegistry<string, null>({ disabledBehavior: "skip" });
  const context: RichTextToolbarContextValue = { registry };
  richTextToolbarContext.provide(context);

  function focusKey(key: string | null): boolean {
    if (key === null) return false;
    registry.setActiveKey(key);
    const target = registry.getItem(key)?.element;
    if (!(target instanceof HTMLElement)) return false;
    target.focus();
    return true;
  }

  const focusFirst = (): boolean =>
    focusKey(registry.activeKey.value ?? registry.getNavigationKey("first"));
  editor.toolbars.value = [...editor.toolbars.value, focusFirst];
  onScopeDispose(() => {
    editor.toolbars.value = editor.toolbars.value.filter((entry) => entry !== focusFirst);
  });

  function onKeydown(event: KeyboardEvent): void {
    const rtl = element.value?.closest("[dir]")?.getAttribute("dir") === "rtl";
    const forward = rtl ? "ArrowLeft" : "ArrowRight";
    const backward = rtl ? "ArrowRight" : "ArrowLeft";
    let key: string | null = null;
    if (event.key === forward) key = registry.getNavigationKey("next", { loop: true });
    else if (event.key === backward) key = registry.getNavigationKey("previous", { loop: true });
    else if (event.key === "Home") key = registry.getNavigationKey("first");
    else if (event.key === "End") key = registry.getNavigationKey("last");
    else if (event.key === "Escape") {
      event.preventDefault();
      editor.focus();
      return;
    } else return;
    event.preventDefault();
    focusKey(key);
  }

  return { context, onKeydown };
}
