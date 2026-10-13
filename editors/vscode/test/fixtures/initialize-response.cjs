const fs = require("node:fs");

exports.sendInitializeResponse = function sendInitializeResponse(
  id,
  capabilities,
  version,
  send,
  appendLog,
) {
  const response = {
    id,
    jsonrpc: "2.0",
    result: {
      capabilities,
      serverInfo: {
        name: "fake-vize",
        version,
      },
    },
  };
  const gate = process.env.VIZE_TEST_SERVER_INITIALIZE_GATE;
  if (gate && !fs.existsSync(gate)) {
    appendLog({ event: "initialize-gated", gate });
    const timer = setInterval(() => {
      if (fs.existsSync(gate)) {
        clearInterval(timer);
        send(response);
      }
    }, 10);
  } else {
    send(response);
  }
};
