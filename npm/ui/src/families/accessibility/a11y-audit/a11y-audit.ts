/** Development-only accessibility auditor for rendered `@vizejs/ui` parts. */
export { a11yAuditEnabled, useA11yAudit } from "./a11y-audit-runtime.ts";
export { accessibleNameOf, auditAccessibility } from "./a11y-audit-rules.ts";
export type {
  A11yAuditController,
  A11yAuditIssue,
  A11yAuditOptions,
  A11yAuditRule,
  A11yAuditTarget,
  UseA11yAuditOptions,
} from "./a11y-audit-types.ts";
