// Workflow file naming rules. A workflow is `NN-<subject>.yml`, where `NN` is its
// catalog group and `<subject>` names the product surface or measurement it covers.
// Protected files keep the names that external trust bindings (npm, JSR, crates.io,
// GitHub Pages, pkg.pr.new and the Parcel publisher) record for them.

export type WorkflowGroupId =
  | "01"
  | "02"
  | "03"
  | "04"
  | "05"
  | "06"
  | "07"
  | "08"
  | "09"
  | "10"
  | "11";

export const protectedWorkflows: Readonly<Record<string, WorkflowGroupId>> = {
  "deploy-docs.yml": "11",
  "parcel-transformer.yml": "08",
  "pkg-pr-new.yml": "02",
  "published-fresh-project.yml": "02",
  "release-cadence.yml": "02",
  "release-jsr.yml": "02",
  "release-npm-bootstrap.yml": "02",
  "release-verify.yml": "02",
  "release.yml": "02",
};

// Words that describe one investigation rather than a lasting surface.
const bannedSubjectWords = new Set([
  "candidate",
  "custody",
  "evidence",
  "frontier",
  "original",
  "packets",
  "pair",
  "paired",
  "public",
  "published",
  "replay",
]);

// Words the group heading already says.
const groupImpliedWords: Partial<Record<WorkflowGroupId, readonly string[]>> = {
  // `vue-benchmarks` names the upstream corpus, so only the generic words are implied.
  "03": ["bench", "benchmark"],
  "11": ["maintenance"],
};

// Names from before the naming rules. Nothing may join this list: a new workflow uses a
// prefixed name. The list goes away with the last rename; deleted names may linger
// until then, so parallel deletion PRs do not conflict here.
export const legacyWorkflowNames: ReadonlySet<string> = new Set([
  "ambient-template-ref-ownership.yml",
  "art-source-products.yml",
  "asset-import-bindings.yml",
  "async-component-ancestry.yml",
  "babel-rc9-frontier.yml",
  "binding-origin-laws.yml",
  "bundler-block-products.yml",
  "canon-component-casing.yml",
  "canon-helper-native-baseline.yml",
  "canon-instance-directives.yml",
  "canon-slot-constructor.yml",
  "canon-slot-provider-any.yml",
  "canon-windows-mirror-paths.yml",
  "checker-constructor-emitter.yml",
  "checker-diagnostic-positions.yml",
  "checker-styled-projections.yml",
  "client-server-decisions.yml",
  "cold-typechecker-scale.yml",
  "compact-collections.yml",
  "compiler-options-snapshot.yml",
  "component-binding-view-profile.yml",
  "component-input-diagnostic-ownership.yml",
  "component-input-projection-packets.yml",
  "component-scope-binding.yml",
  "content-mapper-instance-globals.yml",
  "content-mapper-pair.yml",
  "content-mapper-replay.yml",
  "content-mapper-reported-q16.yml",
  "content-mapper-span-allocation.yml",
  "content-mapper-vue-component-meta.yml",
  "content-mapper-worker-stack.yml",
  "corsa-package-refresh.yml",
  "cross-file-finalization-compare.yml",
  "cross-file-finalization-order.yml",
  "cross-file-finalization.yml",
  "cross-file-script-offset.yml",
  "cross-file-sfc-effects.yml",
  "cross-file-source-updates.yml",
  "cross-file-wasm-effects.yml",
  "css-color-declaration-admission.yml",
  "css-color-literal-hover.yml",
  "css-compile-source-maps.yml",
  "css-hover-visual.yml",
  "davinci-shared-toolchain-session.yml",
  "davinci-unused-bindings.yml",
  "defaulted-inject.yml",
  "diagnostic-digest-bench.yml",
  "doctor-barrel-props.yml",
  "doctor-filter-bench.yml",
  "doctor-lint-ssr-context.yml",
  "doctor-prop-accuracy.yml",
  "document-snapshot-source-profile.yml",
  "editor-script-queries.yml",
  "editor-vite-stub-public.yml",
  "empty-generic-public-contract.yml",
  "expression-overlap-audit.yml",
  "for-bindings.yml",
  "formatter-profiles-public.yml",
  "full-worker-contracts.yml",
  "fuzz-expression-10401-profile.yml",
  "glibc-floor-bench.yml",
  "js-install-closure.yml",
  "jsr-package.yml",
  "jsr-public-types.yml",
  "jsx-authored-native-modules.yml",
  "jsx-backend-origins.yml",
  "l1-document-special-text.yml",
  "language-project-fixtures.yml",
  "lint-unused-disable-config.yml",
  "lsp-config-diagnostics.yml",
  "lsp-dual-server-benchmark.yml",
  "lsp-focused-projects.yml",
  "lsp-jsx-intrinsic-hover.yml",
  "lsp-marker-scan-scale.yml",
  "lsp-native-diagnostic-coverage.yml",
  "lsp-native-diagnostic-warmup.yml",
  "lsp-overlay-custody.yml",
  "lsp-overlay-graph-reuse.yml",
  "lsp-release-resources.yml",
  "lsp-residency-pair.yml",
  "lsp-scheduler-cost-carrier.yml",
  "lsp-source-profile.yml",
  "lsp-tag-prefix-classification.yml",
  "mounted-runtime-devtools-comparison.yml",
  "musea-glyph-feature-boundary.yml",
  "n8n-script-rule-controls.yml",
  "n8n-whole-corpus-observer.yml",
  "named-sfc-value-hover.yml",
  "narrowed-slot-host-guard.yml",
  "native-build-token-layout.yml",
  "native-checker-cpu-metadata.yml",
  "native-image-alt-query.yml",
  "native-navigation-products.yml",
  "native-ops-incident.yml",
  "native-sfc-runtime-options.yml",
  "native-sfc-runtime-version.yml",
  "no-unused-script-vars-public.yml",
  "nuxt-generated-membership.yml",
  "nuxt-public-dependencies.yml",
  "open-document-blocks.yml",
  "packed-browser-equivalence.yml",
  "partition-weight-work.yml",
  "patina-attribute-ignore-public.yml",
  "patina-attribute-ignore-tags-reference.yml",
  "patina-boolean-public.yml",
  "patina-native-boolean-protocol.yml",
  "persistent-batch-storage.yml",
  "plain-script-import-public-acceptance.yml",
  "plain-script-import-ref.yml",
  "plugin-template-products.yml",
  "pr-source-cli.yml",
  "pre-textarea-first-newline.yml",
  "production-reach-accounting.yml",
  "profile-allocation-thread-law.yml",
  "provide-type-assertion.yml",
  "public-jsx-child-values.yml",
  "public-native-events.yml",
  "public-typechecker-consumer.yml",
  "published-ambient-template-ref.yml",
  "published-component-aria.yml",
  "published-computed-event-names.yml",
  "published-constructor-original27.yml",
  "published-human-reports-11252-11253.yml",
  "published-slot-instance-noany.yml",
  "published-slot-provider.yml",
  "qs-runtime-security.yml",
  "queue-release-gates.yml",
  "recovery-normal-codegen.yml",
  "release-build-cache.yml",
  "release-contracts.yml",
  "release-glibc-floor.yml",
  "release-smoke-contracts.yml",
  "release-zed-host.yml",
  "retained-script-formatting.yml",
  "retained-tsrx.yml",
  "router-editor-artifact.yml",
  "router-experimental-navigation.yml",
  "router-preview-conformance.yml",
  "rspack-src-block-ranges.yml",
  "rust-frontend-threads-bench.yml",
  "scoped-nesting-specificity.yml",
  "script-recovery-clone-cost.yml",
  "script-recovery-cost.yml",
  "setup-binding-anchor-profile.yml",
  "sfc-member-profile.yml",
  "sfc-optional-lifecycle.yml",
  "sfc-parse-attributes.yml",
  "sfc-parse-replay.yml",
  "source-operation-profile.yml",
  "source-symbolizer-retained.yml",
  "source-symbolizer-smoke.yml",
  "string-storage-bench.yml",
  "template-alias-paired-11822.yml",
  "template-expression-queries.yml",
  "template-global-policy.yml",
  "template-tool-projections.yml",
  "text-boundary-bench.yml",
  "type-admission-scale.yml",
  "type-snapshot-bench.yml",
  "typecheck-corpus-pair.yml",
  "typechecker-cache-custody.yml",
  "typechecker-syntax-semantic-completion.yml",
  "typed-context-undefined.yml",
  "typescript-plugin.yml",
  "typescript-provider-public.yml",
  "typescript-provider-vsix.yml",
  "unicode-prop-projection.yml",
  "v1-exact-candidate.yml",
  "vapor-literal-object-styles.yml",
  "versioned-fix-publication.yml",
  "vite-style-relative-imports.yml",
  "viteplus-lint-consumers.yml",
  "viteplus-task-cache.yml",
  "vitest-module-cache.yml",
  "vts-source-census.yml",
  "vue-benchmarks-current-qualification.yml",
  "vue2-editor-diagnostic-context.yml",
  "vue2-ir-template.yml",
  "vuefes-ssr-style-dependencies.yml",
  "windows-config-consumer.yml",
  "workspace-route-identity.yml",
]);

