getReleaseServerTarget = () => ({ targetName: "native", archiveName: "native.tar.gz" });
inspectCachedReleaseServer = () => operation("cached", cachedCandidate);
createReleaseDownloadUrl = () => "https://github.com/release/native.tar.gz";
verifyArchiveChecksum = () => operation("verify");
inspectServerCandidate = () => operation("inspect", candidate);
updateStatusBar = (status) => operations.push(status);
return {
  resolve: () =>
    resolveReleaseServerCandidate({ globalStorageUri: { fsPath: "/storage" } }, "vize", "1.2.3", 1),
  disable: () => {
    ++state.configurationSyncGeneration;
    operations.push("disabled");
  },
};
