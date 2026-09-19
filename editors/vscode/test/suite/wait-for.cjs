const { AssertionError } = require("node:assert");

// Bound the provider request itself: VS Code commands can remain pending when
// the server loses a response. A deadline checked only between polls hangs too.
async function waitFor(produce, predicate, label, timeoutMs) {
  let value;
  let timer;
  const deadline = new Promise((_, reject) => {
    timer = setTimeout(() => {
      reject(
        new AssertionError({
          message: `${label} did not happen within ${timeoutMs} ms`,
          actual: value,
        }),
      );
    }, timeoutMs);
  });

  try {
    for (;;) {
      value = await Promise.race([Promise.resolve().then(produce), deadline]);
      if (predicate(value)) return value;
      await Promise.race([new Promise((resolve) => setTimeout(resolve, 100)), deadline]);
    }
  } finally {
    clearTimeout(timer);
  }
}

module.exports = { waitFor };
