import { computed, ref, toValue, type MaybeRefOrGetter, type Ref } from "vue";

import { createFormErrorSummaryFields, validateStandardSchema } from "./form-runtime.ts";
import type {
  FormFieldError,
  FormValidationResult,
  StandardSchemaV1,
  StandardSchemaValidationOptions,
} from "./form-types.ts";
import type {
  FormFieldState,
  FormFocusableElement,
  FormStateController,
  FormStateOptions,
  FormStateResetOptions,
  FormSubmitOptions,
  NativeConstraintErrorOptions,
} from "./form-state-types.ts";

const formStateDiagnostic = "VIZE_UI_FORM_STATE";
const nativeConstraintDiagnostic = "VIZE_UI_FORM_CONSTRAINT";

function assertOptions(value: unknown, name: string): void {
  if (typeof value !== "object" || value === null) {
    throw new TypeError(`${formStateDiagnostic}: ${name} must be an object`);
  }
}

function readFieldName(value: unknown, name = "name"): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`${formStateDiagnostic}: ${name} must resolve to a non-empty string`);
  }
  return value;
}

function readErrors(errors: readonly FormFieldError[] | undefined): readonly FormFieldError[] {
  if (errors === undefined) return Object.freeze([]);
  if (!Array.isArray(errors)) {
    throw new TypeError(`${formStateDiagnostic}: errors must be an array`);
  }
  return Object.freeze(
    errors.map((error) => {
      if (typeof error?.name !== "string" || typeof error.message !== "string") {
        throw new TypeError(`${formStateDiagnostic}: every error needs name and message strings`);
      }
      if (!Array.isArray(error.path)) {
        throw new TypeError(`${formStateDiagnostic}: every error needs a path array`);
      }
      return Object.freeze({
        message: error.message,
        name: error.name,
        path: Object.freeze([...error.path]),
      });
    }),
  );
}

function readNameSet(values: readonly string[] | undefined): ReadonlySet<string> {
  if (values === undefined) return Object.freeze(new Set<string>());
  if (!Array.isArray(values)) {
    throw new TypeError(`${formStateDiagnostic}: field lists must be arrays`);
  }
  const names = new Set<string>();
  for (const value of values) names.add(readFieldName(value, "field name"));
  return Object.freeze(names);
}

function updateNameSet(target: Ref<ReadonlySet<string>>, name: string, active: boolean): void {
  const fieldName = readFieldName(name, "field name");
  const next = new Set(target.value);
  if (active) next.add(fieldName);
  else next.delete(fieldName);
  target.value = Object.freeze(next);
}

function readSchema<Input, Output>(
  schema: StandardSchemaV1<Input, Output> | undefined,
): StandardSchemaV1<Input, Output> {
  if (schema === undefined) {
    throw new TypeError(`${formStateDiagnostic}: schema is required to validate`);
  }
  return schema;
}

function controlName(control: Element, options: NativeConstraintErrorOptions): string | undefined {
  const resolved = options.nameForControl?.(control) ?? control.getAttribute("name") ?? undefined;
  if (resolved === null || resolved === undefined || resolved === "") return undefined;
  if (typeof resolved !== "string") {
    throw new TypeError(`${nativeConstraintDiagnostic}: control names must be strings`);
  }
  return resolved;
}

function controlMessage(control: Element, options: NativeConstraintErrorOptions): string {
  const resolved =
    options.messageForControl?.(control) ??
    ("validationMessage" in control && typeof control.validationMessage === "string"
      ? control.validationMessage
      : undefined);
  if (resolved === undefined || resolved.length === 0) return "Invalid value";
  if (typeof resolved !== "string") {
    throw new TypeError(`${nativeConstraintDiagnostic}: control messages must be strings`);
  }
  return resolved;
}

function isInvalidNativeControl(control: Element): boolean {
  if (!("checkValidity" in control) || typeof control.checkValidity !== "function") return false;
  if ("willValidate" in control && control.willValidate === false) return false;
  return !control.checkValidity();
}

/** Convert native constraint-validation failures into normalized form errors. */
export function normalizeNativeConstraintErrors(
  form: HTMLFormElement,
  options: NativeConstraintErrorOptions = {},
): readonly FormFieldError[] {
  if (!(form instanceof HTMLFormElement)) {
    throw new TypeError(`${nativeConstraintDiagnostic}: form must be an HTMLFormElement`);
  }
  assertOptions(options, "options");

  const errors: FormFieldError[] = [];
  for (const control of Array.from(form.elements)) {
    if (!(control instanceof Element) || !isInvalidNativeControl(control)) continue;
    const name = controlName(control, options);
    if (name === undefined) continue;
    errors.push(
      Object.freeze({
        message: controlMessage(control, options),
        name,
        path: Object.freeze([name]),
      }),
    );
  }
  return Object.freeze(errors);
}

