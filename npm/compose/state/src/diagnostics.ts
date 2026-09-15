import type {
  AnyStateModelDefinition,
  StateDiagnosticsManifest,
  StateDiagnosticsOptions,
  StateManifest,
} from "./types.ts";
import { validateModel } from "./validation.ts";

/** Emit metadata for generators, documentation, and devtools. */
export function createStateManifest<const Models extends readonly AnyStateModelDefinition[]>(
  models: Models,
): StateManifest {
  for (const model of models) validateModel(model);
  return {
    schemaVersion: 1,
    models: models.map((model) => ({
      key: model.key,
      source: model.source,
      version: model.version,
      meta: model.meta ?? {},
      persistence: model.persistence != null,
    })),
  };
}

/** Emit devtools diagnostics while keeping production payloads empty. */
export function createStateDiagnosticsManifest<
  const Models extends readonly AnyStateModelDefinition[],
>(models: Models, options: StateDiagnosticsOptions = {}): StateDiagnosticsManifest {
  if (options.production === true) {
    return { schemaVersion: 1, production: true, models: [] };
  }
  for (const model of models) validateModel(model);
  return {
    schemaVersion: 1,
    production: false,
    models: models.map((model) => ({
      key: model.key,
      source: model.source,
      version: model.version,
      commands: Object.keys(model.commands ?? {}),
      persistence: model.persistence != null,
      meta: model.meta ?? {},
    })),
  };
}
