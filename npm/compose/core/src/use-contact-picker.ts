import {
  computed,
  hasInjectionContext,
  readonly,
  ref,
  shallowRef,
  toValue,
  watchPostEffect,
} from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

/** Contact property that can be requested from the picker. */
export type ContactProperty = "address" | "email" | "icon" | "name" | "tel";

/** Postal address returned for the `address` property. */
export interface ContactAddressLike {
  /** Street address lines. */
  readonly addressLine?: readonly string[];
  /** City. */
  readonly city?: string;
  /** Region or state. */
  readonly region?: string;
  /** Postal code. */
  readonly postalCode?: string;
  /** ISO 3166 country code. */
  readonly country?: string;
}

/** Every value the picker can return, keyed by property. */
export interface ContactInfo {
  /** Postal addresses. */
  readonly address: readonly ContactAddressLike[];
  /** Email addresses. */
  readonly email: readonly string[];
  /** Avatar images. */
  readonly icon: readonly Blob[];
  /** Names. */
  readonly name: readonly string[];
  /** Phone numbers. */
  readonly tel: readonly string[];
}

/** Contact typed with exactly the requested properties. */
export type SelectedContact<Property extends ContactProperty> = Pick<ContactInfo, Property>;

/** Minimal `navigator.contacts` (`ContactsManager`). */
export interface ContactsManagerLike {
  /** Properties this platform supports. */
  getProperties(): Promise<readonly string[]>;
  /** Open the picker (needs user activation). */
  select(
    properties: ContactProperty[],
    options?: { multiple?: boolean },
  ): Promise<readonly Partial<ContactInfo>[]>;
}

/** Options for {@link useContactPicker}. */
export interface UseContactPickerOptions {
  /**
   * Contacts manager for alternate runtimes and tests.
   *
   * @default window.navigator.contacts when it exists
   */
  readonly contacts?: MaybeRefOrGetter<ContactsManagerLike | null | undefined>;
}

/** Options of {@link ContactPickerControls.select}. */
export interface ContactSelectOptions {
  /**
   * Allow picking several contacts.
   *
   * @default false
   */
  readonly multiple?: boolean;
}

/** Discriminated outcome of {@link ContactPickerControls.select}. */
export type ContactPickerResult<Property extends ContactProperty> =
  | {
      /** The picker closed; `contacts` is empty when the user dismissed it. */
      readonly status: "selected";
      /** Picked contacts with exactly the requested keys. */
      readonly contacts: SelectedContact<Property>[];
    }
  | {
      /** `unsupported`: no API; `failed`: the host rejected. */
      readonly status: "unsupported" | "failed";
      /** Error thrown by the host, when one was thrown. */
      readonly error: unknown;
    };

/** Reactive state and actions returned by {@link useContactPicker}. */
export interface ContactPickerControls {
  /** Whether the Contact Picker API is available. */
  readonly supported: ComputedRef<boolean>;
  /** Whether the picker is open. */
  readonly pending: Readonly<Ref<boolean>>;
  /** Most recent failure, cleared on success. */
  readonly error: Readonly<ShallowRef<unknown>>;
  /**
   * Properties the platform supports.
   *
   * @returns Known supported properties; empty when unsupported or failing.
   */
  readonly getProperties: () => Promise<ContactProperty[]>;
  /**
   * Open the contact picker.
   *
   * @param properties Properties to request; the result is typed from them.
   * @param options Selection options.
   * @throws `TypeError` tagged `VIZE_COMPOSE_CONTACT_PICKER_NO_PROPERTIES` when `properties` is empty.
   * @returns The discriminated outcome; never rejects.
   */
  readonly select: <const Property extends ContactProperty>(
    properties: readonly Property[],
    options?: ContactSelectOptions,
  ) => Promise<ContactPickerResult<Property>>;
}

const knownProperties: readonly string[] = ["address", "email", "icon", "name", "tel"];