/** Create reactive form state around Standard Schema validation and submission. */
export function useFormState<Input, Output = Input>(
  options: FormStateOptions<Input, Output> = {},
): FormStateController<Input, Output> {
  assertOptions(options, "options");

  const errors = ref(readErrors(options.initialErrors)) as Ref<readonly FormFieldError[]>;
  const dirtyFields = ref(readNameSet(undefined)) as Ref<ReadonlySet<string>>;
  const touchedFields = ref(readNameSet(undefined)) as Ref<ReadonlySet<string>>;
  const visitedFields = ref(readNameSet(undefined)) as Ref<ReadonlySet<string>>;
  const activeValidations = ref(0);
  const activeSubmits = ref(0);
  const validationSequence = ref(0);
  const submitSequence = ref(0);
  const submitCount = ref(0);
  const registeredFields = new Map<string, FormFocusableElement>();

  const summaryFields = computed(() =>
    createFormErrorSummaryFields(errors.value, {
      ...(options.idForName === undefined ? {} : { idForName: options.idForName }),
      ...(options.labelForName === undefined ? {} : { labelForName: options.labelForName }),
      ...(options.rootId === undefined ? {} : { rootId: options.rootId }),
      ...(options.rootLabel === undefined ? {} : { rootLabel: options.rootLabel }),
    }),
  );

  const hasErrors = computed(() => errors.value.length > 0);
  const isDirty = computed(() => dirtyFields.value.size > 0);
  const isTouched = computed(() => touchedFields.value.size > 0);
  const isVisited = computed(() => visitedFields.value.size > 0);
  const isValidating = computed(() => activeValidations.value > 0);
  const isSubmitting = computed(() => activeSubmits.value > 0);

  function setErrors(nextErrors: readonly FormFieldError[]): void {
    errors.value = readErrors(nextErrors);
  }

  function reset(resetOptions: FormStateResetOptions = {}): void {
    assertOptions(resetOptions, "reset options");
    validationSequence.value += 1;
    submitSequence.value += 1;
    errors.value = readErrors(resetOptions.errors);
    dirtyFields.value = readNameSet(resetOptions.dirtyFields);
    touchedFields.value = readNameSet(resetOptions.touchedFields);
    visitedFields.value = readNameSet(resetOptions.visitedFields);
  }

  async function validate(
    value: Input,
    validationOptions: StandardSchemaValidationOptions = {},
  ): Promise<FormValidationResult<Output>> {
    const sequence = validationSequence.value + 1;
    validationSequence.value = sequence;
    activeValidations.value += 1;
    try {
      const result = await validateStandardSchema(
        readSchema(options.schema),
        value,
        validationOptions,
      );
      if (sequence === validationSequence.value) setErrors(result.errors);
      return result;
    } finally {
      activeValidations.value = Math.max(0, activeValidations.value - 1);
    }
  }

  async function submit(
    value: Input,
    submitOptions: FormSubmitOptions<Output> = {},
  ): Promise<FormValidationResult<Output>> {
    assertOptions(submitOptions, "submit options");
    const sequence = submitSequence.value + 1;
    submitSequence.value = sequence;
    submitCount.value += 1;
    activeSubmits.value += 1;
    try {
      const result = await validate(value, submitOptions);
      if (sequence !== submitSequence.value) return result;
      if (!result.valid) {
        if (submitOptions.focusFirstInvalid ?? options.focusFirstInvalidOnSubmit ?? true) {
          focusFirstInvalid();
        }
        return result;
      }
      const onSubmit = submitOptions.onSubmit ?? options.onSubmit;
      if (onSubmit !== undefined) await onSubmit(result.value, result);
      return result;
    } finally {
      activeSubmits.value = Math.max(0, activeSubmits.value - 1);
    }
  }

  function fieldState(
    nameInput: MaybeRefOrGetter<string>,
  ): ReturnType<FormStateController["fieldState"]> {
    return computed<FormFieldState>(() => {
      const name = readFieldName(toValue(nameInput));
      const fieldErrors = Object.freeze(errors.value.filter((error) => error.name === name));
      const firstError = fieldErrors[0];
      return Object.freeze({
        errorMessage: firstError?.message,
        errors: fieldErrors,
        firstError,
        isDirty: dirtyFields.value.has(name),
        isInvalid: fieldErrors.length > 0,
        isTouched: touchedFields.value.has(name),
        isVisited: visitedFields.value.has(name),
        name,
      });
    });
  }

  function registerField(
    name: string,
    element: FormFocusableElement | null | undefined,
  ): () => void {
    const fieldName = readFieldName(name, "field name");
    if (element === null || element === undefined) {
      registeredFields.delete(fieldName);
      return () => {};
    }
    if (typeof element.focus !== "function") {
      throw new TypeError(`${formStateDiagnostic}: registered fields must be focusable`);
    }
    registeredFields.set(fieldName, element);
    return () => {
      if (registeredFields.get(fieldName) === element) registeredFields.delete(fieldName);
    };
  }

  function focusFirstInvalid(options?: FocusOptions): boolean {
    for (const error of errors.value) {
      const target = registeredFields.get(error.name);
      if (target === undefined) continue;
      target.focus(options);
      return true;
    }
    return false;
  }

  function markFieldDirty(name: string, dirty = true): void {
    updateNameSet(dirtyFields, name, dirty);
  }

  function markFieldTouched(name: string, touched = true): void {
    updateNameSet(touchedFields, name, touched);
  }

  function markFieldVisited(name: string, visited = true): void {
    updateNameSet(visitedFields, name, visited);
  }

  return Object.freeze({
    clearErrors: () => setErrors([]),
    errors,
    fieldState,
    focusFirstInvalid,
    hasErrors,
    isDirty,
    isSubmitting,
    isTouched,
    isValidating,
    isVisited,
    markFieldDirty,
    markFieldTouched,
    markFieldVisited,
    registerField,
    reset,
    setErrors,
    setServerErrors: setErrors,
    submit,
    submitCount,
    summaryFields,
    validate,
  });
}
