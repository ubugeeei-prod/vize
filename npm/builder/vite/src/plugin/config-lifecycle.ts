import type { ResolvedConfig, ViteDevServer } from "vite";

import { vizeConfigStore } from "../config.ts";
import {
  registerResolvedVizeConfig,
  unregisterResolvedVizeConfig,
} from "../internal/config-bridge.ts";
import type { ResolvedVizeConfig } from "../types.ts";

interface PluginEnvironmentContext {
  environment?: { getTopLevelConfig?: () => ResolvedConfig };
}

interface CompatibilityRegistration {
  config: ResolvedVizeConfig;
  root: string;
  token: symbol;
}

const compatibilityOwners = new Map<string, symbol>();
const compatibilityRegistrations = new WeakMap<ResolvedConfig, CompatibilityRegistration>();

function releaseCompatibilityRegistration(resolvedConfig: ResolvedConfig): void {
  const registration = compatibilityRegistrations.get(resolvedConfig);
  compatibilityRegistrations.delete(resolvedConfig);
  if (!registration || compatibilityOwners.get(registration.root) !== registration.token) {
    return;
  }

  compatibilityOwners.delete(registration.root);
  if (vizeConfigStore.get(registration.root) === registration.config) {
    vizeConfigStore.delete(registration.root);
  }
}

function unregisterConfig(resolvedConfig: ResolvedConfig): void {
  unregisterResolvedVizeConfig(resolvedConfig);
  releaseCompatibilityRegistration(resolvedConfig);
}

export async function register(
  resolvedConfig: ResolvedConfig,
  root: string,
  sharedConfigPromise: Promise<ResolvedVizeConfig | null>,
): Promise<ResolvedVizeConfig | null> {
  releaseCompatibilityRegistration(resolvedConfig);
  const token = Symbol(root);
  compatibilityOwners.set(root, token);
  vizeConfigStore.delete(root);
  registerResolvedVizeConfig(resolvedConfig, sharedConfigPromise);

  let sharedConfig: ResolvedVizeConfig | null;
  try {
    sharedConfig = await sharedConfigPromise;
  } catch (error) {
    unregisterResolvedVizeConfig(resolvedConfig);
    if (compatibilityOwners.get(root) === token) {
      compatibilityOwners.delete(root);
      vizeConfigStore.delete(root);
    }
    throw error;
  }

  if (compatibilityOwners.get(root) !== token) {
    return sharedConfig;
  }
  if (sharedConfig) {
    compatibilityRegistrations.set(resolvedConfig, { config: sharedConfig, root, token });
    vizeConfigStore.set(root, sharedConfig);
  } else {
    compatibilityOwners.delete(root);
    vizeConfigStore.delete(root);
  }

  return sharedConfig;
}

export function configureServerCleanup(devServer: ViteDevServer): void {
  const resolvedConfig = devServer.config;
  let unregistered = false;
  const unregister = () => {
    if (unregistered) return;
    unregistered = true;
    devServer.httpServer?.off("close", unregister);
    devServer.watcher.off("close", unregister);
    unregisterConfig(resolvedConfig);
  };
  devServer.httpServer?.once("close", unregister);
  devServer.watcher.once("close", unregister);
}

export function unregisterBuild(context: unknown): void {
  const environment = (context as PluginEnvironmentContext).environment;
  const resolvedConfig = environment?.getTopLevelConfig?.();
  if (resolvedConfig) {
    unregisterConfig(resolvedConfig);
  }
}
