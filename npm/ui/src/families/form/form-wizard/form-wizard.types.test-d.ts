/** Compile-only assertions for the public FormWizard contract. */

import {
  FormWizard,
  FormWizardBack,
  FormWizardNext,
  FormWizardProgress,
  FormWizardStep,
  type FormWizardDirection,
  type FormWizardDraftStore,
  type FormWizardExpose,
  type FormWizardSlotState,
  type FormWizardState,
} from "./form-wizard.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type WizardProps<StepId extends string> = Parameters<typeof FormWizard<StepId>>[0];

/** Infers step ids exactly as a template usage would. */
declare function inferSteps<StepId extends string>(props: WizardProps<StepId>): StepId;

declare const wizard: FormWizardExpose<"a" | "b">;
declare const slot: FormWizardSlotState<"a" | "b">;

const inferred = inferSteps({ steps: ["account", "profile", "review"] });

type _InfersStepUnion = Expect<Equal<typeof inferred, "account" | "profile" | "review">>;
type _CurrentIsTyped = Expect<Equal<typeof slot.current, "a" | "b">>;
type _VisitedIsTyped = Expect<Equal<typeof wizard.visited, readonly ("a" | "b")[]>>;
type _DirectionIsClosed = Expect<Equal<FormWizardDirection, "back" | "forward">>;
type _StateIsClosed = Expect<Equal<FormWizardState, "complete" | "in-progress" | "validating">>;

const props: WizardProps<"a" | "b"> = {
  steps: ["a", "b"],
  defaultValue: "b",
  validate: ({ step, target, signal }) => step === "a" && target === "b" && !signal.aborted,
  onChange: (step: "a" | "b", previous: "a" | "b", direction: FormWizardDirection) => {
    void step;
    void previous;
    void direction;
  },
};
const draft: FormWizardDraftStore<"a" | "b"> = {
  load: () => null,
  save: (snapshot) => {
    const step: "a" | "b" = snapshot.step;
    void step;
  },
};
const stepProps: InstanceType<typeof FormWizardStep>["$props"] = { step: "a", label: "A" };
const nextProps: InstanceType<typeof FormWizardNext>["$props"] = {};
const backProps: InstanceType<typeof FormWizardBack>["$props"] = {};
const progressProps: InstanceType<typeof FormWizardProgress>["$props"] = {
  formatValueText: (step, count) => `${step}/${count}`,
};

void wizard.goTo("b");

// @ts-expect-error unknown step ids are rejected.
void wizard.goTo("c");

// @ts-expect-error the default step must be one of the steps.
const invalidDefault: WizardProps<"a" | "b"> = { steps: ["a", "b"], defaultValue: "c" };

// @ts-expect-error gates return booleans.
const invalidGate: WizardProps<"a"> = { steps: ["a"], validate: () => "ok" };

void backProps;
void draft;
void invalidDefault;
void invalidGate;
void nextProps;
void progressProps;
void props;
void stepProps;
