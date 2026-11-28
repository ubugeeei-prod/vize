import { injectNuxtComponentImports } from "./components.ts";
import type { createNuxtComponentBridge } from "./component-hmr.ts";
import * as bridgeFastPath from "./bridge-fast-path.ts";
import { NUXT_TRANSFORM_BRIDGE_ID_FILTER } from "./module-id.ts";
import type { VizeNuxtBridgeOptions } from "./options.ts";
import {
  isVizeGeneratedVueModuleId,
  isVizeJsxModuleId,
  preserveExplicitVueImportsFromNuxtAutoImports,
  preserveExplicitVueImportsFromVizeModuleSource,
  stabilizeNuxtInjectedKeysForVizeVirtualModule,
} from "./utils.ts";

export interface NuxtUnimportContext {
  injectImports(
    code: string,
    id?: string,
  ): Promise<{ code: string; s: unknown; imports: unknown[] }>;
}

interface TransformBridgeOptions {
  bridge: Required<VizeNuxtBridgeOptions>;
  components: ReturnType<typeof createNuxtComponentBridge> | null;
  getUnimportContext(): NuxtUnimportContext | null;
  isBuild(): boolean;
  isSsrTransform(args: unknown[]): boolean;
}

type BridgeContext = { environment?: object };

/** Keep dynamic Nuxt hook state live while filtering dispatch in the host. */
export function createNuxtTransformBridge(options: TransformBridgeOptions) {
  return {
    name: "vizejs:nuxt-transform-bridge",
    enforce: "post" as const,
    hotUpdate: options.components?.hotUpdate,
    transform: {
      filter: { id: NUXT_TRANSFORM_BRIDGE_ID_FILTER },
      async handler(this: BridgeContext, code: string, id: string, ...args: unknown[]) {
        // Bridge owned Vue query IDs and in-place JSX/TSX components.
        // Legacy null-byte and .vue.ts[x] IDs remain supported.
        if (!isVizeGeneratedVueModuleId(id) && !isVizeJsxModuleId(id)) return;

        let result = code;
        let changed = false;

        // 1. Component auto-imports: replace _resolveComponent("Name") with direct imports
        // Nuxt's LoaderPlugin normally does this, but skips \0-prefixed IDs.
        const resolveComponent = options.components?.resolverFor(
          this.environment,
          id,
          options.isSsrTransform(args) ? "server" : "client",
        );
        if (resolveComponent && bridgeFastPath.hasComponentBridgeInput(result)) {
          const nextComponentResult = injectNuxtComponentImports(result, resolveComponent);
          if (nextComponentResult !== result) {
            result = nextComponentResult;
            changed = true;
          }
        }

        // 2. i18n function injection: inject useI18n() for $t, $rt, $d, $n, $tm, $te
        // @nuxtjs/i18n's TransformI18nFunctionPlugin skips \0-prefixed IDs.
        // Must inject inside the setup() function body, not at module top level.
        // Parse the virtual module so string literals, comments, and Vue template
        // globals do not look like setup-scope runtime helper calls.
        if (options.bridge.i18n && bridgeFastPath.hasI18nBridgeInput(result)) {
          const { injectNuxtI18nHelpers } = await import("./i18n.ts");
          const nextResult = injectNuxtI18nHelpers(result, id);
          if (nextResult !== result) {
            result = nextResult;
            changed = true;
          }
        }

        // 3. Composable auto-imports: inject useRoute, ref, computed, useI18n, etc.
        // Nuxt's unimport TransformPlugin normally does this, but skips \0-prefixed IDs.
        // Runs after i18n injection so unimport picks up the `useI18n` reference.
        const unimportCtx = options.getUnimportContext();
        if (unimportCtx) {
          try {
            const beforeUnimport = result;
            const injected = await unimportCtx.injectImports(result, id);
            if (injected.imports && injected.imports.length > 0) {
              result = preserveExplicitVueImportsFromNuxtAutoImports(beforeUnimport, injected.code);
              changed = true;
            }
          } catch {
            // Ignore errors — auto-imports might not be needed for all modules
          }
        }

        if (options.bridge.autoImports) {
          const nextResult = preserveExplicitVueImportsFromVizeModuleSource(id, result);
          if (nextResult !== result) {
            result = nextResult;
            changed = true;
          }
        }

        if (options.bridge.stableInjectedKeys && bridgeFastPath.hasStableKeyBridgeInput(result)) {
          const stableKeyResult = stabilizeNuxtInjectedKeysForVizeVirtualModule(result, id);
          if (stableKeyResult !== result) {
            result = stableKeyResult;
            changed = true;
          }
        }

        if (options.isBuild() && !options.isSsrTransform(args)) {
          const clientRuntimeResult = bridgeFastPath.rewriteBareVueImportsToClientRuntime(result);
          if (clientRuntimeResult !== result) {
            result = clientRuntimeResult;
            changed = true;
          }
        }

        if (changed) {
          return { code: result, map: null };
        }
      },
    },
  };
}
