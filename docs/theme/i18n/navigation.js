/**
 * Sidebar grouping, label localization, and the header language picker.
 *
 * This file is behavior only. The sidebar structure lives in `sitemap.js` and
 * the strings live in `locales/<code>.js`; both register themselves on globals
 * that `theme/background.ts` guarantees are concatenated ahead of this file.
 * They are read lazily, at call time, so nothing here depends on evaluation
 * order beyond `initialize` running after the page is parsed.
 */
const vizeDocsI18nNavigation = (() => {
  function sitemap() {
    return globalThis.__vizeDocsSitemap;
  }

  function locales() {
    return globalThis.__vizeDocsLocales;
  }

  function localeStrings(locale) {
    return locales()[locale] ?? locales().en;
  }

  function label(locale, path, fallback) {
    return localeStrings(locale).labels[path] ?? locales().en.labels[path] ?? fallback;
  }

  function pathLocale(path) {
    const firstSegment = path.split("/").find(Boolean);
    return sitemap().supportedLocales.some(({ code }) => code === firstSegment)
      ? firstSegment
      : "en";
  }

  function currentLocale() {
    return pathLocale(globalThis.window?.location?.pathname ?? "/");
  }

  function canonicalPath(href) {
    const origin = globalThis.window?.location?.origin ?? "https://vizejs.dev";
    const url = new URL(href, origin);
    let path = url.pathname.replace(/\/index\.html$/, "").replace(/\/$/, "") || "/";
    const locale = pathLocale(path);
    if (locale !== "en") {
      path = path.slice(locale.length + 1) || "/";
    }
    return path;
  }

  function createSection(title, items) {
    const section = document.createElement("div");
    section.className = "nav-section";

    const heading = document.createElement("div");
    heading.className = "nav-title";
    heading.textContent = title;
    section.append(heading);

    const list = document.createElement("ul");
    list.className = "nav-list";
    for (const item of items) {
      list.append(item);
    }
    section.append(list);

    return section;
  }

  function isHiddenFallbackPath(path) {
    return sitemap().hiddenPathPatterns.some((pattern) => pattern.test(path));
  }

  function applyNavigationOrder(root = document) {
    const nav = root.querySelector?.(".sidebar nav");
    if (!nav || nav.dataset.vizeNavigation === "structured") {
      return;
    }

    const locale = currentLocale();
    const ui = localeStrings(locale).ui;
    const itemsByPath = new Map();
    const unusedItems = [];
    for (const item of nav.querySelectorAll(".nav-item")) {
      const link = item.querySelector(".nav-link[href]");
      if (!link) {
        unusedItems.push(item);
        continue;
      }

      const href = link.getAttribute("href");
      const url = new URL(href, window.location.origin);
      if (pathLocale(url.pathname) !== locale) {
        continue;
      }

      const path = canonicalPath(href);
      link.textContent = label(locale, path, link.textContent.trim());
      if (itemsByPath.has(path)) {
        unusedItems.push(item);
        continue;
      }
      itemsByPath.set(path, item);
    }

    const nextNav = document.createDocumentFragment();
    const used = new Set();
    for (const group of sitemap().navGroups) {
      const items = group.paths
        .map((path) => {
          used.add(path);
          return itemsByPath.get(path);
        })
        .filter(Boolean);

      if (items.length > 0) {
        nextNav.append(createSection(ui.groups[group.key], items));
      }
    }

    const remainingItems = [...itemsByPath]
      .filter(([path]) => !used.has(path) && !isHiddenFallbackPath(path))
      .map(([, item]) => item)
      .concat(unusedItems);
    if (remainingItems.length > 0) {
      nextNav.append(createSection(ui.more, remainingItems));
    }

    nav.replaceChildren(nextNav);
    nav.dataset.vizeNavigation = "structured";
  }

  function localizedPagePath(locale) {
    const logicalPath = canonicalPath(window.location.pathname);
    const localePrefix = locale === "en" ? "" : `/${locale}`;
    const pagePath = logicalPath === "/" ? "" : logicalPath;
    return `${localePrefix}${pagePath}/index.html`.replace(/^\/\//, "/");
  }

  function applyLocalizedChrome(root = document) {
    const locale = currentLocale();
    const ui = localeStrings(locale).ui;
    const headerTitle = root.querySelector?.(".header-title");
    if (headerTitle) {
      headerTitle.setAttribute("href", locale === "en" ? "/index.html" : `/${locale}/index.html`);
    }

    const searchButton = root.querySelector?.(".search-button");
    const searchText = searchButton?.querySelector("span");
    if (searchText) searchText.textContent = ui.search;
    if (searchButton) searchButton.setAttribute("aria-label", ui.search);

    const searchInput = root.querySelector?.(".search-input");
    if (searchInput) searchInput.setAttribute("placeholder", ui.searchPlaceholder);

    const footerMessage = root.querySelector?.(".footer-message");
    if (footerMessage) footerMessage.innerHTML = ui.footer;
  }

  function installLocaleSwitcher(root = document) {
    const headerActions = root.querySelector?.(".header-actions");
    if (!headerActions || headerActions.querySelector(".docs-locale")) return;

    const locale = currentLocale();
    const language = localeStrings(locale).ui.language;
    const wrapper = document.createElement("label");
    wrapper.className = "docs-locale";
    wrapper.setAttribute("aria-label", language);

    const labelElement = document.createElement("span");
    labelElement.textContent = language;
    const select = document.createElement("select");
    select.className = "docs-locale-select";
    select.setAttribute("aria-label", language);
    for (const supportedLocale of sitemap().supportedLocales) {
      const option = document.createElement("option");
      option.value = supportedLocale.code;
      option.textContent = supportedLocale.name;
      option.selected = supportedLocale.code === locale;
      select.append(option);
    }
    select.addEventListener("change", () => {
      window.location.href = `${localizedPagePath(select.value)}${window.location.search}${window.location.hash}`;
    });
    wrapper.append(labelElement, select);

    const searchButton = headerActions.querySelector(".search-button");
    headerActions.insertBefore(wrapper, searchButton);
  }

  function initialize(root = document) {
    applyNavigationOrder(root);
    applyLocalizedChrome(root);
    installLocaleSwitcher(root);
  }

  return {
    applyNavigationOrder,
    canonicalPath,
    currentLocale,
    initialize,
  };
})();

if (typeof globalThis !== "undefined") {
  globalThis.__vizeDocsNavigation = vizeDocsI18nNavigation;
}

(() => {
  if (typeof document === "undefined") {
    return;
  }

  const start = () => vizeDocsI18nNavigation.initialize(document);

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start, { once: true });
    return;
  }

  start();
})();
