import { computed, toValue, type MaybeRefOrGetter } from "vue";

import type { ErrorSummaryField } from "./error-summary-types.ts";
import type {
  FormErrorSummaryFieldOptions,
  FormErrorSummaryController,
  FormErrorSummaryOptions,
  FormFieldController,
  FormFieldError,
  FormFieldErrorOptions,
  FormFieldOptions,
  FormPathKey,
  FormValidationFailure,
  FormValidationResult,
  FormValidationSuccess,
  StandardSchemaV1,
  StandardSchemaValidationOptions,
} from "./form-types.ts";

const formOptionDiagnostic = "VIZE_UI_FORM_OPTION";
const formResultDiagnostic = "VIZE_UI_FORM_SCHEMA_RESULT";
const formSchemaDiagnostic = "VIZE_UI_FORM_SCHEMA";
const identifierSegmentPattern = /^[A-Za-z_$][\w$]*$/;

function isRecord(value: unknown): value is Record<PropertyKey, unknown> {
  return typeof value === "object" && value !== null;
}

function isPropertyKey(value: unknown): value is PropertyKey {
  return typeof value === "string" || typeof value === "number" || typeof value === "symbol";
}

function assertCallback(value: unknown, name: string): void {
  if (value !== undefined && typeof value !== "function") {
    throw new TypeError(`${formOptionDiagnostic}: ${name} must be a function`);
  }
}

function assertOptions(value: unknown, name: string): void {
  if (!isRecord(value)) {
    throw new TypeError(`${formOptionDiagnostic}: ${name} must be an object`);
  }
}

function readName(value: unknown): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`${formOptionDiagnostic}: name must resolve to a non-empty string`);
  }
  return value;
}

function readRootId(value: unknown): string {
  if (value === undefined) return "form";
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`${formOptionDiagnostic}: rootId must resolve to a non-empty string`);
  }
  return value;
}

function readFieldErrors(
  value: MaybeRefOrGetter<readonly FormFieldError[] | undefined> | undefined,
): readonly FormFieldError[] {
  const resolved = toValue(value);
  if (resolved === undefined) return [];
  if (!Array.isArray(resolved)) {
    throw new TypeError(`${formOptionDiagnostic}: errors must resolve to an array`);
  }
  for (const error of resolved) {
    if (!isRecord(error)) {
      throw new TypeError(`${formOptionDiagnostic}: every error must be an object`);
    }
    if (typeof error.name !== "string") {
      throw new TypeError(`${formOptionDiagnostic}: every error needs a name string`);
    }
    if (typeof error.message !== "string") {
      throw new TypeError(`${formOptionDiagnostic}: every error needs a message string`);
    }
    if (!Array.isArray(error.path)) {
      throw new TypeError(`${formOptionDiagnostic}: every error needs a path array`);
    }
  }
  return resolved;
}

function readIssuePath(issue: StandardSchemaV1.Issue): readonly FormPathKey[] {
  const path = issue.path;
  if (path === undefined) return Object.freeze([]);
  if (!Array.isArray(path)) {
    throw new TypeError(`${formResultDiagnostic}: issue path must be an array`);
  }
  return Object.freeze(
    path.map((segment): FormPathKey => {
      if (isPropertyKey(segment)) return segment;
      if (isRecord(segment) && isPropertyKey(segment.key)) return segment.key;
      throw new TypeError(
        `${formResultDiagnostic}: issue path segments must contain property keys`,
      );
    }),
  );
}

function formatSegment(key: FormPathKey, index: number): string {
  const segment = String(key);
  if (typeof key === "number") return `[${segment}]`;
  if (identifierSegmentPattern.test(segment)) {
    return index === 0 ? segment : `.${segment}`;
  }
  return `[${JSON.stringify(segment)}]`;
}

/** Convert a Standard Schema issue path into a conventional HTML field name. */
export function formatFormFieldName(path: readonly FormPathKey[]): string {
  if (!Array.isArray(path)) {
    throw new TypeError(`${formOptionDiagnostic}: path must be an array`);
  }
  return path.map(formatSegment).join("");
}

/** Convert Standard Schema issues into normalized field errors. */
export function normalizeStandardSchemaIssues(
  issues: readonly StandardSchemaV1.Issue[],
  options: FormFieldErrorOptions = {},
): readonly FormFieldError[] {
  assertOptions(options, "options");
  assertCallback(options.nameForPath, "nameForPath");
  return Object.freeze(
    issues.map((issue): FormFieldError => {
      if (!isRecord(issue)) {
        throw new TypeError(`${formResultDiagnostic}: every issue must be an object`);
      }
      if (typeof issue.message !== "string") {
        throw new TypeError(`${formResultDiagnostic}: every issue needs a message string`);
      }
      const path = readIssuePath(issue);
      const name = options.nameForPath?.(path, issue) ?? formatFormFieldName(path);
      if (typeof name !== "string") {
        throw new TypeError(`${formOptionDiagnostic}: nameForPath must return a string`);
      }
      return Object.freeze({ message: issue.message, name, path });
    }),
  );
}

