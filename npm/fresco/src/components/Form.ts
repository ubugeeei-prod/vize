/**
 * Form Component - Form container with labels
 */

import { computed, defineComponent, h, ref, type PropType, type VNode, watch } from "vue";
import { useInput } from "../composables/useInput.js";
import {
  adjacentEnabledIndex,
  firstEnabledIndex,
  lastEnabledIndex,
} from "./collection-navigation.js";

/** Values submitted and validated by Form, keyed by field key. */
export type FormValues = Readonly<Record<string, unknown>>;

/** Validation messages keyed by field key. */
export type FormErrors = Readonly<Record<string, string>>;

/** Metadata and validation contract for one named Form slot. */
export interface FormField {
  key: string;
  label: string;
  required?: boolean;
  disabled?: boolean;
  hint?: string;
  validate?: (value: unknown, values: FormValues) => boolean | string | null | undefined;
}

/** Props passed to each named field slot. */
export interface FormSlotProps {
  readonly field: FormField;
  readonly focused: boolean;
  readonly error?: string;
}

/** Public props for the focus- and validation-aware Form container. */
export interface FormProps {
  /** Form fields metadata */
  fields?: FormField[];
  /** Label width */
  labelWidth?: number;
  /** Gap between fields */
  gap?: number;
  /** Label position */
  labelPosition?: "left" | "top";
  /** Label foreground color */
  labelFg?: string;
  /** Required indicator */
  requiredIndicator?: string;
  /** Hint foreground color */
  hintFg?: string;
  /** Validation error foreground color */
  errorFg?: string;
  /** Current field values keyed by field key */
  modelValue?: FormValues;
  /** Field key that owns focus */
  focusedKey?: string;
  /** Whether the form owns keyboard focus navigation and submission */
  focused?: boolean;
}

