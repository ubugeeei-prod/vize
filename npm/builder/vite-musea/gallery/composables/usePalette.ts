import { ref, shallowRef, computed } from "vue";
import type { PaletteApiResponse, PaletteControl } from "../api";
import { fetchPalette } from "../api";
import {
  PALETTE_STATE_VERSION,
  clearSavedPaletteState,
  initialPaletteValues,
  readSavedPaletteState,
  restorePaletteState,
  writeSavedPaletteState,
  type CustomProp,
} from "./paletteState";

export function usePalette() {
  const palette = shallowRef<PaletteApiResponse | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  // These JSON snapshots are replaced on every edit. Deep proxies reinterpret
  // literal hasOwnProperty keys and make nested defaults uncloneable messages.
  const values = shallowRef<Record<string, unknown>>({});
  const customProps = shallowRef<CustomProp[]>([]);
  const deletedPaletteProps = ref<Set<string>>(new Set());
  let loadVersion = 0;

  const allControls = computed<PaletteControl[]>(() => {
    const paletteControls = (palette.value?.controls ?? []).filter(
      (c) => !deletedPaletteProps.value.has(c.name),
    );
    const custom: PaletteControl[] = customProps.value.map((cp) => ({
      name: cp.name,
      control: cp.control,
      default_value: cp.default_value,
      required: false,
      options: [],
    }));
    return [...paletteControls, ...custom];
  });

  const mergedValues = computed<Record<string, unknown>>(() => {
    return Object.fromEntries(
      Object.entries(values.value).filter(
        ([name]) => !deletedPaletteProps.value.has(name) || customPropNames.value.has(name),
      ),
    );
  });

  const customPropNames = computed<Set<string>>(
    () => new Set(customProps.value.map((cp) => cp.name)),
  );

  async function load(artPath: string) {
    const version = ++loadVersion;
    loading.value = true;
    error.value = null;
    try {
      const loadedPalette = await fetchPalette(artPath);
      if (version !== loadVersion) return;
      palette.value = loadedPalette;
      const saved = readSavedPaletteState(artPath);
      if (saved) {
        const restored = restorePaletteState(palette.value.controls, saved);
        values.value = restored.values;
        customProps.value = restored.customProps;
        deletedPaletteProps.value = restored.deletedPaletteProps;
      } else {
        values.value = initialPaletteValues(palette.value.controls);
        customProps.value = [];
        deletedPaletteProps.value = new Set();
      }
    } catch (e) {
      if (version !== loadVersion) return;
      error.value = e instanceof Error ? e.message : String(e);
      palette.value = null;
    } finally {
      if (version === loadVersion) loading.value = false;
    }
  }

  function setValue(name: string, value: unknown) {
    values.value = { ...values.value, [name]: value };
  }

  function setAllValues(newValues: Record<string, unknown>) {
    values.value = { ...newValues };
  }

  function addProp(name: string, controlKind: string, defaultValue: unknown) {
    // Duplicate check against palette props and custom props
    const paletteNames = new Set((palette.value?.controls ?? []).map((c) => c.name));
    if (paletteNames.has(name) && !deletedPaletteProps.value.has(name)) return;
    if (customPropNames.value.has(name)) return;

    customProps.value = [
      ...customProps.value,
      { name, control: controlKind, default_value: defaultValue },
    ];
    values.value = { ...values.value, [name]: defaultValue };
  }

  function removeProp(name: string) {
    if (customPropNames.value.has(name)) {
      // Remove custom prop entirely
      customProps.value = customProps.value.filter((cp) => cp.name !== name);
      const next = { ...values.value };
      delete next[name];
      values.value = next;
    } else {
      // Palette prop: mark as deleted
      const next = new Set(deletedPaletteProps.value);
      next.add(name);
      deletedPaletteProps.value = next;
    }
  }

  function resetValues() {
    if (!palette.value) return;
    values.value = initialPaletteValues(palette.value.controls);
    customProps.value = [];
    deletedPaletteProps.value = new Set();
  }

  function saveValues(artPath: string) {
    writeSavedPaletteState(artPath, {
      version: PALETTE_STATE_VERSION,
      values: values.value,
      customProps: customProps.value,
      deletedPaletteProps: [...deletedPaletteProps.value],
    });
  }

  function clearSavedValues(artPath: string) {
    clearSavedPaletteState(artPath);
  }

  return {
    palette,
    loading,
    error,
    values,
    customProps,
    deletedPaletteProps,
    allControls,
    mergedValues,
    customPropNames,
    load,
    setValue,
    setAllValues,
    addProp,
    removeProp,
    resetValues,
    saveValues,
    clearSavedValues,
  };
}