/** Convert normalized field errors into unique fields for the error summary. */
export function createFormErrorSummaryFields(
  errors: readonly FormFieldError[],
  options: FormErrorSummaryFieldOptions = {},
): readonly ErrorSummaryField[] {
  assertOptions(options, "options");
  assertCallback(options.idForName, "idForName");
  assertCallback(options.labelForName, "labelForName");
  const rootId = readRootId(options.rootId);
  const fields: ErrorSummaryField[] = [];
  const seen = new Set<string>();

  for (const error of readFieldErrors(errors)) {
    const fallbackId = error.name.length === 0 ? rootId : error.name;
    const id = options.idForName?.(error.name, error) ?? fallbackId;
    if (id === null) continue;
    if (id === undefined) continue;
    if (typeof id !== "string" || id.length === 0) {
      throw new TypeError(`${formOptionDiagnostic}: idForName must return a non-empty string`);
    }
    if (seen.has(id)) continue;
    seen.add(id);
    const label =
      error.name.length === 0 ? options.rootLabel : options.labelForName?.(error.name, error);
    fields.push(
      Object.freeze(
        label === undefined
          ? { id, message: error.message }
          : { id, label, message: error.message },
      ),
    );
  }

  return Object.freeze(fields);
}

/** Normalize a Standard Schema validation result for form fields and summaries. */
export function normalizeStandardSchemaResult<Output>(
  result: StandardSchemaV1.Result<Output>,
  options: StandardSchemaValidationOptions = {},
): FormValidationResult<Output> {
  assertOptions(options, "options");
  if (!isRecord(result)) {
    throw new TypeError(`${formResultDiagnostic}: result must be an object`);
  }

  if (Array.isArray(result.issues)) {
    const errors = normalizeStandardSchemaIssues(result.issues, options);
    return Object.freeze<FormValidationFailure>({
      errors,
      summaryFields: createFormErrorSummaryFields(errors, options),
      valid: false,
    });
  }

  if (result.issues === undefined && "value" in result) {
    return Object.freeze<FormValidationSuccess<Output>>({
      errors: Object.freeze([]),
      summaryFields: Object.freeze([]),
      valid: true,
      value: result.value as Output,
    });
  }

  throw new TypeError(`${formResultDiagnostic}: result must contain value or issues`);
}

/** Validate a Standard Schema and normalize the result for form consumers. */
export async function validateStandardSchema<Input, Output>(
  schema: StandardSchemaV1<Input, Output>,
  value: Input,
  options: StandardSchemaValidationOptions = {},
): Promise<FormValidationResult<Output>> {
  assertOptions(options, "options");
  const standard = schema?.["~standard"];
  if (!isRecord(standard) || standard.version !== 1 || typeof standard.validate !== "function") {
    throw new TypeError(`${formSchemaDiagnostic}: schema must implement Standard Schema V1`);
  }
  const validationOptions =
    options.libraryOptions === undefined ? undefined : { libraryOptions: options.libraryOptions };
  const result = await standard.validate(value, validationOptions);
  return normalizeStandardSchemaResult(result, options);
}

/** Create normalized field state whose invalid flag can feed existing field wiring. */
export function useFormField(options: FormFieldOptions): FormFieldController {
  assertOptions(options, "options");
  const name = computed(() => readName(toValue(options.name)));
  const errors = computed(() =>
    readFieldErrors(options.errors).filter((error) => error.name === name.value),
  );
  const firstError = computed(() => errors.value[0]);
  const errorMessage = computed(() => firstError.value?.message);
  const isInvalid = computed(() => errors.value.length > 0);

  return Object.freeze({
    errorMessage,
    errors,
    firstError,
    isInvalid,
    name,
  });
}

/** Create error-summary fields from normalized form errors. */
export function useFormErrorSummary(
  options: FormErrorSummaryOptions = {},
): FormErrorSummaryController {
  assertOptions(options, "options");
  const fields = computed(() =>
    createFormErrorSummaryFields(readFieldErrors(options.errors), {
      ...(options.idForName === undefined ? {} : { idForName: options.idForName }),
      ...(options.labelForName === undefined ? {} : { labelForName: options.labelForName }),
      rootId: readRootId(toValue(options.rootId)),
      ...(options.rootLabel === undefined ? {} : { rootLabel: options.rootLabel }),
    }),
  );
  const hasErrors = computed(() => fields.value.length > 0);

  return Object.freeze({
    fields,
    hasErrors,
  });
}
