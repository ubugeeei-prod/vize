document.querySelector("form").addEventListener("submit", (event) => {
  event.preventDefault();
  const result = document.querySelector("#result");
  result.textContent = document.querySelector("input").value;
  result.style.background = "#0000ff";
});
