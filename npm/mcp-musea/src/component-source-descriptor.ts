import fs from "node:fs";
import path from "node:path";

import type { ArtInfo, ServerContext } from "./types.js";
import { isVueSourcePath } from "./vue-source-path.js";

export interface ComponentSourceDescriptor {
  reference?: string;
  absolutePath?: string;
  path?: string;
  exists: boolean;
  error?: string;
}

interface ComponentSourceReference {
  info: Pick<ArtInfo, "component">;
  absolutePath: string;
}

function realpathNearest(targetPath: string): string {
  let current = path.resolve(targetPath);
  const missingParts: string[] = [];

  while (true) {
    try {
      const real = fs.realpathSync.native(current);
      return missingParts.length > 0 ? path.join(real, ...missingParts.reverse()) : real;
    } catch {
      const parent = path.dirname(current);
      if (parent === current) {
        return path.resolve(targetPath);
      }
      missingParts.push(path.basename(current));
      current = parent;
    }
  }
}

function isProjectPath(projectRoot: string, candidatePath: string): boolean {
  const root = realpathNearest(projectRoot);
  const candidate = realpathNearest(candidatePath);
  const relativePath = path.relative(root, candidate);
  return relativePath === "" || (!relativePath.startsWith("..") && !path.isAbsolute(relativePath));
}

function toProjectPath(projectRoot: string, absolutePath: string): string {
  const root = realpathNearest(projectRoot);
  const resolved = realpathNearest(absolutePath);
  const relativePath = path.relative(root, resolved);
  return isProjectPath(root, resolved) ? relativePath || "." : resolved;
}

export function resolveComponentSourcePath(
  artAbsolutePath: string,
  componentReference?: string,
): string | null {
  if (!componentReference) {
    return null;
  }

  if (path.isAbsolute(componentReference)) {
    return componentReference;
  }

  return path.resolve(path.dirname(artAbsolutePath), componentReference);
}

export async function getComponentSourceDescriptor(
  ctx: ServerContext,
  resolved: ComponentSourceReference,
): Promise<ComponentSourceDescriptor> {
  const componentPath = resolveComponentSourcePath(resolved.absolutePath, resolved.info.component);
  if (!componentPath) {
    return {
      reference: resolved.info.component,
      exists: false,
      error: "This art file does not declare a component source.",
    };
  }

  if (!isProjectPath(ctx.projectRoot, componentPath)) {
    return {
      reference: resolved.info.component,
      exists: false,
      error: "Component source is outside the project root.",
    };
  }

  if (!isVueSourcePath(componentPath)) {
    return {
      reference: resolved.info.component,
      exists: false,
      error: "Component source must be a .vue file.",
    };
  }

  try {
    await fs.promises.access(componentPath, fs.constants.R_OK);
    return {
      reference: resolved.info.component,
      absolutePath: componentPath,
      path: toProjectPath(ctx.projectRoot, componentPath),
      exists: true,
    };
  } catch {
    return {
      reference: resolved.info.component,
      absolutePath: componentPath,
      path: toProjectPath(ctx.projectRoot, componentPath),
      exists: false,
      error: `Component source not found: ${toProjectPath(ctx.projectRoot, componentPath)}`,
    };
  }
}
