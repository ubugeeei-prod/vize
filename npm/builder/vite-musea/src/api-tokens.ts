/**
 * Musea gallery API token route handlers.
 *
 * Handles GET/POST/PUT/DELETE for /api/tokens endpoints:
 * - GET /tokens       -- list all resolved design tokens
 * - GET /tokens/usage -- token usage across art files
 * - POST /tokens      -- create a new token
 * - PUT /tokens       -- update an existing token
 * - DELETE /tokens    -- delete a token
 */

import type { ApiRoutesContext, SendJson, SendError, ReadBody } from "./api-routes/index.js";
import { HttpError, parseJsonBody, resolveInsideAny } from "./security.js";
import {
  parseTokens,
  buildTokenMap,
  resolveReferences,
  readRawTokenFile,
  writeRawTokenFile,
  setTokenAtPath,
  deleteTokenAtPath,
  validateSemanticReference,
  findDependentTokens,
  scanTokenUsage,
  type DesignToken,
  type TokenUsageMap,
} from "./style-dictionary.js";

function resolveTokensPath(ctx: ApiRoutesContext): string {
  const allowedRoots = [ctx.config.root, ...ctx.scanRoots];
  if (ctx.projectRoot) allowedRoots.push(ctx.projectRoot);
  return resolveInsideAny(allowedRoots, ctx.tokensPath!, "tokensPath");
}

function sendTokenMutationError(e: unknown, sendError: SendError): void {
  if (e instanceof HttpError) {
    sendError(e.message, e.status);
    return;
  }

  if (e instanceof Error && /(token path|token editing)/i.test(e.message)) {
    sendError(e.message, 400);
    return;
  }

  sendError(e instanceof Error ? e.message : String(e));
}

/** GET /api/tokens/usage */
export async function handleTokensUsage(ctx: ApiRoutesContext, sendJson: SendJson): Promise<void> {
  if (!ctx.tokensPath) {
    sendJson({});
    return;
  }

  try {
    const absoluteTokensPath = resolveTokensPath(ctx);
    const categories = await parseTokens(absoluteTokensPath);
    const tokenMap = buildTokenMap(categories);
    resolveReferences(categories, tokenMap);
    const resolvedTokenMap = buildTokenMap(categories);
    const usage: TokenUsageMap = scanTokenUsage(ctx.artFiles, resolvedTokenMap);
    sendJson(usage);
  } catch (e) {
    console.error("[musea] Failed to scan token usage:", e);
    sendJson({});
  }
}

/** GET /api/tokens */
export async function handleTokensGet(ctx: ApiRoutesContext, sendJson: SendJson): Promise<void> {
  if (!ctx.tokensPath) {
    sendJson({
      categories: [],
      tokenMap: {},
      meta: {
        filePath: "",
        tokenCount: 0,
        primitiveCount: 0,
        semanticCount: 0,
      },
    });
    return;
  }

  try {
    const absoluteTokensPath = resolveTokensPath(ctx);
    const categories = await parseTokens(absoluteTokensPath);
    const tokenMap = buildTokenMap(categories);
    resolveReferences(categories, tokenMap);
    const resolvedTokenMap = buildTokenMap(categories);
    let primitiveCount = 0;
    let semanticCount = 0;
    for (const token of Object.values(resolvedTokenMap)) {
      if (token.$tier === "semantic") semanticCount++;
      else primitiveCount++;
    }
    sendJson({
      categories,
      tokenMap: resolvedTokenMap,
      meta: {
        filePath: absoluteTokensPath,
        tokenCount: Object.keys(resolvedTokenMap).length,
        primitiveCount,
        semanticCount,
      },
    });
  } catch (e) {
    console.error("[musea] Failed to load tokens:", e);
    sendJson({ categories: [], tokenMap: {}, error: String(e) });
  }
}

