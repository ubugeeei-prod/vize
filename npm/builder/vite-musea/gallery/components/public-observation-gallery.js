console.error("actual-gallery-diagnostic-console");
void fetch("/missing.js")
  .then((response) => response.text())
  .then((text) => (window.__actualFailureBody = text));
