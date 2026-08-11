import { getCurrentInstance, getCurrentScope, onMounted, onScopeDispose } from "vue";

import { createFocusScope } from "./focus-scope.ts";
import type { FocusScopeController, FocusScopeOptions } from "./focus-scope-types.ts";

const setupDiagnostic = "VIZE_UI_FOCUS_SCOPE_SETUP";

/** Create, mount-activate, and scope-dispose a focus scope from Vue setup. */
export function useFocusScope(options: FocusScopeOptions): FocusScopeController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createFocusScope(options);
  if (getCurrentInstance()) onMounted(controller.activate);
  else controller.activate();
  onScopeDispose(controller.dispose);
  return controller;
}
