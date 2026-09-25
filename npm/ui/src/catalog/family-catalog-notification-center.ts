import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const notificationCenterFamilyRoot = "src/families/feedback/notification-center/";

export const notificationCenterFamilyCatalog = [
  {
    canonicalName: "notification-center",
    title: "Notification Center",
    packageSubpath: "./notification-center",
    entryFile: `${notificationCenterFamilyRoot}notification-center.ts`,
    sourceFiles: [
      `${notificationCenterFamilyRoot}notification-center-context.ts`,
      `${notificationCenterFamilyRoot}notification-center-empty.vue`,
      `${notificationCenterFamilyRoot}notification-center-item.vue`,
      `${notificationCenterFamilyRoot}notification-center-list.vue`,
      `${notificationCenterFamilyRoot}notification-center-root.vue`,
      `${notificationCenterFamilyRoot}notification-center-store.ts`,
      `${notificationCenterFamilyRoot}notification-center-trigger.vue`,
      `${notificationCenterFamilyRoot}notification-center-types.ts`,
      `${notificationCenterFamilyRoot}notification-center.ts`,
    ],
    behaviorContract: `${notificationCenterFamilyRoot}notification-center.behavior.md`,
    tests: [
      `${notificationCenterFamilyRoot}notification-center.test.ts`,
      `${notificationCenterFamilyRoot}notification-center-ssr.test.ts`,
      `${notificationCenterFamilyRoot}notification-center-store.test.ts`,
    ],
    typeTests: [`${notificationCenterFamilyRoot}notification-center.types.test-d.ts`],
    rendererFixture: "NotificationCenterConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "NotificationCenterRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}notification-center-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 3_500,
      maximumCssGzipBytes: 0,
    },
    aliases: ["notification center", "inbox", "activity feed", "toast history", "notifications"],
    upstreamCoverage: [
      "WAI-ARIA Feed pattern",
      "Mantine Notifications",
      "Novu Inbox",
      "Sonner toast history recipes",
    ],
    dependencies: ["context", "controllable-state", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
