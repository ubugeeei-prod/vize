import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import type { FormControls, FormFieldControls, FormPath, FormPathValue } from "./use-form.ts";

/**
 * Accessible relations for a form control.
 *
 * Structurally identical to `FieldControlProps` from `@vizejs/ui/field-wiring`,
 * so the object can be bound with `v-bind` onto any `@vizejs/ui` control (or a
 * native element) without this package depending on `@vizejs/ui`.
 */
export interface FormFieldControlProps {
  /** Control id, referenced by the label's `for`. */
  readonly id: string;

  /** Id of the field label. */
  readonly "aria-labelledby": string;

  /** Description id and, while invalid, error id. */
  readonly "aria-describedby": string | undefined;

  /** Error id while invalid. */
  readonly "aria-errormessage": string | undefined;

  /** `"true"` while invalid. */
  readonly "aria-invalid": "true" | undefined;
}

/** Props for a `<label>` (or ui `FieldLabel`) element. */
export interface FormFieldLabelProps {
  /** Label id. */
  readonly id: string;

  /** Id of the labelled control. */
  readonly for: string;
}

/** Props for description and error-message elements. */
export interface FormFieldTextProps {
  /** Element id referenced by the control. */
  readonly id: string;
}

/**
 * `v-model` bindings for any component that follows the `modelValue` /
 * `update:modelValue` convention, plus a blur listener that marks the field
 * touched (and validates on blur when the form is configured to).
 */
export interface FormFieldModelProps<Value> {
  /** Current value at the path. */
  readonly modelValue: Value;

  /** Writes the value at the path. */
  readonly "onUpdate:modelValue": (value: Value) => void;

  /** Marks the field touched. */
  readonly onBlur: () => void;
}

/** When a field with errors is reported as invalid. */
export type FormFieldErrorVisibility = "always" | "submitted" | "touched";

/** Options for {@link useFormFieldProps}. */
export interface UseFormFieldPropsOptions {
  /**
   * Explicit control id. Label, description, and error ids derive from it
   * as `<id>-label`, `<id>-description`, and `<id>-error`.
   *
   * @default `${idPrefix}-${path}` with dots replaced by dashes
   */
  readonly id?: MaybeRefOrGetter<string | null | undefined>;

  /**
   * Prefix of the derived id. Use a unique prefix per form on pages with
   * several forms that share field paths.
   *
   * @default "field"
   */
  readonly idPrefix?: string;

  /**
   * Whether a description element is rendered and joins `aria-describedby`.
   *
   * @default false
   */
  readonly hasDescription?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Whether an error element is rendered while invalid and is referenced.
   *
   * @default true
   */
  readonly hasErrorMessage?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * When errors make the field invalid: immediately, after the first
   * submit, or once the field is touched (or the form was submitted).
   *
   * @default "touched"
   */
  readonly showErrors?: FormFieldErrorVisibility;
}

/** Bindings returned by {@link useFormFieldProps}. */
export interface FormFieldPropsControls<Value, Path extends string> {
  /** Underlying typed field controls. */
  readonly field: FormFieldControls<Value, Path>;

  /** Control id. */
  readonly id: ComputedRef<string>;

  /** Whether errors are currently reported. */
  readonly invalid: ComputedRef<boolean>;

  /** First reported error, or `undefined` while valid or hidden. */
  readonly errorMessage: ComputedRef<string | undefined>;

  /** Accessible relations for the control. */
  readonly fieldProps: ComputedRef<FormFieldControlProps>;

  /** Props for the label element. */
  readonly labelProps: ComputedRef<FormFieldLabelProps>;

  /** Props for the description element. */
  readonly descriptionProps: ComputedRef<FormFieldTextProps>;

  /** Props for the error element. */
  readonly errorMessageProps: ComputedRef<FormFieldTextProps>;

  /** `v-model` and blur bindings for the control. */
  readonly modelProps: ComputedRef<FormFieldModelProps<Value>>;

  /** `fieldProps` and `modelProps` merged, for a single `v-bind`. */
  readonly controlProps: ComputedRef<FormFieldControlProps & FormFieldModelProps<Value>>;
}

/**
 * One invalid field, structurally identical to `ErrorSummaryField` from
 * `@vizejs/ui/error-summary`.
 */
