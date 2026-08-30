import { computed, getCurrentInstance, toValue } from "vue";

import { deriveDeterministicId, useDeterministicId } from "../../../deterministic-id.ts";
import type { FieldWiringController, FieldWiringOptions } from "./field-wiring-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_FIELD_WIRING_OPTION";
const setupDiagnostic = "VIZE_UI_FIELD_WIRING_SETUP";

type BooleanOption = FieldWiringOptions["invalid"];

function readBoolean(value: BooleanOption, name: string, fallback: boolean): boolean {
  const resolved = toValue(value);
  if (resolved === undefined) return fallback;
  if (typeof resolved !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return resolved;
}

/**
 * Wire one form control to its accessible name, description, and error message.
 *
 * All four ids derive from one deterministic control id, so server and client
 * renders agree. `aria-errormessage` only applies while the control is invalid,
 * and the error id also joins `aria-describedby` for assistive technology
 * without `aria-errormessage` support.
 */
export function useFieldWiring(options: FieldWiringOptions = {}): FieldWiringController {
  if (getCurrentInstance() === null) {
    throw new Error(`${setupDiagnostic}: useFieldWiring() must run during component setup`);
  }
  for (const name of ["invalid", "hasDescription", "hasErrorMessage"] as const) {
    if (typeof options[name] !== "function") readBoolean(options[name], name, false);
  }
  const fieldId = useDeterministicId({ id: options.id, hint: "field" });
  const labelId = computed(() => deriveDeterministicId(fieldId.value, "label"));
  const descriptionId = computed(() => deriveDeterministicId(fieldId.value, "description"));
  const errorMessageId = computed(() => deriveDeterministicId(fieldId.value, "error"));
  const isInvalid = computed(() => readBoolean(options.invalid, "invalid", false));
  const showsError = computed(
    () => isInvalid.value && readBoolean(options.hasErrorMessage, "hasErrorMessage", true),
  );
  const describedBy = computed(() => {
    const ids: string[] = [];
    if (readBoolean(options.hasDescription, "hasDescription", false)) {
      ids.push(descriptionId.value);
    }
    if (showsError.value) ids.push(errorMessageId.value);
    return ids.length === 0 ? undefined : ids.join(" ");
  });

  return Object.freeze({
    fieldId,
    labelId,
    descriptionId,
    errorMessageId,
    isInvalid,
    labelProps: computed(() => ({ id: labelId.value, for: fieldId.value })),
    fieldProps: computed(() => ({
      id: fieldId.value,
      "aria-labelledby": labelId.value,
      "aria-describedby": describedBy.value,
      "aria-errormessage": showsError.value ? errorMessageId.value : undefined,
      "aria-invalid": isInvalid.value ? ("true" as const) : undefined,
    })),
    descriptionProps: computed(() => ({ id: descriptionId.value })),
    errorMessageProps: computed(() => ({ id: errorMessageId.value })),
  });
}
