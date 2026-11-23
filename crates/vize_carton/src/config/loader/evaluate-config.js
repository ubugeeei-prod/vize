import { dirname } from "node:path";

const configPath = process.argv[1];
const stopRecording =
  process.env.VIZE_RECORD_CONFIG_INPUTS === "1" ? recordConfigInputs() : undefined;
const observing =
  process.argv[2] === "--observe-config-inputs" || process.argv[3] === "--observe-config-inputs";
// Existing wrappers retain their argv shape; the session adds one private path before the marker.
enableConfigCompileCache(
  process.argv[3] === "--observe-config-inputs" ? process.argv[2] : undefined,
);
const acknowledgements = observing ? process.stdin[Symbol.asyncIterator]() : undefined;
const beforeRead = observing
  ? async (paths) => {
      const payload = Buffer.from(JSON.stringify(paths));
      if (payload.length > 65536) throw new Error("Config observation frame exceeds 64 KiB");
      const frame = Buffer.concat([Buffer.from(`B${payload.length}\n`), payload]);
      await new Promise((resolve, reject) =>
        process.stdout.write(frame, (error) => (error ? reject(error) : resolve())),
      );
      // Only a pending acknowledgement keeps the process alive, so a config that
      // never settles exits here exactly as it does without observation.
      process.stdin.ref?.();
      const { value: ack, done } = await acknowledgements.next();
      process.stdin.unref?.();
      if (done || ack.length !== 1 || ack[0] !== 6) {
        throw new Error("Config observation acknowledgement ended");
      }
    }
  : undefined;
console.log = (...args) => console.error(...args);
let attemptedDependencies = [configPath];
let attemptedComplete = false;
let evaluated;
let failureArtifacts = [];
let configTimings;
try {
  const loaded = await importConfigModuleWithDependencies(
    configPath,
    undefined,
    undefined,
    async (module) => {
      const exported = module.default ?? module;
      return process.env.VIZE_EXPLICIT_NATIVE_CARRIER === "1"
        ? await resolveExplicitSerializedConfigExport(exported, undefined, dirname(configPath))
        : process.env.VIZE_EXPLICIT_VITE_CONFIG === "1"
          ? await resolveExplicitViteConfigExport(exported, undefined, dirname(configPath))
          : isViteConfigFile(configPath)
            ? await resolveViteConfigExport(exported, undefined, dirname(configPath))
            : typeof exported === "function"
              ? await exported({ mode: "development", command: "serve" })
              : exported;
    },
    (paths, complete) => {
      attemptedDependencies = paths;
      attemptedComplete = complete;
    },
    beforeRead,
  );
  const { value: config, dependencies } = loaded;
  configTimings = configModuleEvaluationTimings(loaded);
  evaluated = {
    config: config ?? {},
    dependencies,
    watchDependencies: attemptedDependencies,
    watchDependenciesComplete: true,
  };
} catch (error) {
  if (observing) failureArtifacts = configModuleFailureArtifacts(error);
  configTimings = configModuleEvaluationTimings(error);
  throw error;
} finally {
  if (observing) process.stdout.write("R\n");
  const failure = {
    dependencies: attemptedDependencies,
    watchDependenciesComplete: attemptedComplete,
  };
  if (failureArtifacts.length) failure.failureArtifacts = failureArtifacts;
  const result = evaluated ?? failure;
  if (stopRecording) result.inputs = stopRecording();
  if (configTimings) result.configEvaluationTimings = configTimings;
  // One line; the host stops reading at its newline.
  await new Promise((resolve) => process.stdout.write(JSON.stringify(result) + "\n", resolve));
}
// The result is out. Timers, servers or promises the config left behind must
// not keep this process, or the host waiting on it, alive.
process.exit();
