/** Compile-only assertions for the public accessibility auditor contract. */

import type { ShallowRef } from "vue";

import type {
  A11yAuditController,
  A11yAuditIssue,
  A11yAuditRule,
  UseA11yAuditOptions,
} from "./a11y-audit.ts";
import {
  a11yAuditEnabled,
  accessibleNameOf,
  auditAccessibility,
  useA11yAudit,
} from "./a11y-audit.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const controller: A11yAuditController;
declare const issue: A11yAuditIssue;

type _Rule = Expect<
  Equal<
    A11yAuditRule,
    "accessible-name" | "dangling-idref" | "dialog-name" | "image-alt" | "landmark-name"
  >
>;
type _Issues = Expect<
  Equal<typeof controller.issues, Readonly<ShallowRef<readonly A11yAuditIssue[]>>>
>;
type _Audit = Expect<Equal<ReturnType<typeof auditAccessibility>, readonly A11yAuditIssue[]>>;
type _Name = Expect<Equal<ReturnType<typeof accessibleNameOf>, string>>;
type _Enabled = Expect<Equal<typeof a11yAuditEnabled, boolean>>;
type _Part = Expect<Equal<typeof issue.part, string | null>>;
type _Composable = Expect<Equal<ReturnType<typeof useA11yAudit>, A11yAuditController>>;

const options: UseA11yAuditOptions = {
  ignore: ["image-alt"],
  observe: false,
  onIssue: (found) => void found.rule,
  onlyUiParts: false,
};

// @ts-expect-error rules are a closed union.
const badIgnore: UseA11yAuditOptions = { ignore: ["color-contrast"] };

// @ts-expect-error roots are elements.
auditAccessibility("#app");

void badIgnore;
void options;
