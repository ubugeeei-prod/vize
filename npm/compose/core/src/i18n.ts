import { computed, inject, shallowReactive, shallowRef } from "vue";
import type { App, ComputedRef, InjectionKey, ShallowRef } from "vue";

import { createMessageFormatter } from "./icu-message.ts";
import type {
  FormatMessageOptions,
  MessageArgument,
  MessageArguments,
  MessageFormatter,
  ResolveMessageArguments,
} from "./icu-message.ts";
import { useLocale } from "./locale.ts";
import type { LocaleControls, TextDirection } from "./locale.ts";

/* -------------------------------------------------------------------------- */
/* Catalog types                                                              */
/* -------------------------------------------------------------------------- */

/** A nested message catalog: leaves are ICU message strings. */
export interface MessageTree {
  readonly [key: string]: string | MessageTree;
}

/** Catalogs keyed by locale. */
export type MessageCatalogs = Readonly<Record<string, MessageTree>>;

/** Dotted message keys of one catalog, e.g. `"home.title"`. */
export type MessageKeysOf<Tree> = {
  [Key in keyof Tree & string]: Tree[Key] extends string
    ? Key
    : `${Key}.${MessageKeysOf<Tree[Key]>}`;
}[keyof Tree & string];

/** Message literal stored at a dotted key. */
export type MessageAt<Tree, Path extends string> = Path extends `${infer Head}.${infer Rest}`
  ? Head extends keyof Tree
    ? MessageAt<Tree[Head], Rest>
    : never
  : Path extends keyof Tree
    ? Tree[Path] extends string
      ? Tree[Path]
      : never
    : never;

/** Every key present in any locale of a catalog set. */
export type MessageKey<Catalogs> = {
  [Locale in keyof Catalogs]: MessageKeysOf<Catalogs[Locale]>;
}[keyof Catalogs];

/**
 * Parameters `t(key)` needs: the arguments of every translation of `Key`,
 * merged (a typed occurrence in any locale wins over a plain `{name}`).
 */
export type MessageParamsFor<Catalogs, Key extends string> = ResolveMessageArguments<
  Extract<
    {
      [Locale in keyof Catalogs]: MessageAt<Catalogs[Locale], Key> extends infer Message extends
        string
        ? MessageArguments<Message>
        : never;
    }[keyof Catalogs],
    MessageArgument
  >
>;

/** Keys whose merged parameters contain an impossible (`never`) value. */
type ConflictingKeys<Catalogs> = {
  [Key in MessageKey<Catalogs> & string]: {
    [Param in keyof MessageParamsFor<Catalogs, Key>]: [
      MessageParamsFor<Catalogs, Key>[Param],
    ] extends [never]
      ? Param
      : never;
  }[keyof MessageParamsFor<Catalogs, Key>] extends never
    ? never
    : Key;
}[MessageKey<Catalogs> & string];

/**
 * Compile-time validation mixed into the `defineMessages` argument. A
 * locale that misses keys other locales define must carry a
 * `__missingTranslations` property (which it cannot), so the compiler
 * reports the missing keys; incompatible parameter types across locales are
 * reported the same way through `__incompatibleParams`.
 */
export type ValidateCatalogs<Catalogs> = {
  readonly [Locale in keyof Catalogs]: (Exclude<
    MessageKey<Catalogs>,
    MessageKeysOf<Catalogs[Locale]>
  > extends infer Missing
    ? [Missing] extends [never]
      ? unknown
      : { readonly __missingTranslations: Missing }
    : unknown) &
    ([ConflictingKeys<Catalogs>] extends [never]
      ? unknown
      : { readonly __incompatibleParams: ConflictingKeys<Catalogs> });
};

/** Same key structure as a catalog, with any string as message. */
export type MessageShape<Tree> = {
  readonly [Key in keyof Tree]: Tree[Key] extends string ? string : MessageShape<Tree[Key]>;
};

/** Catalog shape a lazily loaded locale must provide. */
export type LocaleMessages<Catalogs> = MessageShape<Catalogs[keyof Catalogs]>;

/**
 * Per-key check for {@link defineLocale}: a translation may only use
 * parameters the schema knows, with compatible types.
 */
type ValidateTranslation<Catalogs, Tree, Prefix extends string = ""> = {
  readonly [Key in keyof Tree & string]: Tree[Key] extends infer Message extends string
    ? keyof ResolveMessageArguments<MessageArguments<Message>> extends keyof MessageParamsFor<
        Catalogs,
        `${Prefix}${Key}`
      >
      ? MessageParamsFor<Catalogs, `${Prefix}${Key}`> extends ResolveMessageArguments<
          MessageArguments<Message>
        >
        ? Message
        : `incompatible parameter types for "${Prefix}${Key}"`
      : `unknown parameter in "${Prefix}${Key}"`
    : ValidateTranslation<Catalogs, Tree[Key], `${Prefix}${Key}.`>;
};

