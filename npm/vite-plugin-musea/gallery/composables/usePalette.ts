import { ref, computed } from "vue";
import type { PaletteApiResponse, PaletteControl } from "../api";
import { fetchPalette } from "../api";

export interface CustomProp {
  name: string;
  control: string;
  default_value: unknown;
}

export function usePalette() {
  const palette = ref<PaletteApiResponse | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const values = ref<Record<string, unknown>>({});
  const customProps = ref<CustomProp[]>([]);
  const deletedPaletteProps = ref<Set<string>>(new Set());

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
    const result: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(values.value)) {
      if (!deletedPaletteProps.value.has(k)) {
        result[k] = v;
      }
    }
    return result;
  });

  const customPropNames = computed<Set<string>>(
    () => new Set(customProps.value.map((cp) => cp.name)),
  );

  async function load(artPath: string) {
    loading.value = true;
    error.value = null;
    try {
      palette.value = await fetchPalette(artPath);
      // Initialize values from defaults
      const initial: Record<string, unknown> = {};
      for (const control of palette.value.controls) {
        if (control.default_value !== undefined) {
          initial[control.name] = control.default_value;
        }
      }
      values.value = initial;
      customProps.value = [];
      deletedPaletteProps.value = new Set();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      palette.value = null;
    } finally {
      loading.value = false;
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
    const initial: Record<string, unknown> = {};
    for (const control of palette.value.controls) {
      if (control.default_value !== undefined) {
        initial[control.name] = control.default_value;
      }
    }
    values.value = initial;
    customProps.value = [];
    deletedPaletteProps.value = new Set();
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
  };
}
