"use strict";

const crypto = require("node:crypto");
const path = require("node:path");

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
// the file a language-service request names and its resolved source dependencies.
function createLazySfcScripts({
  ts,
  get,
  options,
  settings,
  sourceDocuments,
  snapshots,
  isSfc,
  getSnapshot,
  readFile,
  resolveImport,
  changed,
  projectVersion,
  scriptVersion,
  openFiles,
  project,
}) {
  const eager = new Set();
  const imports = new Map();
  let updating = false;
  // One walk per project snapshot: it promoted every SFC its files reach.
  // Script versions also cover hosts that keep one project version across edits.
  let walked;
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
      const version = projectVersion?.();
      const roots = [fileName, ...openFiles()];
      if (
        version !== undefined &&
        walked?.version === version &&
        roots.every((name) => {
          const key = path.normalize(name);
          return walked.files.has(key) && walked.files.get(key) === scriptVersion?.(name);
        })
      )
        return;
      updating = true;
      try {
        // References in an ordinary TS file also need the SFCs imported by
        // open entrypoints. Unrequested external SFC roots remain excluded.
        const pending = [...roots],
          seen = new Set(),
          files = new Map();
        let promoted = false,
          stale = false;
        while (pending.length) {
          const name = pending.pop(),
            key = path.normalize(name);
          // Global component declarations describe all project roots. Reading
          // one is not a request to generate every declared component.
          if (seen.has(key) || /\.d\.[cm]?ts$/i.test(name)) continue;
          seen.add(key);
          files.set(key, scriptVersion?.(name));
          let source;
          if (isSfc(name)) {
            if (mark(name)) {
              promoted = true;
              const cached = snapshots.get(name);
              stale ||= !!cached?.stubSnapshot && cached.configuration === undefined;
            }
            // The canonical projection owns which authored imports survive.
            // Template text and quoted import-like strings are not dependencies.
            source = get(name)?.projection?.text;
          } else {
            const snapshot = getSnapshot(name);
            source = snapshot?.getText(0, snapshot.getLength()) ?? readFile(name);
          }
          if (typeof source !== "string") continue;
          let scan = imports.get(key);
          if (scan?.source !== source) {
            scan = { source, files: ts.preProcessFile(source, true, true).importedFiles };
            if (scan.files.length)
              scan.file = ts.createSourceFile(
                name,
                source,
                ts.ScriptTarget.Latest,
                true,
                ts.ScriptKind.TSX,
              );
            imports.set(key, scan);
          }
          for (const imported of scan.files) {
            const resolved = resolveImport(imported, scan.file)?.resolvedFileName;
            if (resolved && /\.(?:[cm]?[jt]sx?|vue|art)$/i.test(resolved)) pending.push(resolved);
          }
        }
        // TypeScript skips all snapshot/version checks when the project version
        // is unchanged. Promoting a cached stub is a real program change even
        // when no authored source or tsserver project version changed.
        if (promoted) changed();
        if (stale) project?.updateGraph?.();
        for (const name of roots) files.set(path.normalize(name), scriptVersion?.(name));
        walked = version === undefined ? undefined : { version: projectVersion(), files };
      } finally {
        updating = false;
      }
    },
  };
}

module.exports = { createSourceDocuments, createLazySfcScripts };
