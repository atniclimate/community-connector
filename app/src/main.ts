const app = document.querySelector<HTMLElement>("#app");

if (app === null) {
  throw new Error("Missing #app element");
}

app.textContent = "Community Navigator - Phase 0 scaffold";