export interface FormErrorSummaryItem {
  /** Id of the invalid control (link target). */
  readonly id: string;

  /** First error message. */
  readonly message: string;

  /** Optional field label prefixed to the message. */
  readonly label?: string;
}

/**
 * One error, structurally identical to `FormFieldError` from `@vizejs/ui/form`,
 * so ui `Field` components match errors by `name`.
 */
export interface FormFieldErrorEntry {
  /** Dotted field path, used as the ui field `name`. */
  readonly name: string;

  /** Error message. */
  readonly message: string;

  /** Path segments; numeric segments are numbers. */
  readonly path: readonly (string | number)[];
}

/** Options for {@link useFormErrorSummaryFields}. */
export interface UseFormErrorSummaryFieldsOptions<Values> {
  /**
   * Prefix used to derive control ids; must match {@link useFormFieldProps}.
   *
   * @default "field"
   */
  readonly idPrefix?: string;

  /**
   * Explicit control ids per path, for fields bound with a custom `id`.
   *
   * @default {}
   */
  readonly ids?: { readonly [Path in FormPath<Values>]?: string };

  /**
   * Human-readable labels per path.
   *
   * @default {}
   */
  readonly labels?: { readonly [Path in FormPath<Values>]?: string };

  /**
   * Paths in document order; unlisted paths follow in error order.
   *
   * @default []
   */
  readonly order?: readonly FormPath<Values>[];

  /**
   * Include form-level errors (the `""` path) with this control id.
   *
   * @default undefined
   */
  readonly rootId?: string;

  /**
   * When errors are listed: immediately, or only after the first submit.
   *
   * @default "submitted"
   */
  readonly showErrors?: "always" | "submitted";
}

/** Derive the default control id for a path. */
export function formFieldId(path: string, idPrefix = "field"): string {
  const segment = path.replace(/[^A-Za-z0-9_-]+/g, "-");
  return segment.length === 0 ? idPrefix : `${idPrefix}-${segment}`;
}

function readFlag(
  source: MaybeRefOrGetter<boolean | undefined> | undefined,
  fallback: boolean,
): boolean {
  const value = source === undefined ? undefined : toValue(source);
  return typeof value === "boolean" ? value : fallback;
}

/**
 * Bind one `useForm` field to accessible field markup and any `v-model` control.
 *
 * Returns ids, ARIA relations (`fieldProps`), label/description/error props,
 * and `modelProps` whose shapes match `@vizejs/ui`'s Field contracts
 * structurally, so the bridge lives here without a runtime dependency on the
 * UI package. Ids are derived from the path, so server and client agree
 * without any instance-local id sequence.
 *
 * Server rendering: pure derived state, no host access. Cleanup: none needed
 * beyond the owning form.
 *
 * @example
 * ```vue
 * <script setup lang="ts">
 * const form = useForm({ initialValues: { address: { city: "" } } });
 * const city = useFormFieldProps(form, "address.city");
 * </script>
 * <template>
 *   <label v-bind="city.labelProps.value">City</label>
 *   <TextInput v-bind="city.controlProps.value" />
 *   <p v-if="city.invalid.value" v-bind="city.errorMessageProps.value">{{ city.errorMessage.value }}</p>
 * </template>
 * ```
 *
 * @param form Controls returned by `useForm`.
 * @param path Typed dotted path.
 * @param options Id, relation, and error-visibility options.
 * @default options {}
 * @returns Typed bindings for the field.
 */
