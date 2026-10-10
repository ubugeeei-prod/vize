/** Authored language links with disclosure-button and native link semantics. */
const vizeDocsLocaleSwitcher = (() => {
  const mounts = new WeakMap();

  function install(root, { locale, language, supportedLocales, pagePath }) {
    const headerActions = root.querySelector?.(".header-actions");
    if (!headerActions) return;
    const previous = mounts.get(root);
    if (previous?.host === headerActions && previous.wrapper.isConnected) {
      previous.update({ locale, language, pagePath });
      return;
    }
    previous?.dispose();

    const wrapper = document.createElement("div");
    wrapper.className = "docs-locale";
    const label = document.createElement("span");
    label.className = "docs-locale-label";
    const button = document.createElement("button");
    button.type = "button";
    button.className = "docs-locale-select";
    button.setAttribute("aria-controls", "docs-locale-options");
    button.setAttribute("aria-expanded", "false");
    const name = document.createElement("span");
    name.className = "docs-locale-name";
    const chevron = document.createElement("span");
    chevron.className = "docs-locale-chevron";
    chevron.setAttribute("aria-hidden", "true");
    button.append(name, chevron);

    const options = document.createElement("nav");
    options.id = "docs-locale-options";
    options.className = "docs-locale-options";
    options.hidden = true;
    const list = document.createElement("ul");
    list.className = "docs-locale-list";
    const links = supportedLocales.map((supported) => {
      const item = document.createElement("li");
      const link = document.createElement("a");
      link.className = "docs-locale-option";
      link.lang = supported.code;
      link.hreflang = supported.code;
      link.dataset.locale = supported.code;
      const text = document.createElement("span");
      text.textContent = supported.name;
      const check = document.createElement("span");
      check.className = "docs-locale-check";
      check.textContent = "✓";
      check.setAttribute("aria-hidden", "true");
      link.append(text, check);
      item.append(link);
      list.append(item);
      return link;
    });
    options.append(list);
    wrapper.append(label, button, options);
    const mobile = window.matchMedia("(max-width: 768px)");
    let ownedFocus = null;
    function place() {
      const header = headerActions.closest(".header");
      const container = mobile.matches && header ? header : headerActions;
      if (wrapper.parentNode === container) return;
      const focused = wrapper.contains(document.activeElement)
        ? document.activeElement
        : document.activeElement === document.body
          ? ownedFocus
          : null;
      container.insertBefore(
        wrapper,
        container === header ? headerActions : headerActions.querySelector(".search-button"),
      );
      if (focused instanceof HTMLElement) focused.focus({ preventScroll: true });
    }
    place();

    let selected = 0;
    let activePagePath;
    function refreshLinks() {
      links.forEach((link, i) => {
        link.href = `${activePagePath(supportedLocales[i].code)}${window.location.search}${window.location.hash}`;
      });
    }

    function update(next) {
      place();
      activePagePath = next.pagePath;
      const index = supportedLocales.findIndex((supported) => supported.code === next.locale);
      selected = index < 0 ? 0 : index;
      label.textContent = next.language;
      name.textContent = supportedLocales[selected].name;
      button.setAttribute("aria-label", `${next.language}: ${name.textContent}`);
      options.setAttribute("aria-label", next.language);
      links.forEach((link, i) => {
        if (i === selected) link.setAttribute("aria-current", "page");
        else link.removeAttribute("aria-current");
      });
      refreshLinks();
      close(wrapper.contains(document.activeElement));
    }

    function close(restoreFocus = false) {
      options.hidden = true;
      button.setAttribute("aria-expanded", "false");
      if (restoreFocus) button.focus();
    }

    function open(index) {
      refreshLinks();
      options.hidden = false;
      button.setAttribute("aria-expanded", "true");
      if (index !== undefined) links[index].focus();
    }

    const events = new AbortController();
    const listenerOptions = { signal: events.signal };
    mobile.addEventListener("change", place, listenerOptions);
    wrapper.addEventListener(
      "focusin",
      (event) => {
        ownedFocus = event.target;
      },
      listenerOptions,
    );
    // Refresh before native modified-click, middle-click, and context-menu actions.
    wrapper.addEventListener("pointerdown", refreshLinks, listenerOptions);
    wrapper.addEventListener("click", refreshLinks, listenerOptions);
    wrapper.addEventListener("contextmenu", refreshLinks, listenerOptions);
    button.addEventListener(
      "click",
      (event) => {
        if (options.hidden) open(event.detail === 0 ? selected : undefined);
        else close();
      },
      listenerOptions,
    );
    wrapper.addEventListener(
      "keydown",
      (event) => {
        if (event.defaultPrevented) return;
        if (event.key === "Enter") refreshLinks();
        if (event.key === "Escape" && !options.hidden) {
          event.preventDefault();
          event.stopPropagation();
          close(true);
          return;
        }
        const index = links.indexOf(document.activeElement);
        if (event.target !== button && index < 0) return;
        if (event.key === "ArrowDown" || event.key === "ArrowUp") {
          event.preventDefault();
          const next =
            index < 0
              ? event.key === "ArrowDown"
                ? 0
                : links.length - 1
              : (index + (event.key === "ArrowDown" ? 1 : -1) + links.length) % links.length;
          open(next);
        } else if (!options.hidden && (event.key === "Home" || event.key === "End")) {
          event.preventDefault();
          open(event.key === "Home" ? 0 : links.length - 1);
        }
        // Tab and Enter retain native link navigation and browser focus order.
      },
      listenerOptions,
    );
    root.addEventListener(
      "click",
      (event) => {
        if (!wrapper.contains(event.target)) {
          ownedFocus = null;
          close();
        }
      },
      listenerOptions,
    );
    root.addEventListener(
      "focusin",
      (event) => {
        if (!wrapper.contains(event.target)) {
          ownedFocus = null;
          close();
        }
      },
      listenerOptions,
    );

    function dispose() {
      events.abort();
      ownedFocus = null;
      observer.disconnect();
      wrapper.remove();
      mounts.delete(root);
    }
    const observer = new MutationObserver(() => {
      if (!wrapper.isConnected) dispose();
    });
    observer.observe(root, { childList: true, subtree: true });
    update({ locale, language, pagePath });
    mounts.set(root, { wrapper, host: headerActions, update, dispose });
  }

  return { install };
})();

globalThis.__vizeDocsLocaleSwitcher = vizeDocsLocaleSwitcher;
