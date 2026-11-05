"use strict";

// tsserver caches its extends resolution, including absent package candidates.
// Refresh only configured projects through public APIs; roots and references
// remain owned by tsserver. No extended-config cache enters this fresh parse.
function createProjectOptions(ts, info, originalOptions, readFile, fileExists) {
  const project = info.project;
  if (
    project?.projectKind !== ts.server?.ProjectKind.Configured ||
    typeof project.setCompilerOptions !== "function" ||
    typeof ts.getParsedCommandLineOfConfigFile !== "function"
  )
    return undefined;
  const server = info.serverHost || ts.sys;
  const controlled = new Set();
  let parsedOptions, applied;
  return {
    refresh(configFileName) {
      const parsed = ts.getParsedCommandLineOfConfigFile(configFileName, undefined, {
        ...server,
        readFile,
        fileExists,
        readDirectory: server.readDirectory.bind(server),
        getCurrentDirectory: project.getCurrentDirectory.bind(project),
        useCaseSensitiveFileNames: ts.sys.useCaseSensitiveFileNames,
        onUnRecoverableConfigFileDiagnostic() {},
      });
      if (!parsed) return;
      parsedOptions = parsed.options;
      applied = undefined;
      this.apply();
    },
    apply() {
      if (!parsedOptions) return;
      const original = originalOptions();
      if (original === applied) return;
      for (const name of Object.keys(parsedOptions))
        if (name !== "configFile") controlled.add(name);
      const changed = [...controlled].some(
        (name) => JSON.stringify(original[name]) !== JSON.stringify(parsedOptions[name]),
      );
      applied = original;
      if (!changed) return;
      const descriptors = Object.getOwnPropertyDescriptors(original);
      for (const name of controlled)
        if (!Object.hasOwn(parsedOptions, name)) delete descriptors[name];
      for (const [name, descriptor] of Object.entries(
        Object.getOwnPropertyDescriptors(parsedOptions),
      )) {
        // The server's SourceFile carries its root specs and reload state.
        // Preserve its exact non-enumerable descriptor and object identity.
        if (name === "configFile") continue;
        controlled.add(name);
        const prior = descriptors[name];
        descriptors[name] =
          prior && "value" in prior ? { ...prior, value: descriptor.value } : descriptor;
      }
      const options = Object.create(Object.getPrototypeOf(original), descriptors);
      applied = options;
      // Besides changing the option values, this public operation invalidates
      // the server's module-resolution cache when the selected options require it.
      // reloadProjects may subsequently restore its stale cached options. Reapply
      // this parsed snapshot to a changed host object without parsing it again.
      project.setCompilerOptions(options);
    },
  };
}

module.exports = { createProjectOptions };
