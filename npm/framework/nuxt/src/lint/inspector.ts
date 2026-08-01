import type {
  VizeNuxtCompilerOptions,
  VizeNuxtInspectorLintPlanRequest,
} from "../compiler-options.ts";
import type { NuxtLintConfigGeneration } from "./generation.ts";
import { setupNuxtLintDevtools, type NuxtLintDevtoolsNuxt } from "./inspector-devtools.ts";
import type { VizeNuxtLintOptions } from "./options.ts";

type Awaitable<T> = T | Promise<T>;
type InspectLintPlan = (plan: string, root: string, files: string[]) => Awaitable<string>;

export interface NuxtLintInspectorDependencies {
  inspectLintPlan?: InspectLintPlan;
}

export function createNuxtLintInspectorProvider(
  generation: Pick<NuxtLintConfigGeneration, "resolvePlan" | "root">,
  dependencies: NuxtLintInspectorDependencies = {},
): (request: VizeNuxtInspectorLintPlanRequest) => Promise<Record<string, unknown>> {
  const inspect = dependencies.inspectLintPlan ?? inspectWithNative;

  return async (request) => {
    const items = await generation.resolvePlan(request.fresh);
    const serialized = await inspect(JSON.stringify({ items }), generation.root, request.files);
    const payload = JSON.parse(serialized) as unknown;
    if (!isLintPlanPayload(payload)) {
      throw new Error("Native lint-plan inspector returned an invalid payload");
    }
    return payload;
  };
}

export async function setupLintInspector(
  lint: boolean | VizeNuxtLintOptions | undefined,
  nuxt: NuxtLintDevtoolsNuxt,
  compiler: false | VizeNuxtCompilerOptions,
  generation: NuxtLintConfigGeneration | undefined,
  enabled: boolean | undefined,
): Promise<void> {
  if (enabled === false || !generation) return;
  const provider = createNuxtLintInspectorProvider(generation);
  if (compiler !== false) {
    compiler.inspector ||= {};
    compiler.inspector.lintPlan ||= provider;
  }
  const devtools = typeof lint === "object" && lint !== null ? lint.devtools : undefined;
  await setupNuxtLintDevtools(
    devtools,
    nuxt,
    compiler === false ? provider : compiler.inspector.lintPlan,
  );
}

async function inspectWithNative(plan: string, root: string, files: string[]): Promise<string> {
  const { inspectLintPlan } = await import("@vizejs/native");
  return inspectLintPlan(plan, root, files);
}

function isLintPlanPayload(value: unknown): value is Record<string, unknown> {
  return (
    value !== null &&
    typeof value === "object" &&
    !Array.isArray(value) &&
    (value as Record<string, unknown>).schema === "vize.inspector.lint-plan" &&
    (value as Record<string, unknown>).version === 1 &&
    typeof (value as Record<string, unknown>).root === "string" &&
    Array.isArray((value as Record<string, unknown>).items) &&
    Array.isArray((value as Record<string, unknown>).files)
  );
}
