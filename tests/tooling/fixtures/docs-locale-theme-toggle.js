document.addEventListener("click", (event) => {
  if (event.target.closest(".theme-toggle, [data-mobile-theme]")) {
    const { dataset } = document.documentElement;
    dataset.theme = dataset.theme === "dark" ? "light" : "dark";
  }
});
