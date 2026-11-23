parent.frames[0].postMessage(
  {
    type: "musea:set-globals",
    browserProbe: "foreign",
    payload: { theme: "light", locale: "en", rtl: false },
  },
  document.currentScript.dataset.origin,
);