/** POST /api/tokens (create) */
export async function handleTokensCreate(
  ctx: ApiRoutesContext,
  readBody: ReadBody,
  sendJson: SendJson,
  sendError: SendError,
): Promise<void> {
  if (!ctx.tokensPath) {
    sendError("No tokens path configured", 400);
    return;
  }

  const body = await readBody();
  try {
    const { path: dotPath, token } = parseJsonBody<{
      path: string;
      token: Omit<DesignToken, "$resolvedValue">;
    }>(body);
    if (!dotPath || !token || token.value === undefined) {
      sendError("Missing required fields: path, token.value", 400);
      return;
    }
    const absoluteTokensPath = resolveTokensPath(ctx);
    const rawData = await readRawTokenFile(absoluteTokensPath);

    const currentCategories = await parseTokens(absoluteTokensPath);
    const currentMap = buildTokenMap(currentCategories);
    if (currentMap[dotPath]) {
      sendError(`Token already exists at path "${dotPath}"`, 409);
      return;
    }

    if (token.$reference) {
      const validation = validateSemanticReference(currentMap, token.$reference, dotPath);
      if (!validation.valid) {
        sendError(validation.error!, 400);
        return;
      }
      token.value = `{${token.$reference}}`;
      token.$tier = "semantic";
    }

    setTokenAtPath(rawData, dotPath, token);
    await writeRawTokenFile(absoluteTokensPath, rawData);

    const categories = await parseTokens(absoluteTokensPath);
    const tokenMap = buildTokenMap(categories);
    resolveReferences(categories, tokenMap);
    const resolvedTokenMap = buildTokenMap(categories);
    sendJson({ categories, tokenMap: resolvedTokenMap }, 201);
  } catch (e) {
    sendTokenMutationError(e, sendError);
  }
}

/** PUT /api/tokens (update) */
export async function handleTokensUpdate(
  ctx: ApiRoutesContext,
  readBody: ReadBody,
  sendJson: SendJson,
  sendError: SendError,
): Promise<void> {
  if (!ctx.tokensPath) {
    sendError("No tokens path configured", 400);
    return;
  }

  const body = await readBody();
  try {
    const { path: dotPath, token } = parseJsonBody<{
      path: string;
      token: Omit<DesignToken, "$resolvedValue">;
    }>(body);
    if (!dotPath || !token || token.value === undefined) {
      sendError("Missing required fields: path, token.value", 400);
      return;
    }
    const absoluteTokensPath = resolveTokensPath(ctx);

    if (token.$reference) {
      const currentCategories = await parseTokens(absoluteTokensPath);
      const currentMap = buildTokenMap(currentCategories);
      const validation = validateSemanticReference(currentMap, token.$reference, dotPath);
      if (!validation.valid) {
        sendError(validation.error!, 400);
        return;
      }
      token.value = `{${token.$reference}}`;
      token.$tier = "semantic";
    }

    const rawData = await readRawTokenFile(absoluteTokensPath);
    setTokenAtPath(rawData, dotPath, token);
    await writeRawTokenFile(absoluteTokensPath, rawData);

    const categories = await parseTokens(absoluteTokensPath);
    const tokenMap = buildTokenMap(categories);
    resolveReferences(categories, tokenMap);
    const resolvedTokenMap = buildTokenMap(categories);
    sendJson({ categories, tokenMap: resolvedTokenMap });
  } catch (e) {
    sendTokenMutationError(e, sendError);
  }
}

/** DELETE /api/tokens */
export async function handleTokensDelete(
  ctx: ApiRoutesContext,
  readBody: ReadBody,
  sendJson: SendJson,
  sendError: SendError,
): Promise<void> {
  if (!ctx.tokensPath) {
    sendError("No tokens path configured", 400);
    return;
  }

  const body = await readBody();
  try {
    const { path: dotPath } = parseJsonBody<{ path: string }>(body);
    if (!dotPath) {
      sendError("Missing required field: path", 400);
      return;
    }
    const absoluteTokensPath = resolveTokensPath(ctx);

    const currentCategories = await parseTokens(absoluteTokensPath);
    const currentMap = buildTokenMap(currentCategories);
    const dependents = findDependentTokens(currentMap, dotPath);

    const rawData = await readRawTokenFile(absoluteTokensPath);
    const deleted = deleteTokenAtPath(rawData, dotPath);
    if (!deleted) {
      sendError(`Token not found at path "${dotPath}"`, 404);
      return;
    }
    await writeRawTokenFile(absoluteTokensPath, rawData);

    const categories = await parseTokens(absoluteTokensPath);
    const tokenMap = buildTokenMap(categories);
    resolveReferences(categories, tokenMap);
    const resolvedTokenMap = buildTokenMap(categories);
    sendJson({
      categories,
      tokenMap: resolvedTokenMap,
      dependentsWarning: dependents.length > 0 ? dependents : undefined,
    });
  } catch (e) {
    sendTokenMutationError(e, sendError);
  }
}
