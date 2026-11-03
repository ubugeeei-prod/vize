"use strict";

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

module.exports = { createSourceDocuments };