/* -------------------------------------------------------------------------- */
/* defineMessages / defineLocale                                              */
/* -------------------------------------------------------------------------- */

/**
 * Declare typed message catalogs for several locales.
 *
 * The literal messages are preserved, so {@link MessageParamsFor} knows the
 * ICU parameters of every key. The compiler rejects catalogs whose locales
 * do not share the same keys (`__missingTranslations`) or whose
 * translations need incompatible parameter types (`__incompatibleParams`).
 * Returns the catalogs unchanged; the function exists for inference and
 * validation only.
 *
 * @example
 * ```ts
 * const messages = defineMessages({
 *   en: { cart: "{count, plural, one {# item} other {# items}}" },
 *   ja: { cart: "{count, plural, other {# 個}}" },
 * });
 * ```
 *
 * @param catalogs Messages keyed by locale, then by (nested) key.
 * @throws `TypeError` tagged `VIZE_COMPOSE_I18N_INVALID_KEY` when a key
 * contains a dot (dots separate nested keys).
 * @returns The same catalogs.
 */
export function defineMessages<const Catalogs extends MessageCatalogs>(
  catalogs: Catalogs & ValidateCatalogs<Catalogs>,
): Catalogs {
  for (const tree of Object.values(catalogs)) assertKeys(tree, "");
  return catalogs;
}

/**
 * Declare one additional locale (typically a lazily loaded module) against
 * the catalogs produced by {@link defineMessages}.
 *
 * The keys must match exactly, and every translation may only use
 * parameters the schema declares, with compatible types.
 *
 * @example
 * ```ts
 * // fr.ts
 * export default defineLocale<typeof messages>()({ cart: "{count, plural, one {# article} other {# articles}}" });
 * ```
 *
 * @returns A function that validates and returns the locale catalog.
 */
export function defineLocale<Catalogs extends MessageCatalogs>(): <
  const Tree extends LocaleMessages<Catalogs>,
>(
  catalog: Tree & ValidateTranslation<Catalogs, Tree>,
) => Tree {
  return (catalog) => {
    assertKeys(catalog, "");
    return catalog;
  };
}

function assertKeys(tree: MessageTree, prefix: string): void {
  for (const [key, value] of Object.entries(tree)) {
    if (key.includes(".")) {
      throw new TypeError(
        `[VIZE_COMPOSE_I18N_INVALID_KEY] message keys must not contain "."; received "${prefix}${key}"`,
      );
    }
    if (typeof value !== "string") assertKeys(value, `${prefix}${key}.`);
  }
}

/* -------------------------------------------------------------------------- */
/* Runtime                                                                    */
/* -------------------------------------------------------------------------- */

/** Loader returning the catalog of a lazily loaded locale. */
export type LocaleLoader<Catalogs> = () => Promise<
  LocaleMessages<Catalogs> | { readonly default: LocaleMessages<Catalogs> }
>;

/** Static configuration shared by every request, passed to {@link defineI18n}. */
export interface DefineI18nOptions<
  Catalogs extends MessageCatalogs,
  Lazy extends string,
> extends FormatMessageOptions {
  /** Eagerly bundled catalogs (see {@link defineMessages}). */
  readonly messages: Catalogs;

  /**
   * Locale used when a key is missing in the active locale.
   *
   * @default the first eager locale
   */
  readonly fallbackLocale?: keyof Catalogs & string;

  /**
   * Loaders for locales fetched on demand.
   *
   * @default {}
   */
  readonly loaders?: { readonly [Locale in Lazy]: LocaleLoader<Catalogs> };

  /**
   * Called when a key is missing in the active and fallback locales; the
   * key itself is rendered.
   *
   * @default undefined
   */
  readonly onMissing?: (key: string, locale: string) => void;
}

/** Arguments of `t(key, …)`: the params object is optional only when empty. */
export type TranslateArguments<Params> = {} extends Params ? [params?: Params] : [params: Params];

/** Per-request i18n state and typed translation functions. */
export interface I18n<Catalogs extends MessageCatalogs, Lazy extends string> {
  /** Active locale. Change it with {@link I18n.setLocale}. */
  readonly locale: Readonly<ShallowRef<(keyof Catalogs & string) | Lazy>>;

  /** Every locale this instance can switch to. */
  readonly availableLocales: readonly ((keyof Catalogs & string) | Lazy)[];

  /** Locales whose messages are available right now. */
  readonly loadedLocales: ComputedRef<((keyof Catalogs & string) | Lazy)[]>;

  /** Whether a lazy locale is being loaded. */
  readonly isLoading: Readonly<ShallowRef<boolean>>;