function isContactProperty(value: string): value is ContactProperty {
  return knownProperties.includes(value);
}

function isContactsManager(candidate: unknown): candidate is ContactsManagerLike {
  return (
    typeof candidate === "object" &&
    candidate !== null &&
    "select" in candidate &&
    "getProperties" in candidate &&
    typeof candidate.select === "function" &&
    typeof candidate.getProperties === "function"
  );
}

function browserContacts(): ContactsManagerLike | undefined {
  if (typeof window === "undefined") return undefined;
  const candidate: unknown = Reflect.get(window.navigator, "contacts");
  return isContactsManager(candidate) ? candidate : undefined;
}

function pickContact<Property extends ContactProperty>(
  contact: Partial<ContactInfo>,
  properties: readonly Property[],
): SelectedContact<Property> {
  const full: ContactInfo = {
    address: contact.address ?? [],
    email: contact.email ?? [],
    icon: contact.icon ?? [],
    name: contact.name ?? [],
    tel: contact.tel ?? [],
  };
  const picked: Partial<Pick<ContactInfo, Property>> = {};
  for (const property of properties) picked[property] = full[property];
  // Every requested key was assigned in the loop above.
  return picked as SelectedContact<Property>;
}

/**
 * Let the user share contacts with the Contact Picker API.
 *
 * `select(["name", "email"])` resolves to contacts typed with exactly those
 * keys (missing values become empty arrays). Dismissing the picker resolves
 * with an empty list. The composable holds no resources, so nothing needs
 * cleanup.
 *
 * Server rendering: nothing is requested, `supported` and `pending` are false.
 * Inside a component `supported` turns true only after mounting, so
 * hydration renders this server state first.
 *
 * @example
 * ```ts
 * const picker = useContactPicker();
 * const result = await picker.select(["name", "tel"], { multiple: true });
 * ```
 *
 * @param options Capability host.
 * @default options {}
 * @returns Picker state and actions.
 */
export function useContactPicker(options: UseContactPickerOptions = {}): ContactPickerControls {
  const pending = ref(false);
  const error = shallowRef<unknown>(undefined);

  // Inside a component the host is resolved only after mounting, so a
  // hydrating client renders the server's unsupported state first. Outside
  // components it resolves synchronously.
  const hydrated = shallowRef(!hasInjectionContext());
  if (!hydrated.value) {
    watchPostEffect(() => {
      hydrated.value = true;
    });
  }

  const resolveHost = (): ContactsManagerLike | undefined =>
    options.contacts === undefined ? browserContacts() : (toValue(options.contacts) ?? undefined);

  const getProperties = async (): Promise<ContactProperty[]> => {
    try {
      return [...((await resolveHost()?.getProperties()) ?? [])].filter(isContactProperty);
    } catch {
      return [];
    }
  };

  const select = <const Property extends ContactProperty>(
    properties: readonly Property[],
    selectOptions: ContactSelectOptions = {},
  ): Promise<ContactPickerResult<Property>> => {
    if (properties.length === 0) {
      throw new TypeError(
        "[VIZE_COMPOSE_CONTACT_PICKER_NO_PROPERTIES] select() needs at least one property",
      );
    }
    const host = resolveHost();
    if (!host) return Promise.resolve({ status: "unsupported", error: undefined });
    pending.value = true;
    return host
      .select([...properties], { multiple: selectOptions.multiple ?? false })
      .then(
        (contacts): ContactPickerResult<Property> => {
          error.value = undefined;
          return {
            status: "selected",
            contacts: contacts.map((contact) => pickContact(contact, properties)),
          };
        },
        (cause: unknown): ContactPickerResult<Property> => {
          error.value = cause;
          return { status: "failed", error: cause };
        },
      )
      .finally(() => {
        pending.value = false;
      });
  };

  return {
    supported: computed(() => hydrated.value && resolveHost() !== undefined),
    pending: readonly(pending),
    error: readonly(error),
    getProperties,
    select,
  };
}
