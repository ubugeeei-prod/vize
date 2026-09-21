// The negative control required by TS-45's acceptance: on the very same
// transcript, a deliberately wrong expectation must fail. `run.ts` corrupts
// every step in turn, so no expectation in `scenario.json` is decorative.
import { evaluate, type Entry, type Scenario, type Step } from "./conformance.ts";

/** A deterministic corruption of one step's expectation, for the negative control. */
export function corruptExpectation(step: Step): Step {
  const corrupt = (value: unknown): unknown => {
    if (typeof value === "string") return `${value}\u0000`;
    if (typeof value === "number") return value + 1;
    if (Array.isArray(value))
      return value.length > 0 ? [corrupt(value[0]), ...value.slice(1)] : [null];
    if (value !== null && typeof value === "object") {
      const [key] = Object.keys(value).sort();
      return key == null ? { corrupted: true } : { ...value, [key]: corrupt((value as any)[key]) };
    }
    return { corrupted: true };
  };
  return { ...step, expect: corrupt(step.expect) } as Step;
}

/** Every step must reject a corrupted expectation on the same transcript. */
export function negativeControl(scenario: Scenario, entries: Entry[], roots: string[]): string[] {
  const survivors: string[] = [];
  scenario.steps.forEach((step, index) => {
    const steps = scenario.steps.map((candidate, at) =>
      at === index ? corruptExpectation(step) : candidate,
    );
    const verdict = evaluate({ ...scenario, steps }, entries, roots).steps[index];
    if (verdict.pass) survivors.push(step.id);
  });
  return survivors;
}
