(() => {
  if (typeof document === "undefined") return;
  const { choices } = globalThis.__vizeDocsCommands;
  const words = {
    en: [
      "Package manager",
      "Copy command",
      "Copied",
      "Copy failed; select the command to copy it.",
      "This operation uses npm packages. JSR installation is not available for this command.",
      "This example uses workspace-specific flags. Use the original command below; changing package managers requires updating the workspace and lockfile.",
    ],
    ja: [
      "パッケージマネージャー",
      "コマンドをコピー",
      "コピーしました",
      "コピーできませんでした。コマンドを選択してコピーしてください。",
      "この操作には npm パッケージを使用します。このコマンドは JSR からインストールできません。",
      "この例はワークスペース固有のフラグを使用します。下記の元のコマンドを使用してください。パッケージマネージャーを変更する場合は、ワークスペース設定とロックファイルの更新が必要です。",
    ],
    "zh-CN": [
      "包管理器",
      "复制命令",
      "已复制",
      "复制失败；请选择命令并复制。",
      "此操作使用 npm 包。此命令无法通过 JSR 安装。",
      "此示例使用工作区特定选项。请使用下面的原始命令；更换包管理器需要更新工作区和锁文件。",
    ],
    "pt-BR": [
      "Gerenciador de pacotes",
      "Copiar comando",
      "Copiado",
      "Selecione o comando para copiá-lo.",
      "Esta operação usa pacotes npm. Este comando não pode ser instalado pelo JSR.",
      "Este exemplo usa opções específicas do workspace. Use o comando original abaixo; trocar de gerenciador exige atualizar o workspace e o arquivo de lock.",
    ],
    fr: [
      "Gestionnaire de paquets",
      "Copier la commande",
      "Copié",
      "Sélectionnez la commande pour la copier.",
      "Cette opération utilise des paquets npm. Cette commande ne peut pas être installée depuis JSR.",
      "Cet exemple utilise des options propres au workspace. Utilisez la commande originale ci-dessous ; changer de gestionnaire demande de mettre à jour le workspace et le fichier de verrouillage.",
    ],
  };
  const strings = words[location.pathname.split("/")[1]] ?? words.en;
  let serial = 0;
  const widgets = new Set();
  let preference = "vp";
  try {
    preference = localStorage.getItem("vize-package-manager") ?? preference;
  } catch {}

  function mount(code) {
    const pre = code.parentElement;
    if (!pre || pre.dataset.commandTabsSource || pre.closest(".vize-command-tabs")) return;
    const language = `${code.className} ${pre.className} ${code.dataset.language ?? ""} ${pre.dataset.language ?? ""}`;
    if (!/\b(?:language-|lang-)?(?:bash|sh|shell|console|shellscript)\b/i.test(language)) return;
    const source = code.textContent ?? "";
    const variants = choices(source);
    if (!variants) return;
    const sourceName = source.match(
      /^\s*(vp|vpx|npm|npx|pnpm|yarn|bun|bunx|aube|aubr|aubx)\b/m,
    )?.[1];
    const originalManager =
      { vpx: "vp", npx: "npm", bunx: "bun", aubr: "aube", aubx: "aube" }[sourceName] ?? sourceName;
    variants.find((value) => value.manager === originalManager).command = source;
    pre.dataset.commandTabsSource = "true";
    const wrapper = document.createElement("div");
    wrapper.className = "vize-command-tabs";
    const list = document.createElement("div");
    list.className = "vize-command-tablist";
    list.setAttribute("role", "tablist");
    list.setAttribute("aria-label", strings[0]);
    wrapper.append(list);
    const tabs = [],
      panels = [];
    const identifier = `vize-command-${++serial}`;
    for (const [index, value] of variants.entries()) {
      const tab = document.createElement("button");
      tab.type = "button";
      tab.textContent = value.manager;
      tab.id = `${identifier}-tab-${index}`;
      tab.setAttribute("role", "tab");
      tab.setAttribute("aria-controls", `${identifier}-panel-${index}`);
      const panel = document.createElement("div");
      panel.className = "vize-command-panel";
      panel.id = `${identifier}-panel-${index}`;
      panel.setAttribute("role", "tabpanel");
      panel.setAttribute("aria-labelledby", tab.id);
      panel.tabIndex = 0;
      if (value.command === null) {
        const note = document.createElement("p");
        note.className = "vize-command-gap";
        note.dataset.commandGap = value.gap;
        note.textContent = strings[value.gap === "registry" ? 4 : 5];
        panel.append(note);
        if (value.gap !== "registry") panel.append(pre.cloneNode(true));
      } else {
        const original = value.manager === originalManager;
        const block = original ? pre.cloneNode(true) : document.createElement("pre");
        if (original) block.querySelector("code").dataset.commandTabsOriginal = "true";
        else {
          const example = document.createElement("code");
          example.className = "language-bash";
          example.textContent = value.command;
          block.append(example);
        }
        const copy = document.createElement("button");
        copy.type = "button";
        copy.className = "vize-command-copy";
        copy.textContent = strings[1];
        const status = document.createElement("span");
        status.className = "vize-command-copy-status";
        status.setAttribute("role", "status");
        copy.addEventListener("click", async () => {
          try {
            await navigator.clipboard.writeText(value.command);
            status.textContent = strings[2];
          } catch {
            status.textContent = strings[3];
          }
        });
        panel.append(copy, status, block);
      }
      tabs.push(tab);
      panels.push(panel);
      list.append(tab);
      wrapper.append(panel);
    }
    function select(index, focus = false, synchronize = false) {
      tabs.forEach((tab, i) => {
        tab.setAttribute("aria-selected", String(i === index));
        tab.tabIndex = i === index ? 0 : -1;
        panels[i].hidden = i !== index;
      });
      if (focus) tabs[index].focus();
      preference = variants[index].manager;
      try {
        localStorage.setItem("vize-package-manager", preference);
      } catch {}
      if (synchronize) {
        for (const widget of widgets) {
          if (!widget.element.isConnected) widgets.delete(widget);
          else widget.apply(preference);
        }
      }
    }
    tabs.forEach((tab, index) => {
      tab.addEventListener("click", () => select(index, false, true));
      tab.addEventListener("keydown", (event) => {
        let next;
        if (event.key === "ArrowRight") next = (index + 1) % tabs.length;
        if (event.key === "ArrowLeft") next = (index + tabs.length - 1) % tabs.length;
        if (event.key === "Home") next = 0;
        if (event.key === "End") next = tabs.length - 1;
        if (next !== undefined) {
          event.preventDefault();
          select(next, true, true);
        }
      });
    });
    select(
      Math.max(
        0,
        variants.findIndex((value) => value.manager === preference),
      ),
    );
    // Keep the original SSR code block readable when JavaScript is disabled.
    pre.replaceWith(wrapper);
    widgets.add({
      element: wrapper,
      apply: (manager) => select(variants.findIndex((value) => value.manager === manager)),
    });
  }

  const enhance = () => document.querySelectorAll("pre > code").forEach(mount);
  const start = () => {
    enhance();
    let scheduled = false;
    new MutationObserver((changes) => {
      if (scheduled || !changes.some((change) => change.addedNodes.length)) return;
      scheduled = true;
      requestAnimationFrame(() => {
        scheduled = false;
        enhance();
      });
    }).observe(document.body, { childList: true, subtree: true });
  };
  if (document.readyState === "loading")
    document.addEventListener("DOMContentLoaded", start, { once: true });
  else start();
})();