export function useFormFieldProps<Values extends object, Output, Path extends FormPath<Values>>(
  form: FormControls<Values, Output>,
  path: Path,
  options: UseFormFieldPropsOptions = {},
): FormFieldPropsControls<FormPathValue<Values, Path>, Path> {
  const field = form.field(path);
  const showErrors = options.showErrors ?? "touched";
  const id = computed(() => {
    const explicit = options.id === undefined ? undefined : toValue(options.id);
    return typeof explicit === "string" && explicit.length > 0
      ? explicit
      : formFieldId(path, options.idPrefix);
  });
  const invalid = computed(() => {
    if (field.errors.value.length === 0) return false;
    if (showErrors === "always") return true;
    const submitted = form.submitCount.value > 0;
    return showErrors === "submitted" ? submitted : submitted || field.touched.value;
  });
  const errorMessage = computed(() => (invalid.value ? field.error.value : undefined));
  const fieldProps = computed<FormFieldControlProps>(() => {
    const describedBy: string[] = [];
    if (readFlag(options.hasDescription, false)) describedBy.push(`${id.value}-description`);
    const referencesError = invalid.value && readFlag(options.hasErrorMessage, true);
    if (referencesError) describedBy.push(`${id.value}-error`);
    return {
      id: id.value,
      "aria-labelledby": `${id.value}-label`,
      "aria-describedby": describedBy.length === 0 ? undefined : describedBy.join(" "),
      "aria-errormessage": referencesError ? `${id.value}-error` : undefined,
      "aria-invalid": invalid.value ? "true" : undefined,
    };
  });
  const modelProps = computed<FormFieldModelProps<FormPathValue<Values, Path>>>(() => ({
    modelValue: field.value.value,
    "onUpdate:modelValue": (value) => {
      field.value.value = value;
    },
    onBlur: field.onBlur,
  }));
  return {
    field,
    id,
    invalid,
    errorMessage,
    fieldProps,
    labelProps: computed(() => ({ id: `${id.value}-label`, for: id.value })),
    descriptionProps: computed(() => ({ id: `${id.value}-description` })),
    errorMessageProps: computed(() => ({ id: `${id.value}-error` })),
    modelProps,
    controlProps: computed(() => ({ ...fieldProps.value, ...modelProps.value })),
  };
}

/**
 * Convert form errors into ui `FormFieldError` entries (`{ name, message, path }`),
 * one per message, so `@vizejs/ui` `Field` components match them by `name`.
 *
 * @param errors `form.errors.value`.
 * @returns Entries in record order; the form-level `""` path has an empty `path`.
 */
export function toFormFieldErrors(
  errors: Readonly<Record<string, readonly string[]>>,
): readonly FormFieldErrorEntry[] {
  return Object.entries(errors).flatMap(([name, messages]) => {
    const path =
      name.length === 0
        ? []
        : name.split(".").map((segment) => (/^\d+$/.test(segment) ? Number(segment) : segment));
    return messages.map((message) => ({ name, message, path }));
  });
}

/**
 * Derive `@vizejs/ui` `ErrorSummary` fields from a `useForm` instance.
 *
 * Each invalid path becomes `{ id, message, label? }` pointing at the control
 * id that {@link useFormFieldProps} assigns, in `order` then error order.
 *
 * Server rendering: pure derived state. Cleanup: none.
 *
 * @example
 * ```vue
 * <ErrorSummary :fields="summary" heading="Fix these fields" />
 * ```
 *
 * @param form Controls returned by `useForm`.
 * @param options Id, label, ordering, and visibility options.
 * @default options {}
 * @returns Summary fields, empty until errors should be shown.
 */
export function useFormErrorSummaryFields<Values extends object, Output>(
  form: FormControls<Values, Output>,
  options: UseFormErrorSummaryFieldsOptions<Values> = {},
): ComputedRef<readonly FormErrorSummaryItem[]> {
  const order: readonly string[] = options.order ?? [];
  const ids: Readonly<Record<string, string | undefined>> = options.ids ?? {};
  const labels: Readonly<Record<string, string | undefined>> = options.labels ?? {};
  return computed(() => {
    if ((options.showErrors ?? "submitted") === "submitted" && form.submitCount.value === 0) {
      return [];
    }
    const record = form.errors.value;
    const paths = Object.keys(record).filter((path) => (record[path]?.length ?? 0) > 0);
    const rank = (path: string): number => {
      const index = order.indexOf(path);
      return index === -1 ? order.length : index;
    };
    const sorted = paths
      .map((path, index) => ({ path, index }))
      .sort((left, right) => rank(left.path) - rank(right.path) || left.index - right.index);
    return sorted.flatMap(({ path }): FormErrorSummaryItem[] => {
      const message = record[path]?.[0];
      if (message === undefined) return [];
      if (path.length === 0 && options.rootId === undefined) return [];
      const id =
        path.length === 0
          ? (options.rootId ?? "")
          : (ids[path] ?? formFieldId(path, options.idPrefix));
      const label = labels[path];
      return [label === undefined ? { id, message } : { id, message, label }];
    });
  });
}
