window.__museaRunPlay = async () => ({
  status: "passed",
  duration: 2,
  report: {
    trace: [{ kind: "action", name: "click", status: "passed", duration: 1, target: "button" }],
  },
});
window.postMessage({ type: "musea:ready" }, location.origin);
