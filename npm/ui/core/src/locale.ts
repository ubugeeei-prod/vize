export {
  localeContext,
  resolveDirection,
  useDirection,
  useLocale,
  type DirectionPreference,
  type LocaleValue,
  type TextDirection,
} from "./locale-runtime.ts";

/** Locale and direction provider for a document subtree. */
export { default as LocaleProvider } from "./locale-provider.vue";
