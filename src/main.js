const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;
let lvStatusEl;
let vgStatusEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
  lvStatusEl.textContent = await invoke("get_logical_volume_status");
  
  let vvvv = await invoke("get_volume_group_status");
  vgStatusEl.textContent = vvvv;
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  lvStatusEl = document.querySelector("#lv-status");
  vgStatusEl = document.querySelector("#vg-status");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});
