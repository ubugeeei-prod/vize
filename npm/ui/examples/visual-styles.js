const themeButtons = document.querySelectorAll("[data-theme]");
const overlay = document.querySelector('[data-vize-ui="dialog-overlay"]');
const dialog = document.querySelector('[data-vize-ui="dialog-content"]');
const opener = document.querySelector('[data-action="open-dialog"]');
const checkbox = document.querySelector('[data-vize-ui="checkbox"]');
const switchControl = document.querySelector('[data-vize-ui="switch"]');
const tooltipTrigger = document.querySelector('[data-vize-ui="tooltip-trigger"]');
const tooltipContent = document.querySelector('[data-vize-ui="tooltip-content"]');
tooltipContent.id = `tooltip-content-${crypto.randomUUID()}`;
const tooltipPositioner = tooltipContent.parentElement;
const tooltipRoot = tooltipTrigger.closest('[data-vize-ui="tooltip-root"]');

for (const button of themeButtons) {
  button.addEventListener("click", () => {
    document.documentElement.dataset.vizeTheme = button.dataset.theme;
    for (const choice of themeButtons) {
      choice.setAttribute("aria-pressed", String(choice === button));
    }
  });
}

checkbox.addEventListener("change", () => {
  checkbox.dataset.state = checkbox.checked ? "checked" : "unchecked";
});
switchControl.addEventListener("click", () => {
  const checked = switchControl.getAttribute("aria-checked") !== "true";
  switchControl.setAttribute("aria-checked", String(checked));
  switchControl.dataset.checked = String(checked);
  switchControl.dataset.state = checked ? "checked" : "unchecked";
});

const previewTabs = [
  ...document.querySelectorAll('[data-visual-tabs] [data-vize-ui="tabs-trigger"]'),
];
const previewPanels = [
  ...document.querySelectorAll('[data-visual-tabs] [data-vize-ui="tabs-content"]'),
];
previewTabs.forEach((tab, index) => {
  const panel = previewPanels[index];
  tab.id = `visual-tab-${index}`;
  panel.id = `visual-panel-${index}`;
  tab.setAttribute("aria-controls", panel.id);
  panel.setAttribute("aria-labelledby", tab.id);
});
function selectPreviewTab(selected, focus = false) {
  previewTabs.forEach((tab, index) => {
    const active = tab === selected;
    tab.dataset.selected = String(active);
    tab.setAttribute("aria-selected", String(active));
    tab.tabIndex = active ? 0 : -1;
    previewPanels[index].hidden = !active;
  });
  if (focus) selected.focus();
}
previewTabs.forEach((tab, index) => {
  tab.addEventListener("click", () => selectPreviewTab(tab));
  tab.addEventListener("keydown", (event) => {
    if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const next =
      event.key === "Home"
        ? 0
        : event.key === "End"
          ? previewTabs.length - 1
          : (index + (event.key === "ArrowRight" ? 1 : -1) + previewTabs.length) %
            previewTabs.length;
    selectPreviewTab(previewTabs[next], true);
  });
});
selectPreviewTab(previewTabs[0]);

const pageButtons = [...document.querySelectorAll('[data-vize-ui="pagination-page"]')];
const previousPage = document.querySelector('[data-vize-ui="pagination-previous"]');
const nextPage = document.querySelector('[data-vize-ui="pagination-next"]');
let currentPage = 2;
function selectPreviewPage(page) {
  currentPage = Math.min(3, Math.max(1, page));
  for (const button of pageButtons) {
    const active = Number(button.textContent.trim()) === currentPage;
    button.dataset.current = String(active);
    if (active) button.setAttribute("aria-current", "page");
    else button.removeAttribute("aria-current");
  }
  previousPage.disabled = currentPage === 1;
  nextPage.disabled = currentPage === 3;
}
pageButtons.forEach((button) =>
  button.addEventListener("click", () => selectPreviewPage(Number(button.textContent.trim()))),
);
previousPage.addEventListener("click", () => selectPreviewPage(currentPage - 1));
nextPage.addEventListener("click", () => selectPreviewPage(currentPage + 1));

const previewSteps = [...document.querySelectorAll('[data-vize-ui="stepper-trigger"]')];
const previewStepContent = document.querySelector('[data-vize-ui="stepper-content"]');
const stepDescriptions = [
  "The draft is ready for review.",
  "Review the details before publishing.",
  "Everything is ready to publish.",
];
previewSteps.forEach((button, index) =>
  button.addEventListener("click", () => {
    previewSteps.forEach((step, stepIndex) => {
      step.dataset.current = String(stepIndex === index);
      step.dataset.completed = String(stepIndex < index);
      if (stepIndex === index) step.setAttribute("aria-current", "step");
      else step.removeAttribute("aria-current");
    });
    previewStepContent.textContent = stepDescriptions[index];
  }),
);

function showTooltip(open) {
  tooltipRoot.dataset.state = open ? "open" : "closed";
  tooltipTrigger.dataset.state = open ? "open" : "closed";
  if (open) tooltipTrigger.setAttribute("aria-describedby", tooltipContent.id);
  else tooltipTrigger.removeAttribute("aria-describedby");
  tooltipContent.dataset.state = open ? "open" : "closed";
  tooltipContent.hidden = !open;
  tooltipPositioner.hidden = !open;
}

tooltipTrigger.addEventListener("mouseenter", () => showTooltip(true));
tooltipTrigger.addEventListener("mouseleave", () => showTooltip(false));
tooltipTrigger.addEventListener("focus", () => showTooltip(true));
tooltipTrigger.addEventListener("blur", () => showTooltip(false));

function closeDialog() {
  overlay.hidden = true;
  dialog.hidden = true;
  opener.focus();
}

opener.addEventListener("click", () => {
  overlay.hidden = false;
  dialog.hidden = false;
  dialog.focus();
});
dialog.querySelector('[data-vize-ui="dialog-close"]').addEventListener("click", closeDialog);
dialog.querySelector('[data-action="save-dialog"]').addEventListener("click", closeDialog);
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && !dialog.hidden) closeDialog();
  if (event.key === "Escape") showTooltip(false);
});
