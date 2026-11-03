// P4-16 worker arm: the plugin loaded in a worker thread, one batch per
// message round trip (the alternative the spike measured against sync napi).
import { parentPort, workerData } from "node:worker_threads";

const { default: plugin } = await import(`./${workerData}`);

parentPort.on("message", (json) => {
  parentPort.postMessage(json === "{}" ? "[]" : plugin.run(json));
});
