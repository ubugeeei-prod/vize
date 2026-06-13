import assert from "node:assert/strict";
import test from "node:test";

await import("./navigation.js");

const navigation = globalThis.__vizeDocsNavigation;

class TestElement {
  constructor(tagName) {
    this.tagName = tagName.toUpperCase();
    this.attributes = new Map();
    this.children = [];
    this.className = "";
    this.dataset = {};
    this.parentNode = null;
    this.isFragment = false;
    this.value = "";
  }

  append(...nodes) {
    for (const node of nodes) {
      if (node.isFragment) {
        const children = node.children.slice();
        node.children.length = 0;
        this.append(...children);
        continue;
      }

      if (node.parentNode) {
        const siblings = node.parentNode.children;
        const index = siblings.indexOf(node);
        if (index !== -1) {
          siblings.splice(index, 1);
        }
      }

      node.parentNode = this;
      this.children.push(node);
    }
  }

  getAttribute(name) {
    return this.attributes.get(name) ?? null;
  }

  setAttribute(name, value) {
    this.attributes.set(name, String(value));
  }

  get textContent() {
    return this.value + this.children.map((child) => child.textContent).join("");
  }

  set textContent(value) {
    this.children.length = 0;
    this.value = String(value);
  }

  querySelector(selector) {
    return this.querySelectorAll(selector)[0] ?? null;
  }

  querySelectorAll(selector) {
    const selectors = selector.trim().split(/\s+/);
    const targetSelector = selectors.pop();
    const matches = [];

    this.visitChildren((element) => {
      if (
        matchesSimpleSelector(element, targetSelector) &&
        matchesAncestorSelectors(element, selectors)
      ) {
        matches.push(element);
      }
    });

    return matches;
  }

  replaceChildren(...nodes) {
    for (const child of this.children) {
      child.parentNode = null;
    }
    this.children.length = 0;
    this.value = "";
    this.append(...nodes);
  }

  visitChildren(callback) {
    for (const child of this.children) {
      callback(child);
      child.visitChildren(callback);
    }
  }
}

class TestDocument extends TestElement {
  constructor(nav) {
    super("#document");
    this.readyState = "complete";

    const sidebar = this.createElement("div");
    sidebar.className = "sidebar";
    sidebar.append(nav);
    this.append(sidebar);
  }

  addEventListener() {}

  createDocumentFragment() {
    const fragment = new TestElement("#fragment");
    fragment.isFragment = true;
    return fragment;
  }

  createElement(tagName) {
    return new TestElement(tagName);
  }
}

function matchesAncestorSelectors(element, selectors) {
  let parent = element.parentNode;

  for (let index = selectors.length - 1; index >= 0; index -= 1) {
    while (parent && !matchesSimpleSelector(parent, selectors[index])) {
      parent = parent.parentNode;
    }

    if (!parent) {
      return false;
    }

    parent = parent.parentNode;
  }

  return true;
}

function matchesSimpleSelector(element, selector) {
  const attrIndex = selector.indexOf("[");
  const baseSelector = attrIndex === -1 ? selector : selector.slice(0, attrIndex);
  const attrName = attrIndex === -1 ? "" : selector.slice(attrIndex + 1, -1);

  if (attrName && !element.attributes.has(attrName)) {
    return false;
  }

  if (!baseSelector) {
    return true;
  }

  if (baseSelector[0] === ".") {
    return element.className.split(/\s+/).includes(baseSelector.slice(1));
  }

  return element.tagName.toLowerCase() === baseSelector.toLowerCase();
}

function createNavigationDocument(items) {
  const nav = new TestElement("nav");

  for (const [href, label] of items) {
    const item = new TestElement("li");
    item.className = "nav-item";

    const link = new TestElement("a");
    link.className = "nav-link";
    link.setAttribute("href", href);
    link.textContent = label;

    item.append(link);
    nav.append(item);
  }

  return new TestDocument(nav);
}

function applyNavigation(document) {
  const previousDocument = globalThis.document;
  const previousWindow = globalThis.window;

  globalThis.document = document;
  globalThis.window = { location: { origin: "https://docs.example.test" } };

  try {
    navigation.applyNavigationOrder(document);
  } finally {
    globalThis.document = previousDocument;
    globalThis.window = previousWindow;
  }
}

function sections(document) {
  const nav = document.querySelector(".sidebar nav");

  return nav.querySelectorAll(".nav-section").map((section) => ({
    title: section.querySelector(".nav-title").textContent,
    labels: section.querySelectorAll(".nav-link[href]").map((link) => link.textContent),
  }));
}

void test("applyNavigationOrder keeps the Blog group compact", () => {
  const document = createNavigationDocument([
    ["/", "index"],
    ["/blog", "Blog"],
    ["/blog/notes", "Notes"],
    ["/blog/releases", "Releases"],
    [
      "/blog/notes/2026-05-16-performance-tuning-notes-for-a-vue-toolchain",
      "2026-05-16-performance-tuning-notes-for-a-vue-toolchain",
    ],
    ["/blog/releases/2026-03-26-docs-blog-support", "2026-03-26-docs-blog-support"],
  ]);

  applyNavigation(document);

  assert.deepEqual(sections(document), [
    {
      title: "Start",
      labels: ["Overview"],
    },
    {
      title: "Blog",
      labels: ["Overview", "Notes", "Releases"],
    },
  ]);
});

void test("applyNavigationOrder keeps the stability contract in Start", () => {
  const document = createNavigationDocument([
    ["/", "index"],
    ["/getting-started", "Getting Started"],
    ["/stability", "Stability"],
    ["/credits", "Credits"],
  ]);

  applyNavigation(document);

  assert.deepEqual(sections(document), [
    {
      title: "Start",
      labels: ["Overview", "Getting Started", "Stability", "Credits"],
    },
  ]);
});

void test("applyNavigationOrder keeps developer architecture pages together", () => {
  const document = createNavigationDocument([
    ["/architecture/overview", "Architecture"],
    ["/architecture/crates", "Crates"],
    ["/architecture/source-guide", "Source Guide"],
    ["/architecture/language-engineering-practices", "Language Engineering Practices"],
    ["/architecture/performance", "Performance"],
  ]);

  applyNavigation(document);

  assert.deepEqual(sections(document), [
    {
      title: "Architecture",
      labels: [
        "Architecture Overview",
        "Crates",
        "Source Guide",
        "Language Engineering",
        "Performance",
      ],
    },
  ]);
});

void test("applyNavigationOrder hides dated blog posts from the More fallback", () => {
  const document = createNavigationDocument([
    ["/internal/reference", "Internal Reference"],
    [
      "/blog/notes/2026-05-16-performance-tuning-notes-for-a-vue-toolchain",
      "2026-05-16-performance-tuning-notes-for-a-vue-toolchain",
    ],
    [
      "/blog/notes/2026-05-17-long-note-that-is-not-in-the-curated-blog-navigation",
      "2026-05-17-long-note-that-is-not-in-the-curated-blog-navigation",
    ],
    [
      "/blog/releases/2026-05-17-long-release-that-is-not-in-the-curated-blog-navigation",
      "2026-05-17-long-release-that-is-not-in-the-curated-blog-navigation",
    ],
  ]);

  applyNavigation(document);

  assert.deepEqual(sections(document), [
    {
      title: "More",
      labels: ["Internal Reference"],
    },
  ]);
});
