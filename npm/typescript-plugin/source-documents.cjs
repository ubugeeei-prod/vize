"use strict";

const crypto = require("node:crypto");
const path = require("node:path");
const RELATIVE_SFC = /['"](\.[^'"\n]*\.(?:vue|art))['"]/g;

function createSourceDocuments(
  info,
  { getSnapshot, previousVersion, readFile, watchFile, isSfc },
  snapshots,
  changed,
) {
  const sourceWatchers = new Map();
  const get = (fileName) => {
    if (!isSfc(fileName)) return undefined;
    const cached = snapshots.get(fileName);
    const priorInfo = info.project?.projectService?.getScriptInfo?.(fileName);
    const open = priorInfo?.isScriptOpen?.();
    const versioned = !priorInfo?.hasMixedContent || open !== false || sourceWatchers.has(fileName);
    if (
      cached &&
      previousVersion &&
      versioned &&
      cached.hostVersion === previousVersion(fileName) &&
      cached.open === open
    )
      return cached;
    const originalSnapshot = getSnapshot(fileName);
    const scriptInfo = info.project?.projectService?.getScriptInfo?.(fileName);
    // tsserver deliberately exposes an empty snapshot for closed mixed files.
    // An open, unsaved empty file is authoritative and must remain empty.
    const closedMixed = scriptInfo?.hasMixedContent && scriptInfo.isScriptOpen?.() === false;
    const raw = closedMixed ? undefined : originalSnapshot;
    const source = raw?.getText(0, raw.getLength()) ?? readFile(fileName);
    if (typeof source !== "string") {
      snapshots.delete(fileName);
      return undefined;
    }
    if (scriptInfo?.hasMixedContent && !sourceWatchers.has(fileName) && watchFile) {
      sourceWatchers.set(
        fileName,
        watchFile(
          fileName,
          () => {
            if (info.project?.projectService?.getScriptInfo?.(fileName)?.isScriptOpen?.()) return;
            snapshots.delete(fileName);
            changed();
          },
          250,
        ),
      );
    }
    if (closedMixed && originalSnapshot?.getText(0, originalSnapshot.getLength()) !== source) {
      // Protocol navigation uses ScriptInfo's authored line map and snapshot.
      scriptInfo.reloadFromFile?.(fileName);
    }
    const value = cached?.source === source ? cached : { source };
    value.hostVersion = previousVersion?.(fileName);
    value.open = scriptInfo?.isScriptOpen?.();
    snapshots.set(fileName, value);
    return value;
  };
  return {
    get,
    dispose() {
      for (const watcher of sourceWatchers.values()) watcher.close();
      sourceWatchers.clear();
    },
  };
}

// tsserver asks for every external SFC while a project loads. Generate only
// the file a language-service request names, plus its direct relative imports.
function createLazySfcScripts({
  ts,
  get,
  options,
  settings,
  sourceDocuments,
  snapshots,
  isSfc,
  readFile,
  readSource,
  project,
}) {
  const eager = new Set();
  let updating = false;
  const configuration = () => {
    options();
    return settings().project.configuration;
  };
  const stub = (cached, current) => {
    if (!cached.stubSnapshot) cached.stubSnapshot = ts.ScriptSnapshot.fromString("export {};\n");
    if (cached.stubConfiguration !== current || cached.stubSource !== cached.source) {
      cached.stubVersion = `stub:${crypto.createHash("sha256").update(cached.source).update("\0").update(current).digest("hex")}`;
      cached.stubConfiguration = current;
      cached.stubSource = cached.source;
    }
    return { snapshot: cached.stubSnapshot, version: cached.stubVersion };
  };
  const prepared = (name) => {
    if (!isSfc(name)) return undefined;
    const cached = sourceDocuments.get(name);
    if (!cached) return undefined;
    const current = configuration();
    if (cached.configuration === current) return cached;
    if (!eager.has(path.normalize(name))) return stub(cached, current);
    return get(name);
  };
  const mark = (name) => {
    const key = path.normalize(name);
    if (eager.has(key)) return false;
    eager.add(key);
    return true;
  };
  return {
    snapshot: (name) => prepared(name)?.snapshot,
    version: (name) => prepared(name)?.version,
    ensure(fileName) {
      if (typeof fileName !== "string" || updating) return;
      const added = [];
      if (isSfc(fileName) && mark(fileName)) added.push(fileName);
      if (!/\.d\.ts$/i.test(fileName)) {
        const source = isSfc(fileName) ? readSource(fileName) : readFile(fileName);
        if (typeof source === "string") {
          for (const match of source.matchAll(RELATIVE_SFC)) {
            const resolved = path.resolve(path.dirname(fileName), match[1]);
            if (mark(resolved)) added.push(resolved);
          }
        }
      }
      const stale = added.some((name) => {
        const cached = snapshots.get(name);
        return cached?.stubSnapshot && cached.configuration === undefined;
      });
      if (!stale) return;
      updating = true;
      try {
        project?.updateGraph?.();
      } finally {
        updating = false;
      }
    },
  };
}

module.exports = { createSourceDocuments, createLazySfcScripts };
