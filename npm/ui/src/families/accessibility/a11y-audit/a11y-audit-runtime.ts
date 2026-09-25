import {
  getCurrentScope,
  onMounted,
  onScopeDispose,
  shallowReadonly,
  shallowRef,
  toValue,
} from "vue";

import { a11yAuditDiagnostic, auditAccessibility } from "./a11y-audit-rules.ts";
import type {
  A11yAuditController,
  A11yAuditIssue,
  A11yAuditTarget,
  UseA11yAuditOptions,
} from "./a11y-audit-types.ts";

/**
 * Whether development-only auditing is active. Bundlers replace
 * `process.env.NODE_ENV`, so production builds fold this to `false` and drop the
 * auditor entirely.
 */
export const a11yAuditEnabled: boolean =
  typeof process !== "undefined" && process.env.NODE_ENV !== "production";

const emptyIssues: readonly A11yAuditIssue[] = Object.freeze([]);

function warn(issue: A11yAuditIssue): void {
  console.warn(`[${a11yAuditDiagnostic}] ${issue.rule}: ${issue.message}`, issue.element);
}

function createDisabledController(): A11yAuditController {
  const issues = shallowRef(emptyIssues);
  return Object.freeze({
    audit: () => emptyIssues,
    enabled: false,
    issues: shallowReadonly(issues),
  });
}

/**
 * Warn about accessibility mistakes in a rendered subtree during development.
 *
 * The composable audits after mount and, with `observe`, after DOM changes. Each
 * element/rule pair is reported once. In production builds it is a no-op and the
 * audit code is eliminated.
 */
export function useA11yAudit(
  target: A11yAuditTarget,
  options: UseA11yAuditOptions = {},
): A11yAuditController {
  if (!getCurrentScope()) {
    throw new Error(`${a11yAuditDiagnostic}: use inside component setup or an active effect scope`);
  }
  if (!a11yAuditEnabled) return createDisabledController();

  const issues = shallowRef(emptyIssues);
  const reported = new WeakMap<Element, Set<string>>();
  const report = options.onIssue ?? warn;
  let observer: MutationObserver | null = null;
  let scheduled = false;

  const audit = (): readonly A11yAuditIssue[] => {
    const root = toValue(target);
    if (!root) {
      issues.value = emptyIssues;
      return emptyIssues;
    }
    const found = auditAccessibility(root, options);
    issues.value = found;
    for (const item of found) {
      const seen = reported.get(item.element) ?? new Set<string>();
      if (seen.has(item.rule)) continue;
      seen.add(item.rule);
      reported.set(item.element, seen);
      report(item);
    }
    return found;
  };

  const schedule = (): void => {
    if (scheduled) return;
    scheduled = true;
    queueMicrotask(() => {
      scheduled = false;
      audit();
    });
  };

  onMounted(() => {
    audit();
    const root = toValue(target);
    if ((options.observe ?? true) && root && typeof MutationObserver === "function") {
      observer = new MutationObserver(schedule);
      observer.observe(root, {
        attributes: true,
        characterData: true,
        childList: true,
        subtree: true,
      });
    }
  });

  onScopeDispose(() => {
    observer?.disconnect();
    observer = null;
  });

  return Object.freeze({ audit, enabled: true, issues: shallowReadonly(issues) });
}
