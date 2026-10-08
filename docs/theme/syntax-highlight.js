const vizeDocsSyntax = (() => {
  const core = globalThis.__vizeDocsSyntaxCore;
  const { createHighlightedHtml, displayLanguage, normalizeLanguage } = core;

  function detectLanguage(codeElement, preElement) {
    const candidates = [
      codeElement.getAttribute("data-language"),
      preElement?.getAttribute("data-language"),
      ...(codeElement.className || "").split(/\s+/),
      ...((preElement?.className || "").split(/\s+/) ?? []),
    ];

    for (const candidate of candidates) {
      if (!candidate) {
        continue;
      }
      const match = candidate.match(/^(?:language-|lang-)?([\w-]+)$/);
      const normalized = normalizeLanguage(match?.[1] ?? candidate);
      if (normalized !== "text") {
        return normalized;
      }
    }

    return "text";
  }

  function highlightCodeElement(codeElement) {
    const preElement = codeElement.closest("pre");
    if (!preElement) {
      return;
    }

    const language = detectLanguage(codeElement, preElement);
    const rawSource = codeElement.textContent ?? "";
    const signature = `${language}:${rawSource}`;

    if (preElement.dataset.vizeSyntaxSignature === signature) {
      return;
    }

    preElement.dataset.language = displayLanguage(language);
    preElement.dataset.vizeSyntaxSignature = signature;

    if (language === "mermaid" || codeElement.classList.contains("mermaid")) {
      return;
    }

    const highlightedSource = createHighlightedHtml(rawSource, language);
    const nativeLines = Array.from(codeElement.children).filter((line) =>
      line.classList.contains("ox-code-line"),
    );

    if (nativeLines.length === 0) {
      codeElement.innerHTML = highlightedSource;
      return;
    }

    // Keep Ox Content's line elements, annotation attributes, and separators.
    // Highlight the whole source first so multiline syntax retains its context.
    const highlightedLines = highlightedSource.split("\n");
    if (highlightedLines.length !== nativeLines.length) {
      delete preElement.dataset.vizeSyntaxSignature;
      return;
    }
    for (const [index, line] of nativeLines.entries()) {
      line.innerHTML = highlightedLines[index];
    }
  }

  function highlightAll(root = document) {
    if (!root?.querySelectorAll) {
      return;
    }

    const codeBlocks = root.querySelectorAll("pre > code");
    for (const codeElement of codeBlocks) {
      highlightCodeElement(codeElement);
    }
  }

  return {
    createHighlightedHtml,
    detectLanguage,
    displayLanguage,
    highlightAll,
    highlightCodeElement,
    normalizeLanguage,
  };
})();

if (typeof globalThis !== "undefined") {
  globalThis.__vizeDocsSyntax = vizeDocsSyntax;
}

(() => {
  if (typeof document === "undefined") {
    return;
  }

  let scheduled = false;
  const scheduleHighlight = () => {
    if (scheduled) {
      return;
    }

    scheduled = true;
    requestAnimationFrame(() => {
      scheduled = false;
      vizeDocsSyntax.highlightAll(document);
    });
  };

  const observer = new MutationObserver((mutations) => {
    if (mutations.some((mutation) => mutation.addedNodes.length > 0)) {
      scheduleHighlight();
    }
  });

  const start = () => {
    vizeDocsSyntax.highlightAll(document);
    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start, { once: true });
    return;
  }

  start();
})();
