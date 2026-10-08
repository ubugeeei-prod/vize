/** Schema only. No publication/install identity is assigned by this file. */
export interface VizePublicRegistryInstallAuthority {
  schema: "vize-public-registry-install-v1";
  version: string;
  checkedAt: string;
  success: true;
  source: { C: string; H: string; tag: string; R: string };
  installRoot: string;
  packageLockPath: string;
  packageLockSha256: string;
  payloadManifestPath: string;
  payloadManifestSha256: string;
  registry: Array<{
    name: string;
    version: string;
    resolved: string;
    integrity: string;
    provenanceSourceH: string;
    provenanceR: string;
  }>;
  node: { path: string; version: string; sha256: string };
  cli: {
    binPath: string;
    binSha256: string;
    distCliPath: string;
    distCliSha256: string;
    versionStdout: string;
    versionStderr: string;
    versionExitCode: number;
  };
  native: {
    packageName: string;
    version: string;
    path: string;
    sha256: string;
    actualLoadedPath: string;
    journalPath: string;
    journalSha256: string;
    successfulReturnObserved: true;
  };
  bundledCorsa: {
    packageName: string;
    version: string;
    declaredVersion: string;
    path: string;
    sha256: string;
    resolved: string;
    integrity: string;
    allInstalledFileBytesEqualPublicArchive: true;
  };
  custodyHook: {
    path: string;
    sha256: string;
    configurationEnvironment: "VIZE_PUBLIC_NATIVE_CUSTODY";
  };
  sourceOverrides: {
    NODE_OPTIONS: "";
    VIZE_PREFER_WORKSPACE_BINDING: "";
    NAPI_RS_NATIVE_LIBRARY_PATH: "";
    NAPI_RS_FORCE_WASI: "";
    VIZE_ALLOW_NATIVE_VERSION_MISMATCH: "";
    CORSA_PATH: "";
    CORSA_EXECUTABLE: "";
    TSGO_PATH: "";
    TSGO_EXECUTABLE: "";
  };
}

/** The producer snapshot is reviewed tool custody, independently of release H. */
export interface VizePublicRegistryInstallCollectorReceipt extends VizePublicRegistryInstallAuthority {
  collectorAuthority: {
    schema: "vize-public-install-collector-v1";
    sha256: string;
    files: Array<{ path: string; sha256: string }>;
    scope: string;
  };
  sourceManifestPayloads: Array<{ path: string; sha256: string }>;
}
