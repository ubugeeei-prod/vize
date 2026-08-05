/**
 * Vue runtime define parity probes for the `@vitejs/plugin-vue` gate (#3227).
 *
 * Each `features.*` flag resolves to a `__VUE_*__` define, and the shape of that
 * resolution — plugin option, then user `define`, then a default — is easy to
 * get subtly wrong. These probes run the real upstream plugin as the oracle and
 * compare Vize's `config` hook against it.
 *
 * They live beside the parity test rather than inside it to keep that file
 * within its source-length budget.
 */

import assert from "node:assert/strict";
import type { Plugin } from "vite";

import { createUpstreamPlugin } from "./vite-plugin-vue-parity.ts";
import { vize } from "../../../npm/builder/vite/src/plugin/index.ts";

type AnyHook = (...args: never[]) => unknown;

export type DefineParityCase = {
  expected: boolean;
  options: Record<string, unknown>;
  userDefine: Record<string, unknown>;
};

function hook<T extends AnyHook>(candidate: unknown): T {
  const handler =
    typeof candidate === "function" ? candidate : (candidate as { handler?: T })?.handler;
  assert.equal(typeof handler, "function", "expected an implemented plugin hook");
  return handler as T;
}

/**
 * A `features.*` flag resolves to the same Vue runtime define as plugin-vue.
 *
 * The upstream plugin is the oracle, but each case still pins its own expected
 * value: a change in upstream semantics then fails as itself, instead of
 * silently redefining what parity means.
 */
export async function probeFeatureDefine(
  defineName: string,
  cases: ReadonlyArray<DefineParityCase>,
): Promise<void> {
  for (const { expected, options, userDefine } of cases) {
    // Each hook gets its own config object so the parity assertion does not
    // depend on either plugin leaving its input untouched.
    const upstreamConfig = { define: { ...userDefine } };
    const vizeConfig = { define: { ...userDefine } };
    const env = { command: "build", mode: "production" } as const;
    const upstream = createUpstreamPlugin(options) as Plugin;
    const vizePlugin = vize({ configMode: false, ...options }).find(
      (candidate) => candidate.name === "vite-plugin-vize",
    );
    assert.ok(vizePlugin);

    const upstreamResult = await hook<AnyHook>(upstream.config).call({}, upstreamConfig, env);
    const vizeResult = await hook<AnyHook>(vizePlugin.config).call({}, vizeConfig, env);
    const upstreamDefine = (upstreamResult as { define: Record<string, unknown> }).define[
      defineName
    ];
    const vizeDefine = (vizeResult as { define: Record<string, unknown> }).define[defineName];

    assert.equal(upstreamDefine, expected, "the pinned upstream oracle must stay stable");
    assert.equal(vizeDefine, upstreamDefine, `Vize must match plugin-vue's ${defineName}`);
  }
}

/** `features.optionsAPI` produces the same Vue runtime define as plugin-vue. */
export const probeOptionsApiFeature = (): Promise<void> =>
  probeFeatureDefine("__VUE_OPTIONS_API__", [
    { expected: true, options: {}, userDefine: {} },
    { expected: false, options: { features: { optionsAPI: false } }, userDefine: {} },
    {
      expected: true,
      options: { features: { optionsAPI: true } },
      userDefine: { __VUE_OPTIONS_API__: false },
    },
    { expected: false, options: {}, userDefine: { __VUE_OPTIONS_API__: "false" } },
  ]);

/**
 * `features.prodDevtools` matches plugin-vue's OR semantics.
 *
 * The define used to be `command === "serve"`, so a production build could not
 * turn devtools on at all (#3227). Upstream defaults it to `false` even in dev,
 * where Vue enables devtools through its own `__DEV__` instead.
 */
export const probeProdDevtoolsFeature = (): Promise<void> =>
  probeFeatureDefine("__VUE_PROD_DEVTOOLS__", [
    { expected: false, options: {}, userDefine: {} },
    { expected: false, options: { features: { prodDevtools: false } }, userDefine: {} },
    { expected: true, options: { features: { prodDevtools: true } }, userDefine: {} },
    {
      expected: true,
      options: { features: { prodDevtools: false } },
      userDefine: { __VUE_PROD_DEVTOOLS__: true },
    },
    { expected: true, options: {}, userDefine: { __VUE_PROD_DEVTOOLS__: "true" } },
    { expected: false, options: {}, userDefine: { __VUE_PROD_DEVTOOLS__: "false" } },
  ]);

/** `features.prodHydrationMismatchDetails` matches plugin-vue's OR semantics. */
export const probeProdHydrationMismatchDetailsFeature = (): Promise<void> =>
  probeFeatureDefine("__VUE_PROD_HYDRATION_MISMATCH_DETAILS__", [
    { expected: false, options: {}, userDefine: {} },
    {
      expected: false,
      options: { features: { prodHydrationMismatchDetails: false } },
      userDefine: {},
    },
    {
      expected: true,
      options: { features: { prodHydrationMismatchDetails: true } },
      userDefine: { __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: false },
    },
    {
      expected: true,
      options: { features: { prodHydrationMismatchDetails: false } },
      userDefine: { __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: true },
    },
    {
      expected: true,
      options: {},
      userDefine: { __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "true" },
    },
    {
      expected: false,
      options: {},
      userDefine: { __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "false" },
    },
  ]);
