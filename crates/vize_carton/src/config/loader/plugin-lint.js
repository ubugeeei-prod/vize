// Run admitted plugin lint jobs. The native host sends every job in one
// request, so callbacks cross the process boundary once per lint run; each
// rule then runs once per document over host-owned, immutable records.
import { VizePluginPlanError, deepFreeze } from "./plugin-canonical.js";
import { callbackKey } from "./plugin-plan.js";

const LINT_CAPABILITIES = ["lint.plugin-v1", "lint.rule"];

/** Execute `request.jobs` against the plan this process resolved itself. */
export function runPluginLintJobs(resolved, request) {
  if (resolved === undefined)
    throw new VizePluginPlanError("plan-stale", "plan", "vite.config.* no longer lists plugins");
  if (request?.digest !== resolved.digest)
    throw new VizePluginPlanError(
      "plan-stale",
      "plan",
      `the lint request was planned for ${request?.digest}, but vite.config.* now resolves to ${resolved.digest}`,
    );
  return { digest: resolved.digest, jobs: request.jobs.map((job) => runJob(resolved, job)) };
}

function runJob(resolved, job) {
  const plugin = resolved.plan.plugins.find((candidate) => candidate.name === job.plugin);
  const capability = plugin?.capabilities.find(
    (candidate) => candidate.id === job.capability && candidate.key === job.key,
  );
  if (!capability || !LINT_CAPABILITIES.includes(capability.id))
    throw new VizePluginPlanError(
      "plan-stale",
      `${job.plugin} ${job.capability} ${job.key}`,
      "is not a lint capability of the resolved plan",
    );
  const run = resolved.callbacks.get(
    callbackKey(plugin.name, capability.id, capability.key, "run"),
  );
  const where = `${plugin.name} ${capability.key}`;
  const started = process.hrtime.bigint();
  const documents = job.documents.map(({ file, batch }) => {
    try {
      return {
        file,
        // v1 plugins keep their exact string-in, string-out packets.
        reports:
          capability.id === "lint.plugin-v1"
            ? run(batch)
            : runRule(run, plugin, capability.data, JSON.parse(batch)),
      };
    } catch (error) {
      if (error instanceof VizePluginPlanError) throw error;
      throw new VizePluginPlanError(
        "rule-failed",
        where,
        `${file}: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  });
  return {
    capability: capability.id,
    documents,
    jsNs: Number(process.hrtime.bigint() - started),
    key: capability.key,
    plugin: plugin.name,
  };
}

function runRule(run, plugin, rule, batch) {
  deepFreeze(batch);
  const where = `${plugin.name}/${rule.id}`;
  const byId = new Map(batch.nodes.map((node) => [node.id, node]));
  const reports = [];
  const context = Object.freeze({
    file: batch.file,
    nodes: batch.nodes,
    options: plugin.options,
    facts(name) {
      if (!rule.demands.includes(name))
        throw new VizePluginPlanError(
          "report-invalid",
          where,
          `fact group \`${name}\` was not demanded`,
        );
      return new Map(batch.facts[name]);
    },
    *ancestors(node, kind) {
      if (!rule.visit.includes(kind))
        throw new VizePluginPlanError(
          "report-invalid",
          where,
          `ancestors of kind ${kind} need ${kind} in visit`,
        );
      for (let id = batch.parents[node.id]; id !== undefined && id !== -1; id = batch.parents[id]) {
        const found = byId.get(id);
        if (found && found.kind === kind) yield found;
      }
    },
    report(report) {
      const node = report?.node;
      if (!node || byId.get(node.id) !== node)
        throw new VizePluginPlanError("report-invalid", where, "a report needs a visited node");
      if (!Object.hasOwn(rule.messages, report.messageId))
        throw new VizePluginPlanError(
          "report-invalid",
          where,
          `undeclared message id \`${report.messageId}\``,
        );
      const data = report.data ?? undefined;
      if (
        data !== undefined &&
        (data === null ||
          typeof data !== "object" ||
          Object.values(data).some(
            (value) => !["string", "number", "boolean"].includes(typeof value),
          ))
      )
        throw new VizePluginPlanError(
          "report-invalid",
          where,
          "report data values must be strings, numbers or booleans",
        );
      if (report.fix !== undefined && typeof report.fix !== "string")
        throw new VizePluginPlanError(
          "report-invalid",
          where,
          "a fix is replacement text for the reported node",
        );
      if (report.fix !== undefined && rule.fixable === "none")
        throw new VizePluginPlanError(
          "report-invalid",
          where,
          'fixes need meta.fixable "safe" or "unsafe"',
        );
      reports.push({
        ...(data === undefined ? {} : { data: { ...data } }),
        ...(report.fix === undefined ? {} : { fix: report.fix }),
        messageId: report.messageId,
        node: node.id,
      });
    },
  });
  const returned = run(context);
  if (returned !== undefined)
    throw new VizePluginPlanError(
      "report-invalid",
      where,
      "a rule returns nothing; report through context.report",
    );
  return reports;
}
