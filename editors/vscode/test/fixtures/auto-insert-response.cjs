const fs = require("node:fs");

exports.sendResponse = function sendResponse(message, result, send, appendLog, logPath) {
  const autoInsert = message.method === "volar/client/autoInsert";
  function respond() {
    if (autoInsert && fs.existsSync(`${logPath}.hold-auto-insert`)) {
      setTimeout(respond, 10);
      return;
    }
    send({ id: message.id, jsonrpc: "2.0", result });
    if (autoInsert) appendLog({ event: "auto-insert-response", id: message.id });
  }
  respond();
};
