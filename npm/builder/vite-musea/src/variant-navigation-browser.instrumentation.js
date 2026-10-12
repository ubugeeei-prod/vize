// Playwright reads these raw bytes directly. A tsx-transformed function closure can
// contain a host-only __name helper, which must never become part of this browser probe.
(() => {
  if (window !== window.top) return;
  const Native = window.IntersectionObserver;
  const probe = { records: [], events: [] };
  Reflect.set(window, "__museaNavigationProbe", probe);
  const summarize = (entries) =>
    entries.map((entry) => ({
      name: entry.target.dataset.variantName,
      time: entry.time,
      isIntersecting: entry.isIntersecting,
      ratio: entry.intersectionRatio,
      bounds: entry.boundingClientRect.toJSON(),
      root: entry.rootBounds?.toJSON(),
      intersection: entry.intersectionRect.toJSON(),
    }));
  window.IntersectionObserver = class extends Native {
    constructor(callback, options) {
      const record = { id: probe.records.length, callback, disconnected: false, deliveries: [] };
      super((entries, observer) => {
        record.deliveries.push(entries);
        // Forward the complete original native entry array and observer without delay.
        callback(entries, observer);
        probe.events.push({
          kind: "native-delivery",
          id: record.id,
          time: performance.now(),
          disconnected: record.disconnected,
          entries: summarize(entries),
        });
      }, options);
      record.observer = this;
      this.record = record;
      probe.records.push(record);
      probe.events.push({ kind: "construct", id: record.id, options, time: performance.now() });
    }
    disconnect() {
      this.record.disconnected = true;
      probe.events.push({ kind: "disconnect", id: this.record.id, time: performance.now() });
      super.disconnect();
    }
  };
  document.addEventListener(
    "click",
    (event) => {
      const item = event.target?.closest(".variant-toc-item");
      if (item)
        probe.events.push({
          kind: "click",
          time: performance.now(),
          name: item.querySelector(".variant-toc-name")?.textContent,
          aria: item.getAttribute("aria-current"),
        });
    },
    true,
  );
})();
