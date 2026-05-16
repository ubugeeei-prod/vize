import { ref } from "vue";
import type { LocaleInfo } from "../../wasm/index";

export type LocaleCode = "en" | "ja" | "zh";

const LOCALE_STORAGE_KEY = "vize-patina-locale";

export function useLocale(onLocaleChange: () => void) {
  const locales = ref<LocaleInfo[]>([
    { code: "en", name: "English" },
    { code: "ja", name: "日本語" },
    { code: "zh", name: "中文" },
  ]);

  const currentLocale = ref<LocaleCode>("en");

  /** Load saved locale preference from localStorage */
  function loadLocaleConfig() {
    try {
      const saved = localStorage.getItem(LOCALE_STORAGE_KEY);
      if (saved && ["en", "ja", "zh"].includes(saved)) {
        currentLocale.value = saved as LocaleCode;
      }
    } catch (e) {
      console.warn("Failed to load locale config:", e);
    }
  }

  /** Save locale preference to localStorage */
  function saveLocaleConfig() {
    try {
      localStorage.setItem(LOCALE_STORAGE_KEY, currentLocale.value);
    } catch (e) {
      console.warn("Failed to save locale config:", e);
    }
  }

  /** Change locale and re-lint */
  function setLocale(locale: LocaleCode) {
    currentLocale.value = locale;
    saveLocaleConfig();
    onLocaleChange();
  }

  return {
    locales,
    currentLocale,
    loadLocaleConfig,
    saveLocaleConfig,
    setLocale,
  };
}