  /** Most recent loader failure, cleared on success. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /** Text direction of the active locale. */
  readonly direction: ComputedRef<TextDirection>;

  /** `Intl` helpers bound to the active locale (from `useLocale`). */
  readonly intl: LocaleControls;

  /** Resolves once the initial locale's messages are loaded. */
  readonly ready: Promise<void>;

  /**
   * Translate a key. Parameters are typed from the ICU messages of every
   * locale. Reactive: reading inside a render re-renders on locale change.
   *
   * @returns The formatted message, or the key when it is missing.
   */
  readonly t: <Key extends MessageKey<Catalogs> & string>(
    key: Key,
    ...params: TranslateArguments<MessageParamsFor<Catalogs, Key>>
  ) => string;

  /** Whether `key` exists in the active locale. */
  readonly te: (key: MessageKey<Catalogs> & string) => boolean;

  /**
   * Load (if needed) and activate a locale.
   *
   * @returns Resolves once the locale is active; rejects when loading fails.
   */
  readonly setLocale: (locale: (keyof Catalogs & string) | Lazy) => Promise<void>;

  /**
   * Load a lazy locale without activating it (for example on hover).
   *
   * @returns Resolves once its messages are available.
   */
  readonly loadLocale: (locale: (keyof Catalogs & string) | Lazy) => Promise<void>;

  /** Vue plugin hook: provides this instance to the app. */
  readonly install: (app: App) => void;
}

/** Options for {@link I18nDefinition.create}. */
export interface CreateI18nOptions<Locale extends string> {
  /**
   * Initial locale. Pass the locale resolved for the request (server) and
   * the same value on the client so hydration renders identical text.
   *
   * @default the fallback locale
   */
  readonly locale?: Locale;
}

/** Static i18n definition returned by {@link defineI18n}. */
export interface I18nDefinition<Catalogs extends MessageCatalogs, Lazy extends string> {
  /** Injection key under which instances are provided. */
  readonly key: InjectionKey<I18n<Catalogs, Lazy>>;

  /**
   * Create a per-request (per-app) instance. Never share one instance
   * between server requests.
   *
   * @returns The instance; await `ready` before rendering a lazy locale.
   */
  readonly create: (
    options?: CreateI18nOptions<(keyof Catalogs & string) | Lazy>,
  ) => I18n<Catalogs, Lazy>;

  /**
   * Inject the instance provided by an ancestor (or `app.use(instance)`).
   *
   * @returns The typed instance.
   */
  readonly use: () => I18n<Catalogs, Lazy>;
}

function readMessage(tree: MessageTree | undefined, key: string): string | undefined {
  let node: string | MessageTree | undefined = tree;
  for (const part of key.split(".")) {
    if (node === undefined || typeof node === "string") return undefined;
    node = node[part];
  }
  return typeof node === "string" ? node : undefined;
}

function unwrapModule<Tree>(loaded: Tree | { readonly default: Tree }): Tree {
  return typeof loaded === "object" && loaded !== null && "default" in loaded
    ? loaded.default
    : loaded;
}

/**
 * Define the typed i18n setup of an application.
 *
 * The definition holds only static configuration (catalogs, loaders,
 * formats) and an injection key; it has no mutable module state. Call
 * `create()` per request on the server and once on the client, install the
 * instance with `app.use(instance)`, and read it in components with
 * `definition.use()` (or {@link useI18n}). Locale changes are reactive
 * through `setLocale`, which loads lazy locales first; `Intl` formatting
 * follows the locale through the existing `useLocale` composable.
 *
 * Hydration: locale detection is never automatic. Resolve the locale per
 * request, pass it to `create({ locale })` on both sides, and await
 * `instance.ready` before rendering or mounting when it is lazy.
 * `date`/`time` arguments use a fixed time zone (UTC unless configured).
 *
 * @example
 * ```ts
 * export const appI18n = defineI18n({
 *   messages,
 *   loaders: { fr: () => import("./locales/fr.ts") },
 * });
 * // per request / client entry
 * const i18n = appI18n.create({ locale: "fr" });
 * await i18n.ready;
 * app.use(i18n);
 * // component
 * const { t } = appI18n.use();
 * t("cart", { count: 2 });
 * ```
 *
 * @param options Catalogs, fallback, loaders, formats, and missing-key hook.
 * @returns The i18n definition.
 */
export function defineI18n<
  const Catalogs extends MessageCatalogs,
  const Lazy extends string = never,
