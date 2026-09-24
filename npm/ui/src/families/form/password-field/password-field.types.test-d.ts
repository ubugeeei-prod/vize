/** Compile-only assertions for the public PasswordField contract. */

import {
  PasswordField,
  PasswordFieldInput,
  PasswordFieldToggle,
  estimatePasswordStrength,
  type PasswordFieldExpose,
  type PasswordFieldSlotState,
  type PasswordFieldState,
  type PasswordStrengthEstimate,
  type PasswordStrengthLabel,
  type PasswordStrengthScore,
} from "./password-field.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type RootProps<Strength> = Parameters<typeof PasswordField<Strength>>[0];

/** Infers the strength type exactly as a template usage would. */
declare function inferStrength<Strength>(props: RootProps<Strength>): Strength;

declare const control: PasswordFieldExpose<PasswordStrengthEstimate>;
declare const slot: PasswordFieldSlotState<"weak" | "ok">;

const inferredEstimate = inferStrength({ evaluateStrength: estimatePasswordStrength });
const inferredCustom = inferStrength({
  evaluateStrength: (value: string) => (value.length > 12 ? ("ok" as const) : ("weak" as const)),
});

type _InfersEstimate = Expect<Equal<typeof inferredEstimate, PasswordStrengthEstimate>>;
type _InfersCustom = Expect<Equal<typeof inferredCustom, "ok" | "weak">>;
type _SlotStrengthIsOptional = Expect<Equal<typeof slot.strength, "ok" | "weak" | undefined>>;
type _ExposeStrength = Expect<Equal<typeof control.strength, PasswordStrengthEstimate | undefined>>;
type _StateIsClosed = Expect<
  Equal<PasswordFieldState, "disabled" | "hidden" | "readonly" | "visible">
>;
type _ScoreIsClosed = Expect<Equal<PasswordStrengthScore, 0 | 1 | 2 | 3 | 4>>;
type _LabelIsClosed = Expect<
  Equal<PasswordStrengthLabel, "very-weak" | "weak" | "fair" | "strong" | "very-strong">
>;

const props: RootProps<PasswordStrengthEstimate> = {
  autocomplete: "new-password",
  defaultVisible: true,
  evaluateStrength: estimatePasswordStrength,
  "onUpdate:visible": (visible: boolean) => visible,
  onCapsLockChange: (active: boolean) => active,
};
const inputProps: InstanceType<typeof PasswordFieldInput>["$props"] = { minlength: 8 };
const toggleProps: InstanceType<typeof PasswordFieldToggle>["$props"] = { showLabel: "Show" };

control.toggleVisible();
control.setVisible(false);

// @ts-expect-error the evaluator receives the password string.
const wrongEvaluator: RootProps<number> = { evaluateStrength: (value: number) => value };

// @ts-expect-error visibility is boolean.
control.setVisible("yes");

// @ts-expect-error minlength is numeric.
const wrongInput: InstanceType<typeof PasswordFieldInput>["$props"] = { minlength: "8" };

void inputProps;
void props;
void toggleProps;
void wrongEvaluator;
void wrongInput;
