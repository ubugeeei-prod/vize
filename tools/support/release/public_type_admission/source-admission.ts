/** Immutable source execution identity; this is not public-consumer acceptance. */
export const SOURCE_ADMISSION = {
  schema: "vize.template-ref.source-admission.v1",
  scope:
    "Source and merge-queue execution; public acceptance is recorded separately by this capsule.",
  pullRequest: 10950,
  requiredSource: "83e3ff318fb772340663b5688624dac2ad37e2b1",
  pullRequestHead: "a5ef4d8ebb68619db22f397e95825df1676f6af1",
  mergeQueue: {
    runId: 40603113935,
    jobId: 122536275919,
    source: "83e3ff318fb772340663b5688624dac2ad37e2b1",
    runUrl: "https://github.com/ubugeeei-prod/vize/actions/runs/40603113935",
    jobUrl: "https://github.com/ubugeeei-prod/vize/actions/runs/40603113935/job/122536275919",
    nativePackage: "@typescript/typescript-linux-x64",
    nativeVersion: "7.0.2",
    requiredEnvironment: {
      VIZE_TEST_REQUIRE_TSGO: "1",
    },
    testsRun: 3359,
    testsPassed: 3359,
    testsSkipped: 612,
    binariesRun: 72,
    binariesSkipped: 642,
    criticalTests: [
      "batch::type_checker::tests::batch_type_checker_uses_workspace_vue_runtime_without_node_modules",
      "batch::type_checker::tests::template_ref_runtime_import::workspace_vue_runtime_preserves_missing_import_then_repair",
    ],
  },
  scalePullRequest: {
    number: 10987,
    mergeSource: "91560a59584e71f24cb2070e99c2f99ae652861c",
    runId: 40603117010,
  },
  explicitlyExcludedVersions: ["0.602.0", "0.603.0"],
  globalOriginIssue: 10980,
  globalOriginQualified: false,
} as const;
