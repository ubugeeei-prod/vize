/** Scope Loader — applies scoped CSS transformation after preprocessors, before css-loader. No-op for non-scoped blocks. */

import type { LoaderContext } from "@rspack/core";
import * as native from "@vizejs/native";
import { stripCssCommentsForScoped } from "../shared/utils.ts";

const { compileCss } = native;

export interface VizeScopeLoaderOptions {
  // no options needed — scope metadata is extracted from the query string
}

export default function vizeScopeLoader(
  this: LoaderContext<VizeScopeLoaderOptions>,
  source: string,
): void {
  const callback = this.async();
  const { resourceQuery, resourcePath } = this;

  if (!resourceQuery) {
    callback(null, source);
    return;
  }

  const params = new URLSearchParams(resourceQuery.slice(1));
  const scoped = params.get("scoped");

  // Not scoped — pass through
  if (!scoped) {
    callback(null, source);
    return;
  }

  const fullScopeId = `data-v-${scoped}`;
  const sanitizedCss = stripCssCommentsForScoped(source);

  const result = compileCss(sanitizedCss, {
    filename: resourcePath,
    scoped: true,
    scopeId: fullScopeId,
  });

  for (const error of result.errors) {
    this.emitError(new Error(`[vize] ${error}`));
  }
  for (const warning of result.warnings) {
    this.emitWarning(new Error(`[vize] ${warning}`));
  }

  callback(null, result.code);
}