export const Form = defineComponent({
  name: "Form",
  props: {
    fields: {
      type: Array as PropType<FormField[]>,
      default: () => [],
    },
    labelWidth: {
      type: Number,
      default: 15,
    },
    gap: {
      type: Number,
      default: 1,
    },
    labelPosition: {
      type: String as PropType<"left" | "top">,
      default: "left",
    },
    labelFg: String,
    requiredIndicator: {
      type: String,
      default: "*",
    },
    hintFg: {
      type: String,
      default: "gray",
    },
    errorFg: {
      type: String,
      default: "red",
    },
    modelValue: {
      type: Object as PropType<FormValues>,
      default: () => ({}),
    },
    focusedKey: String,
    focused: {
      type: Boolean,
      default: false,
    },
  },
  emits: ["update:focusedKey", "submit", "invalid"],
  setup(props, { emit, slots }) {
    const enabledFields = computed(() => props.fields.filter((field) => !field.disabled));
    const highlightedIndex = ref(-1);

    const validationErrors = computed<Record<string, string>>(() => {
      const errors: Record<string, string> = {};
      for (const field of props.fields) {
        if (field.disabled) continue;
        const value = props.modelValue[field.key];
        const empty =
          value === undefined ||
          value === null ||
          value === "" ||
          (Array.isArray(value) && value.length === 0);
        if (field.required && empty) {
          errors[field.key] = `${field.label} is required.`;
          continue;
        }
        if (empty || !field.validate) continue;
        const result = field.validate(value, props.modelValue);
        if (result === false) errors[field.key] = `${field.label} is invalid.`;
        else if (typeof result === "string") errors[field.key] = result;
      }
      return errors;
    });

    const syncHighlight = () => {
      const requestedIndex = enabledFields.value.findIndex(
        (field) => field.key === props.focusedKey,
      );
      highlightedIndex.value =
        requestedIndex >= 0
          ? requestedIndex
          : highlightedIndex.value >= 0 && highlightedIndex.value < enabledFields.value.length
            ? highlightedIndex.value
            : firstEnabledIndex(enabledFields.value);
    };

    watch([() => props.focusedKey, enabledFields], syncHighlight, {
      deep: true,
      immediate: true,
    });

    const setHighlight = (index: number) => {
      const field = enabledFields.value[index];
      if (!field || index === highlightedIndex.value) return;
      highlightedIndex.value = index;
      emit("update:focusedKey", field.key);
    };

    const submit = () => {
      const errors = validationErrors.value;
      const firstInvalidKey = Object.keys(errors).at(0);
      if (firstInvalidKey) {
        emit("invalid", errors);
        setHighlight(enabledFields.value.findIndex((field) => field.key === firstInvalidKey));
        return;
      }
      emit("submit", props.modelValue);
    };

    useInput({
      isActive: computed(() => props.focused),
      onArrow: (direction) => {
        if (direction !== "up" && direction !== "down") return;
        setHighlight(
          adjacentEnabledIndex(
            enabledFields.value,
            highlightedIndex.value,
            direction === "up" ? -1 : 1,
          ),
        );
      },
      onKey: (key, modifiers) => {
        if (key === "tab" || key === "backtab") {
          setHighlight(
            adjacentEnabledIndex(
              enabledFields.value,
              highlightedIndex.value,
              key === "backtab" || modifiers.shift ? -1 : 1,
            ),
          );
        }
        if (key === "home") setHighlight(firstEnabledIndex(enabledFields.value));
        if (key === "end") setHighlight(lastEnabledIndex(enabledFields.value));
      },
      onSubmit: submit,
    });

    return () => {
      const children: VNode[] = [];

      props.fields.forEach((field, _index) => {
        const enabledIndex = enabledFields.value.findIndex(
          (candidate) => candidate.key === field.key,
        );
        const isFocused = enabledIndex >= 0 && enabledIndex === highlightedIndex.value;
        const error = validationErrors.value[field.key];
        const labelContent = [
          h(
            "text",
            {
              fg: props.labelFg,
              bold: props.focused && isFocused,
              dim: field.disabled,
            },
            field.label.padEnd(props.labelWidth),
          ),
        ];

        if (field.required) {
          labelContent.push(h("text", { fg: "red" }, props.requiredIndicator));
        }

        const slotProps: FormSlotProps = { field, focused: isFocused, error };
        const fieldSlot = slots[field.key]?.(slotProps);

        if (props.labelPosition === "top") {
          children.push(
            h(
              "box",
              {
                key: field.key,
                style: { flex_direction: "column", margin_bottom: props.gap },
              },
              [
                h("box", { style: { flex_direction: "row" } }, labelContent),
                fieldSlot ? h("box", { style: { margin_top: 0.5 } }, fieldSlot) : null,
                field.hint ? h("text", { fg: props.hintFg, dim: true }, field.hint) : null,
                error ? h("text", { fg: props.errorFg }, error) : null,
              ].filter(Boolean),
            ),
          );
        } else {
          children.push(
            h(
              "box",
              {
                key: field.key,
                style: {
                  flex_direction: "row",
                  align_items: "center",
                  margin_bottom: props.gap,
                },
              },
              [
                h(
                  "box",
                  { style: { width: String(props.labelWidth), flex_direction: "row" } },
                  labelContent,
                ),
                h("box", { style: { flex_grow: 1 } }, fieldSlot),
              ],
            ),
          );

          if (field.hint) {
            children.push(
              h(
                "text",
                {
                  key: `hint-${field.key}`,
                  fg: props.hintFg,
                  dim: true,
                  style: { margin_left: props.labelWidth, margin_bottom: props.gap },
                },
                field.hint,
              ),
            );
          }
          if (error) {
            children.push(
              h(
                "text",
                {
                  key: `error-${field.key}`,
                  fg: props.errorFg,
                  style: { margin_left: props.labelWidth, margin_bottom: props.gap },
                },
                error,
              ),
            );
          }
        }
      });

      return h("box", { style: { flex_direction: "column" } }, children);
    };
  },
});
