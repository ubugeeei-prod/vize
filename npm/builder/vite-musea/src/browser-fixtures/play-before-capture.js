window.__museaRunPlay = async () => {
  await new Promise((resolve) => setTimeout(resolve, 20));
  const state = document.querySelector("#state");
  state.textContent = "Played";
  state.style.background = "#00f";
};
window.postMessage({ type: "musea:ready" }, location.origin);