const prefixedName = /^(0[1-9]|1[01])-([a-z0-9]+(?:-[a-z0-9]+)*)\.yml$/u;

export function workflowPrefixGroup(file: string): WorkflowGroupId | undefined {
  return prefixedName.exec(file)?.[1] as WorkflowGroupId | undefined;
}

export function workflowNameViolations(files: readonly string[]): string[] {
  const violations: string[] = [];
  const present = new Set(files);
  // Protected names keep their subjects, so no prefixed file may reuse one.
  const subjects = new Map<string, string>(
    files.filter((file) => file in protectedWorkflows).map((file) => [file.slice(0, -4), file]),
  );
  for (const file of files) {
    if (file in protectedWorkflows || legacyWorkflowNames.has(file)) continue;
    const match = prefixedName.exec(file);
    if (match == null) {
      violations.push(`${file}: name must be NN-<subject>.yml`);
      continue;
    }
    const group = match[1] as WorkflowGroupId;
    const subject = match[2]!;
    for (const word of subject.split("-")) {
      if (bannedSubjectWords.has(word))
        violations.push(`${file}: "${word}" names an investigation`);
      if (/^\d+$/u.test(word)) violations.push(`${file}: "${word}" is an issue or version number`);
      if (groupImpliedWords[group]?.includes(word))
        violations.push(`${file}: group ${group} already implies "${word}"`);
    }
    const previous = subjects.get(subject);
    if (previous != null) violations.push(`${file}: subject is already used by ${previous}`);
    subjects.set(subject, file);
  }
  for (const file of Object.keys(protectedWorkflows))
    if (!present.has(file)) violations.push(`${file}: protected workflow is missing`);
  return violations;
}
