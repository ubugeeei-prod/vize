/**
 * Reporting for the scale benchmark.
 *
 * Two tables, both borrowed from `rolldown/benchmarks` `bench.mjs`:
 *
 * 1. Time with spread, next to module count and output sizes. Reported per
 *    scale, plus a per-module cost column so a non-linear trend is readable
 *    without doing the division by hand.
 * 2. An output-divergence section. The reference does not have this because it
 *    compares bundlers, whose outputs legitimately differ. Two Vue plugins on
 *    the same Vite should emit the same module count, and comparable CSS and
 *    sourcemap volume; when they do not, the faster one is suspect.
 */

export function formatBytes(bytes) {
  if (bytes === 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

export function formatMs(ms) {
  return ms >= 1000 ? `${(ms / 1000).toFixed(2)} s` : `${ms.toFixed(0)} ms`;
}

function row(cells) {
  return `| ${cells.join(" | ")} |`;
}

function divider(count) {
  return `|${" --- |".repeat(count)}`;
}

export function printScaleTable(results, tools) {
  console.log("");
  console.log("## Build time and output");
  console.log("");
  console.log(
    row([
      "Scale (SFCs)",
      "Tool",
      "Modules",
      "Time (median [min..max])",
      "µs / module",
      "JS",
      "CSS",
      "Sourcemaps",
    ]),
  );
  console.log(divider(8));

  for (const result of results) {
    for (const tool of tools) {
      const data = result.tools[tool];
      if (!data) continue;
      const perModuleUs =
        data.output.moduleCount > 0
          ? ((data.timing.wallMedianMs * 1000) / data.output.moduleCount).toFixed(0)
          : "n/a";
      console.log(
        row([
          String(result.componentCount),
          tool,
          String(data.output.moduleCount),
          `${formatMs(data.timing.wallMedianMs)} [${formatMs(data.timing.wallMinMs)}..${formatMs(
            data.timing.wallMaxMs,
          )}]`,
          perModuleUs,
          formatBytes(data.output.jsBytes),
          formatBytes(data.output.cssBytes),
          formatBytes(data.output.mapBytes),
        ]),
      );
    }
  }
  console.log("");
}

function pushDivergence(list, componentCount, kind, detail) {
  list.push({ componentCount, kind, detail });
}

/**
 * Print the post-build correctness checks from `tools/benchmarks/scripts/scale/verify.mjs`.
 *
 * These are per-tool, not comparative: a wrong sourcemap or a dropped scope id
 * is a defect whether or not the other plugin has the same one.
 */
export function printVerification(results, tools) {
  const failures = [];

  console.log("## Sourcemap and scoped-style verification");
  console.log("");
  console.log(
    row([
      "Scale (SFCs)",
      "Tool",
      "Tokens traced",
      "Sourcemap failures",
      "Scope ids (JS/CSS)",
      "Scope id mismatch",
    ]),
  );
  console.log(divider(6));

  for (const result of results) {
    for (const tool of tools) {
      const data = result.tools[tool];
      if (!data) continue;
      const { checked, failures: mapFailures } = data.sourcemaps;
      const { jsScopeIdCount, cssScopeIdCount, jsOnly, cssOnly } = data.scopedStyles;
      const mismatch = jsOnly.length + cssOnly.length;
      console.log(
        row([
          String(result.componentCount),
          tool,
          String(checked),
          String(mapFailures.length),
          `${jsScopeIdCount}/${cssScopeIdCount}`,
          mismatch === 0 ? "0" : `${jsOnly.length} js-only, ${cssOnly.length} css-only`,
        ]),
      );

      for (const failure of mapFailures) {
        pushDivergence(
          failures,
          result.componentCount,
          `${tool}:sourcemap`,
          `${failure.token}: ${failure.status}` +
            (failure.source ? ` (mapped to ${failure.source})` : "") +
            (failure.originalLine ? ` original line: ${failure.originalLine}` : ""),
        );
      }
      if (mismatch > 0) {
        pushDivergence(
          failures,
          result.componentCount,
          `${tool}:scope-id`,
          `${jsOnly.length} scope id(s) only in JS (first ${jsOnly[0] ?? "-"}),` +
            ` ${cssOnly.length} only in CSS (first ${cssOnly[0] ?? "-"})`,
        );
      }
    }
  }
  console.log("");

  if (failures.length > 0) {
    for (const failure of failures) {
      console.log(`- ${failure.componentCount} SFCs — ${failure.kind}: ${failure.detail}`);
    }
    console.log("");
  }

  return failures;
}

/**
 * Reduce a rollup module id to the source file it came from.
 *
 * Raw module counts are not comparable between the two plugins and must not be
 * asserted on: `@vitejs/plugin-vue` splits one SFC into three graph nodes
 * (`X.vue`, `X.vue?vue&type=script...`, `X.vue?vue&type=style...`), while Vize
 * emits two (`X.vue.ts?vue&vize`, `X.vue?vue=&type=style...`). Both are correct.
 * What must match is the set of *source files* that reached the bundle — that is
 * what "did a module get silently dropped" actually means.
 *
 * Returns `null` for plugin-internal virtual helpers (`plugin-vue:export-helper`,
 * ` vite/modulepreload-polyfill.js`), which are legitimately plugin-specific.
 */
export function normalizeModuleId(id) {
  const withoutNullByte = id.startsWith("\0") ? id.slice(1) : id;
  const withoutQuery = withoutNullByte.replace(/\?.*$/, "");
  if (!withoutQuery.startsWith("/")) {
    return null;
  }
  return withoutQuery.endsWith(".vue.ts") ? withoutQuery.slice(0, -3) : withoutQuery;
}

function partitionModules(modules) {
  const vueSources = new Set();
  const fileModules = new Set();
  const virtualModules = new Set();

  for (const id of modules) {
    const normalized = normalizeModuleId(id);
    if (normalized === null) {
      virtualModules.add(id);
    } else if (normalized.endsWith(".vue")) {
      vueSources.add(normalized);
    } else {
      fileModules.add(normalized);
    }
  }

  return { vueSources, fileModules, virtualModules };
}

function missingFrom(expected, actual) {
  return [...expected].filter((value) => !actual.has(value));
}

/**
 * Compare the tools' outputs at each scale.
 *
 * CSS and sourcemap volume is compared as a ratio, because the two plugins emit
 * CSS through different paths (Vize can extract component CSS into one asset)
 * and byte-identical output is not expected; an order-of-magnitude gap is.
 */
export function reportDivergence(results) {
  const divergences = [];

  for (const result of results) {
    const vize = result.tools.vize;
    const vue = result.tools.vue;
    if (!vize || !vue) continue;

    const vizeGraph = partitionModules(vize.output.modules);
    const vueGraph = partitionModules(vue.output.modules);

    const droppedSfcs = missingFrom(vueGraph.vueSources, vizeGraph.vueSources);
    const extraSfcs = missingFrom(vizeGraph.vueSources, vueGraph.vueSources);
    if (droppedSfcs.length > 0 || extraSfcs.length > 0) {
      pushDivergence(
        divergences,
        result.componentCount,
        "sfc-set",
        `vize is missing ${droppedSfcs.length} SFC(s) present in the vue build` +
          ` and has ${extraSfcs.length} not present there` +
          (droppedSfcs[0] ? `; first missing: ${droppedSfcs[0]}` : ""),
      );
    }

    const droppedFiles = missingFrom(vueGraph.fileModules, vizeGraph.fileModules);
    const extraFiles = missingFrom(vizeGraph.fileModules, vueGraph.fileModules);
    if (droppedFiles.length > 0 || extraFiles.length > 0) {
      pushDivergence(
        divergences,
        result.componentCount,
        "non-sfc-module-set",
        `vize is missing ${droppedFiles.length} non-SFC module(s) and has ${extraFiles.length} extra` +
          (droppedFiles[0] ? `; first missing: ${droppedFiles[0]}` : "") +
          (extraFiles[0] ? `; first extra: ${extraFiles[0]}` : ""),
      );
    }

    if (vize.output.mapBytes === 0 && vue.output.mapBytes > 0) {
      pushDivergence(
        divergences,
        result.componentCount,
        "sourcemaps-missing",
        `vize emitted no sourcemaps, vue emitted ${formatBytes(vue.output.mapBytes)}`,
      );
    }

    if (vue.output.cssBytes > 0 && vize.output.cssBytes < vue.output.cssBytes / 2) {
      pushDivergence(
        divergences,
        result.componentCount,
        "css-shortfall",
        `vize ${formatBytes(vize.output.cssBytes)} vs vue ${formatBytes(vue.output.cssBytes)}`,
      );
    }

    if (vue.output.jsBytes > 0 && vize.output.jsBytes < vue.output.jsBytes / 2) {
      pushDivergence(
        divergences,
        result.componentCount,
        "js-shortfall",
        `vize ${formatBytes(vize.output.jsBytes)} vs vue ${formatBytes(vue.output.jsBytes)}`,
      );
    }
  }

  console.log("## Output divergence (vize vs vue)");
  console.log("");
  if (divergences.length === 0) {
    console.log(
      "None. Both builds contain the same SFC and non-SFC source files, and comparable CSS/JS/sourcemap volume.",
    );
  } else {
    for (const divergence of divergences) {
      console.log(`- ${divergence.componentCount} SFCs — ${divergence.kind}: ${divergence.detail}`);
    }
  }
  console.log("");

  return divergences;
}
