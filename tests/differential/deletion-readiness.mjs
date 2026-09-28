import { classifyNativeRow, validateResultEnvelope } from "./harness.mjs";

export const deletionProducts = ["compiler", "linter", "formatter", "typechecker", "lsp"];
export const deletionTiers = ["T1", "T2"];
export const deletionDialects = [
  "vue0-template",
  "vue1-template",
  "vue2-sfc",
  "vue2.7-sfc",
  "vue3-sfc",
  "petite-vue",
  "pug-template",
  "jsx-babel",
  "jsx-vapor",
  "js",
  "ts",
  "jsx",
  "tsx",
  "vue-quirks",
];

const shaPattern = /^[a-f0-9]{40}$/;
const dayPattern = /^\d{4}-\d{2}-\d{2}$/;

export function assessDeletionReadiness({
  policy,
  projectIds,
  candidate,
  history,
  headSha,
  today,
  policySha256,
  registrySha256,
  adapters = {},
  verifyRun,
}) {
  const blockers = [];
  const block = (code, detail) => blockers.push({ code, detail });
  const targets = policy?.targets ?? {};
  const days = policy?.minimumStableDays;
  const gap = policy?.maximumGapDays;
  if (!Number.isInteger(days) || days < 1 || !Number.isInteger(gap) || gap < 1) {
    block(
      "stability-policy",
      "Approved positive minimumStableDays and maximumGapDays are required",
    );
  }
  if (!shaPattern.test(headSha ?? "")) block("head-sha", "Exact candidate SHA is required");
  if (!dayPattern.test(today ?? "")) block("today", "UTC evaluation date is required");
  if (!/^[a-f0-9]{64}$/.test(policySha256 ?? "")) block("policy-sha", "Policy digest is required");
  if (!/^[a-f0-9]{64}$/.test(registrySha256 ?? ""))
    block("registry-sha", "Project registry digest is required");
  if (typeof verifyRun !== "function") block("run-verifier", "Actions run verifier is required");
  const projects = new Set(projectIds ?? []);
  if (projects.size !== projectIds?.length || projects.size < 146) {
    block("project-registry", "Pinned project registry must retain at least 146 unique projects");
  }
  for (const product of deletionProducts) {
    if (
      !Array.isArray(targets[product]) ||
      !targets[product].length ||
      new Set(targets[product]).size !== targets[product].length
    ) {
      block("target-policy", `${product}: approved nonempty target set is required`);
    }
    if (
      !adapters[product]?.requiredStages?.length ||
      typeof adapters[product]?.verifyBuildReceipt !== "function" ||
      typeof adapters[product]?.verifyObservation !== "function" ||
      typeof adapters[product]?.verifyComparison !== "function"
    ) {
      block("product-verifier", `${product}: product observation and comparator verifier required`);
    }
  }
  if (Object.keys(targets).some((product) => !deletionProducts.includes(product))) {
    block("target-policy", "Unknown product in target policy");
  }
  if (
    !Array.isArray(policy?.dialects) ||
    policy.dialects.length !== deletionDialects.length ||
    new Set(policy.dialects).size !== deletionDialects.length ||
    deletionDialects.some((dialect) => !policy.dialects.includes(dialect))
  ) {
    block("dialect-policy", "Policy must include every canonical dialect exactly once");
  }
  if (!candidate?.tiers?.T1 || !candidate?.tiers?.T2) {
    block("candidate-evidence", "Exact candidate needs both T1 and on-demand T2 evidence");
  }
  if (candidate?.sourceRevision !== headSha) {
    block("candidate-sha", "T1 and T2 must cover the exact deletion candidate SHA");
  }
  if (!Array.isArray(history) || history.length < 2) {
    block("stability-history", "At least two scheduled T2 observations are required");
  }
  const ordered = Array.isArray(history)
    ? [...history].sort((a, b) => String(a.day).localeCompare(String(b.day)))
    : [];
  const dayNumber = (day) => Date.parse(`${day}T00:00:00Z`);
  const windowStart =
    ordered.length && Number.isInteger(days) && days > 0
      ? ordered.findLastIndex(
          (snapshot) =>
            dayNumber(snapshot.day) <= dayNumber(ordered.at(-1).day) - days * 86_400_000,
        )
      : 0;
  const stable = ordered.slice(Math.max(windowStart, 0));
  if (windowStart < 0 || stable.length < 2) {
    block("stability-history", `Scheduled T2 observations must span at least ${days} days`);
  }
  if (new Set(stable.map((snapshot) => snapshot.day)).size !== stable.length) {
    block("stability-history", "At most one scheduled T2 observation per UTC day is allowed");
  }
  if (stable.length && Number.isInteger(gap) && gap > 0) {
    const intervals = [
      ...stable.slice(1).map((item, index) => dayNumber(item.day) - dayNumber(stable[index].day)),
      dayNumber(today) - dayNumber(stable.at(-1).day),
    ];
    if (
      intervals.some(
        (interval) => !Number.isFinite(interval) || interval < 0 || interval > gap * 86_400_000,
      )
    ) {
      block("stability-history", `Scheduled T2 observations have a gap over ${gap} days`);
    }
  }
  const observations = [
    ...(candidate ? [{ ...candidate, day: today, tiersToCheck: deletionTiers }] : []),
    ...stable.map((snapshot) => ({ ...snapshot, tiersToCheck: ["T2"] })),
  ];
  if (!observations.length) block("evidence", "No T1/T2 observations were supplied");
  for (const snapshot of observations) {
    if (!dayPattern.test(snapshot.day ?? "") || !shaPattern.test(snapshot.sourceRevision ?? "")) {
      block("snapshot-identity", `${snapshot.day}: valid UTC day and source SHA required`);
    }
    if (snapshot.policySha256 !== policySha256 || snapshot.registrySha256 !== registrySha256) {
      block(
        "scope-drift",
        `${snapshot.day}: policy and project registry must stay fixed for the period`,
      );
    }
    for (const tier of snapshot.tiersToCheck) {
      if (
        typeof verifyRun === "function" &&
        verifyRun(snapshot.runs?.[tier], tier, snapshot.sourceRevision, snapshot.day) !== true
      ) {
        block(
          "run-provenance",
          `${snapshot.day}/${tier}: successful exact-SHA Actions run required`,
        );
      }
    }
    for (const product of deletionProducts) {
      const seenDialects = new Set();
      for (const tier of snapshot.tiersToCheck) {
        const entry = snapshot.tiers?.[tier]?.[product];
        const context = `${snapshot.day}/${tier}/${product}`;
        if (!entry?.loaded || !entry?.report) {
          block("missing-result", `${context}: manifest and result are required`);
          continue;
        }
        try {
          validateResultEnvelope(entry.loaded, entry.report, snapshot.sourceRevision);
        } catch (error) {
          block("invalid-result", `${context}: ${error.message}`);
          continue;
        }
        const adapter = adapters[product];
        if (
          typeof adapter?.verifyBuildReceipt === "function" &&
          adapter.verifyBuildReceipt(entry.report, snapshot.sourceRevision) !== true
        ) {
          block("build-receipt", `${context}: source-built product receipt is unverified`);
        }
        const plannedTargets = new Set(targets[product] ?? []);
        const observedTargets = new Set();
        const observedProjects = new Set();
        const rows = new Map(
          entry.report.rows.map((row) => [
            JSON.stringify([
              row.id,
              row.target ?? entry.loaded.cases.find((item) => item.id === row.id)?.targets[0],
            ]),
            row,
          ]),
        );
        for (const fixture of entry.loaded.cases) {
          if (fixture.state !== "active")
            block("inactive-case", `${context}/${fixture.id}: draft is not deletion evidence`);
          const coverage = fixture.dialectCoverage;
          const dialects = coverage?.evidence?.flatMap((item) => item.dialects ?? []) ?? [];
          if (
            coverage?.state !== "verified" ||
            !dialects.length ||
            dialects.some((dialect) => !deletionDialects.includes(dialect))
          ) {
            block("unknown-dialect", `${context}/${fixture.id}: verified case dialects required`);
          } else {
            for (const dialect of dialects) seenDialects.add(dialect);
          }
          if (tier === "T2") {
            if (!projects.has(fixture.projectId)) {
              block("unknown-project", `${context}/${fixture.id}: pinned project ID required`);
            } else {
              observedProjects.add(fixture.projectId);
            }
          }
          for (const target of fixture.targets) {
            observedTargets.add(target);
            if (!plannedTargets.has(target))
              block("unplanned-target", `${context}/${fixture.id}/${target}`);
            const row = rows.get(JSON.stringify([fixture.id, target]));
            if (!row) continue; // validateResultEnvelope reports a missing row above.
            if (row.legacy.state !== "completed")
              block("legacy-observation", `${context}/${fixture.id}/${target}`);
            if (!adapter?.requiredStages?.length || typeof adapter.verifyObservation !== "function")
              continue;
            const native = classifyNativeRow(row, {
              sourceRevision: snapshot.sourceRevision,
              buildReceiptSha256: entry.report.buildReceiptSha256,
              requiredStages: adapter.requiredStages,
              verifyObservation: (observation) =>
                adapter.verifyObservation(observation, fixture, row, entry.report),
            });
            if (native !== "native-handled")
              block("non-native", `${context}/${fixture.id}/${target}: ${native}`);
            if (
              typeof adapter.verifyComparison !== "function" ||
              adapter.verifyComparison(row, fixture, entry.report) !== true
            ) {
              block("non-equivalent", `${context}/${fixture.id}/${target}`);
            }
          }
        }
        for (const target of plannedTargets) {
          if (!observedTargets.has(target)) block("missing-target", `${context}/${target}`);
        }
        if (tier === "T2") {
          for (const project of projects) {
            if (!observedProjects.has(project)) block("missing-project", `${context}/${project}`);
          }
        }
      }
      for (const dialect of deletionDialects) {
        if (!seenDialects.has(dialect))
          block("missing-dialect", `${snapshot.day}/${product}/${dialect}`);
      }
    }
  }
  return { ready: blockers.length === 0, blockers };
}
