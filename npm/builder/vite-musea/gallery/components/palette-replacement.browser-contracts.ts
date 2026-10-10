import type { PaletteApiResponse } from "../api";
import { usePalette } from "../composables/usePalette";
import { getPaletteStateStorageKey, type CustomProp } from "../composables/paletteState";

export interface ReplacementFixture {
  palette: PaletteApiResponse;
  replacement: CustomProp;
  editedValue: number;
}

export async function runPaletteReplacement(input: ReplacementFixture) {
  const originalFetch = window.fetch;
  const artPath = "__palette_replacement__/component.vue";
  const storageKey = getPaletteStateStorageKey(artPath);
  const originalSaved = localStorage.getItem(storageKey);
  const snapshot = (state: ReturnType<typeof usePalette>) =>
    JSON.parse(
      JSON.stringify({
        controls: state.allControls.value,
        values: state.mergedValues.value,
        customProps: state.customProps.value,
        deletedPaletteProps: [...state.deletedPaletteProps.value],
      }),
    );
  window.fetch = (resource, options) => {
    const url =
      typeof resource === "string"
        ? resource
        : resource instanceof URL
          ? resource.href
          : resource.url;
    return url.includes("__palette_replacement__")
      ? Promise.resolve(new Response(JSON.stringify(input.palette)))
      : originalFetch(resource, options);
  };
  try {
    localStorage.removeItem(storageKey);
    const editor = usePalette();
    await editor.load(artPath);
    const { name, control, default_value } = input.replacement;
    editor.addProp(name, control, default_value);
    const undeleted = snapshot(editor);
    editor.removeProp(name);
    editor.addProp(name, control, default_value);
    const replacedDefault = snapshot(editor);
    editor.setValue(name, input.editedValue);
    const replacedEdit = snapshot(editor);
    editor.saveValues(artPath);
    const saved = JSON.parse(localStorage.getItem(storageKey)!);

    const restored = usePalette();
    await restored.load(artPath);
    const afterReload = snapshot(restored);
    restored.removeProp(name);
    const afterRemove = snapshot(restored);
    restored.saveValues(artPath);
    await restored.load(artPath);
    const afterRemovedReload = snapshot(restored);
    restored.resetValues();
    return {
      undeleted,
      replacedDefault,
      replacedEdit,
      saved,
      afterReload,
      afterRemove,
      afterRemovedReload,
      afterReset: snapshot(restored),
    };
  } finally {
    window.fetch = originalFetch;
    if (originalSaved === null) localStorage.removeItem(storageKey);
    else localStorage.setItem(storageKey, originalSaved);
  }
}
