import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Rule identifiers reported by the accessibility auditor. */
export type A11yAuditRule =
  | "accessible-name"
  | "dangling-idref"
  | "dialog-name"
  | "image-alt"
  | "landmark-name";

/** One accessibility problem found under an audited root. */
export interface A11yAuditIssue {
  /** Rule that produced the issue. */
  readonly rule: A11yAuditRule;

  /** Offending element. */
  readonly element: Element;

  /** `data-vize-ui` part of the element or its nearest ancestor, when any. */
  readonly part: string | null;

  /** Human-readable explanation with a suggested fix. */
  readonly message: string;
}

/** Options accepted by {@link auditAccessibility}. */
export interface A11yAuditOptions {
  /**
   * Only audit `@vizejs/ui` parts (`[data-vize-ui]` and their descendants).
   *
   * @default true
   */
  readonly onlyUiParts?: boolean;

  /**
   * Rules to skip.
   *
   * @default []
   */
  readonly ignore?: readonly A11yAuditRule[];
}

/** Options accepted by {@link useA11yAudit}. */
export interface UseA11yAuditOptions extends A11yAuditOptions {
  /**
   * Re-audit when the subtree changes.
   *
   * @default true
   */
  readonly observe?: boolean;

  /**
   * Called with every newly found issue. Defaults to a `console.warn` in development.
   *
   * @default undefined
   */
  readonly onIssue?: (issue: A11yAuditIssue) => void;
}

/** Controller returned by {@link useA11yAudit}. */
export interface A11yAuditController {
  /** Whether auditing is active (development builds only). */
  readonly enabled: boolean;

  /** Issues found by the latest audit. Always empty in production builds. */
  readonly issues: Readonly<ShallowRef<readonly A11yAuditIssue[]>>;

  /** Audit immediately and return the issues. */
  readonly audit: () => readonly A11yAuditIssue[];
}

/** Root accepted by {@link useA11yAudit}. */
export type A11yAuditTarget = MaybeRefOrGetter<Element | null | undefined>;
