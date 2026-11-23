function mutateFakeJsonState(statePath, update) {
  const stateFs = require("node:fs");
  const lock = statePath + ".lock";
  const deadline = Date.now() + 10000;
  for (;;) {
    try {
      stateFs.mkdirSync(lock);
      break;
    } catch (error) {
      if (error.code !== "EEXIST" || Date.now() >= deadline) throw error;
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 1);
    }
  }
  const temporary = statePath + "." + process.pid + ".tmp";
  try {
    const state = JSON.parse(stateFs.readFileSync(statePath, "utf8"));
    update(state);
    stateFs.writeFileSync(temporary, JSON.stringify(state));
    stateFs.renameSync(temporary, statePath);
    return state;
  } finally {
    stateFs.rmSync(temporary, { force: true });
    stateFs.rmdirSync(lock);
  }
}

module.exports = mutateFakeJsonState;
