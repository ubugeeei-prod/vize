import type { ProjectDetection } from "./detect.js";
import { unreadOxlintConfig } from "./lint-target.js";
import type { InitPlan } from "./plan.js";
import { EDITOR_INTEGRATIONS } from "./templates.js";

const PREFIX = "[vize init]";

/**
 * Detection summary, printed before any prompt.
 *
 * Users need to see what `init` concluded before they are asked to act on it;
 * an unexpected line here is the cheapest place to catch a wrong root or a
 * missing lockfile.
 */
export function renderDetection(detection: ProjectDetection): string {
  const lines = [
    `${PREFIX} detected in ${detection.root}:`,
    `  framework:       ${describeFramework(detection)}`,
    `  package manager: ${detection.packageManager ?? "none detected (defaulting to npm)"}`,
    `  language:        ${detection.typescript ? "TypeScript" : "JavaScript"}${
      detection.tsconfig === null ? " (no tsconfig.json)" : " (tsconfig.json)"
    }`,
    `  lint command:    ${detection.usesVitePlus ? "vp lint" : "oxlint"}`,
    `  vize config:     ${detection.vizeConfig ?? "none"}`,
    `  oxlint config:   ${describeOxlintConfig(detection)}`,
  ];
  return `${lines.join("\n")}\n`;
}

function describeFramework(detection: ProjectDetection): string {
  if (detection.framework === "nuxt") {
    return `Nuxt (${detection.nuxtConfig ?? "nuxt dependency, no nuxt.config"})`;
  }
  if (detection.framework === "vite") {
    const configs = detection.viteConfigs.join(", ");
    return detection.usesVitePlus ? `Vite+ (${configs})` : `Vite (${configs})`;
  }
  return "none (no vite.config or nuxt.config)";
}

function describeOxlintConfig(detection: ProjectDetection): string {
  const unread = unreadOxlintConfig(detection);
  if (unread !== null) {
    return `${unread} — present but oxlint does not read this name (#3474)`;
  }
  return detection.oxlintConfig ?? "none";
}

/**
 * The full plan.
 *
 * Printed before anything is written in both modes, so the wording is what the
 * run is about to do, not what it has done. `--dry-run` differs only in stopping
 * afterwards.
 */
export function renderPlan(plan: InitPlan, dryRun: boolean): string {
  const verb = dryRun ? "would" : "will";
  const lines: string[] = [`${PREFIX} plan:`];
  for (const feature of plan.features) {
    lines.push(`  ${feature.id.padEnd(9)} ${feature.outcome.padEnd(10)} ${feature.detail}`);
  }
  for (const filename of plan.createdFiles) {
    lines.push(`${PREFIX} ${verb} create ${filename}`);
  }
  for (const filename of plan.updatedFiles) {
    lines.push(`${PREFIX} ${verb} update ${filename}`);
  }
  if (plan.addedScripts.length > 0) {
    lines.push(`${PREFIX} ${verb} add scripts: ${plan.addedScripts.join(", ")}`);
  }
  for (const command of plan.commands) {
    lines.push(`${PREFIX} ${verb} run: ${command.command} ${command.args.join(" ")}`);
  }
  if (plan.createdFiles.length + plan.updatedFiles.length + plan.commands.length === 0) {
    lines.push(`${PREFIX} nothing to do; the project is already configured`);
  }
  return `${lines.join("\n")}\n`;
}

/**
 * Snippets for anything `init` refused to edit.
 *
 * A blocked feature is deliberately loud. The alternative for the lint feature
 * would be writing an Oxlint config the project's lint command never reads,
 * which reports zero Vize diagnostics and exits 0 (#3389).
 */
export function renderBlocked(plan: InitPlan): string {
  const blocked = plan.features.filter((feature) => feature.outcome === "blocked");
  if (blocked.length === 0) {
    return "";
  }
  const lines: string[] = [];
  for (const feature of blocked) {
    lines.push(`${PREFIX} ${feature.id}: NOT configured — ${feature.detail}`);
    if (feature.snippet !== null) {
      lines.push("", indent(feature.snippet), "");
    }
  }
  return `${lines.join("\n")}\n`;
}

export function renderEditors(): string {
  const lines = [`${PREFIX} editor integrations shipped with Vize:`];
  for (const integration of EDITOR_INTEGRATIONS) {
    lines.push(`  ${integration}`);
  }
  return `${lines.join("\n")}\n`;
}

/**
 * Printed when the prompt ends without a confirmation.
 *
 * Covers both a declined confirmation and an input stream that closed
 * mid-prompt. Saying so is what keeps a closed stdin from looking like a
 * successful run that happened to change nothing.
 */
export function renderCancelled(): string {
  return `${PREFIX} cancelled; nothing was written.\n`;
}

export function renderNonInteractiveRefusal(): string {
  return (
    `${PREFIX} stdin is not a TTY, so init will not prompt.\n` +
    `${PREFIX} pass --yes with the features you want, for example:\n` +
    `${PREFIX}   vize init --yes --lint --vite --fmt --typecheck --editor\n` +
    `${PREFIX} or run with --dry-run to print the plan without writing.\n`
  );
}

function indent(source: string): string {
  return source
    .split("\n")
    .map((line) => (line === "" ? line : `    ${line}`))
    .join("\n")
    .trimEnd();
}