>(options: DefineI18nOptions<Catalogs, Lazy>): I18nDefinition<Catalogs, Lazy> {
  type Locale = (keyof Catalogs & string) | Lazy;
  const key: InjectionKey<I18n<Catalogs, Lazy>> = Symbol("vize-i18n");
  const eager = Object.keys(options.messages);
  const lazy: string[] = Object.keys(options.loaders ?? {});
  const fallbackLocale = options.fallbackLocale ?? eager[0] ?? "en";
  const isLocale = (value: string): value is Locale =>
    eager.includes(value) || lazy.includes(value);
  const availableLocales = [...eager, ...lazy].filter(isLocale);

  const create = (createOptions: CreateI18nOptions<Locale> = {}): I18n<Catalogs, Lazy> => {
    const initial = createOptions.locale ?? fallbackLocale;
    if (!isLocale(initial)) {
      throw new RangeError(
        `[VIZE_COMPOSE_I18N_UNKNOWN_LOCALE] unknown locale "${String(initial)}"`,
      );
    }
    const catalogs = shallowReactive(
      new Map<string, MessageTree>(Object.entries(options.messages)),
    );
    const pending = new Map<string, Promise<void>>();
    const formatters = new Map<string, MessageFormatter>();
    const locale = shallowRef<Locale>(initial);
    const isLoading = shallowRef(false);
    const error = shallowRef<unknown>(undefined);
    const intl = useLocale(locale, { detect: () => undefined, fallback: fallbackLocale });

    const formatterFor = (localeTag: string): MessageFormatter => {
      let formatter = formatters.get(localeTag);
      if (formatter === undefined) {
        formatter = createMessageFormatter(localeTag, options);
        formatters.set(localeTag, formatter);
      }
      return formatter;
    };

    const loadLocale = (target: Locale): Promise<void> => {
      if (catalogs.has(target)) return Promise.resolve();
      const existing = pending.get(target);
      if (existing !== undefined) return existing;
      const loaders: Partial<Record<string, LocaleLoader<Catalogs>>> = options.loaders ?? {};
      const loader = loaders[target];
      if (loader === undefined) {
        return Promise.reject(
          new RangeError(`[VIZE_COMPOSE_I18N_UNKNOWN_LOCALE] unknown locale "${target}"`),
        );
      }
      isLoading.value = true;
      const task = loader()
        .then((loaded) => {
          assertKeys(unwrapModule(loaded), "");
          catalogs.set(target, unwrapModule(loaded));
          error.value = undefined;
        })
        .catch((cause: unknown) => {
          error.value = cause;
          throw cause;
        })
        .finally(() => {
          pending.delete(target);
          isLoading.value = pending.size > 0;
        });
      pending.set(target, task);
      return task;
    };

    const setLocale = async (target: Locale): Promise<void> => {
      await loadLocale(target);
      locale.value = target;
    };

    const translate = (messageKey: string, params?: Readonly<Record<string, unknown>>): string => {
      const active = locale.value;
      for (const candidate of [active, fallbackLocale]) {
        const message = readMessage(catalogs.get(candidate), messageKey);
        if (message !== undefined) {
          return formatterFor(candidate === active ? intl.locale.value : candidate).format(
            message,
            params,
          );
        }
      }
      options.onMissing?.(messageKey, active);
      return messageKey;
    };

    const instance: I18n<Catalogs, Lazy> = {
      locale,
      availableLocales,
      loadedLocales: computed(() => [...catalogs.keys()].filter(isLocale)),
      isLoading,
      error,
      direction: intl.direction,
      intl,
      ready: loadLocale(initial),
      t: (messageKey, ...params) => translate(messageKey, params[0]),
      te: (messageKey) => readMessage(catalogs.get(locale.value), messageKey) !== undefined,
      setLocale,
      loadLocale,
      install: (app) => {
        app.provide(key, instance);
      },
    };
    // The initial load's failure is also exposed through `error`.
    instance.ready.catch(() => undefined);
    return instance;
  };

  const use = (): I18n<Catalogs, Lazy> => {
    const instance = inject(key, null);
    if (instance === null) {
      throw new Error(
        "[VIZE_COMPOSE_I18N_NOT_PROVIDED] no i18n instance was provided; call app.use(definition.create())",
      );
    }
    return instance;
  };

  return { key, create, use };
}

/**
 * Inject the i18n instance of a definition. Equivalent to
 * `definition.use()`; must run inside `setup` (or `app.runWithContext`).
 *
 * @example
 * ```ts
 * const { t, setLocale } = useI18n(appI18n);
 * ```
 *
 * @param definition Result of {@link defineI18n}.
 * @throws `Error` tagged `VIZE_COMPOSE_I18N_NOT_PROVIDED` when no instance
 * was provided.
 * @returns The typed instance.
 */
export function useI18n<Catalogs extends MessageCatalogs, Lazy extends string>(
  definition: I18nDefinition<Catalogs, Lazy>,
): I18n<Catalogs, Lazy> {
  return definition.use();
}
