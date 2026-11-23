inspectServerCandidate = async (candidate) => {
  operations.push(["probe", candidate.path]);
  return { ...candidate, version: "1.2.3" };
};
return { inspect: (file, probe) => inspectCachedReleaseServer(file, "1.2.3", probe) };
