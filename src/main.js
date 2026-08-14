// 跟 src-tauri/src/key.md、remap.rs 的 MACRO_TABLE 一一對應，改動鍵位時兩邊要一起改
// chain 裡的字母大小寫要跟 remap.rs 實際送出的 VK code 一致：
// 沒帶 Shift 的字母鍵（i/v/u/w/q/s）送出的就是小寫，不能因為好看硬改大寫
const KEYMAP = [
  { trigger: "CapsLock", chain: ["Esc"], purpose: "正常模式" },
  { trigger: "CapsLock + I", chain: ["Esc", "i"], purpose: "編輯模式" },
  { trigger: "CapsLock + V", chain: ["Esc", "v"], purpose: "視覺模式" },
  { trigger: "CapsLock + R", chain: ["Esc", "u"], purpose: "復原" },
  { trigger: "CapsLock + U", chain: ["Esc", "Ctrl + R"], purpose: "重做" },
  { trigger: "CapsLock + ;", chain: ["Esc", ":"], purpose: "命令列模式" },
  { trigger: "CapsLock + F", chain: ["Esc", "/"], purpose: "尋找" },
  { trigger: "CapsLock + S", chain: ["Esc", ":", "w", "q", "!"], purpose: "儲存並離開" },
  { trigger: "CapsLock + H", chain: ["Esc", ":", "%", "s", "/"], purpose: "全域取代" },
];

function pill(text) {
  const span = document.createElement("span");
  span.className = "pill";
  span.textContent = text;
  return span;
}

function arrow() {
  const span = document.createElement("span");
  span.className = "arrow";
  span.textContent = "→";
  return span;
}

// 把「箭頭 + 按鍵」包成一個不可拆的區塊，換行時整段一起換，不會拆散
function step(text) {
  const wrapper = document.createElement("span");
  wrapper.className = "step";
  wrapper.appendChild(arrow());
  wrapper.appendChild(pill(text));
  return wrapper;
}

// 每一列分三行：觸發鍵 / 解釋（放中間，當標題） / 實際送出的按鍵鏈
function renderKeymap() {
  const container = document.getElementById("keymap-list");

  for (const { trigger, chain, purpose } of KEYMAP) {
    const row = document.createElement("div");
    row.className = "chain-row";

    const triggerLine = document.createElement("div");
    triggerLine.className = "row-trigger";
    triggerLine.appendChild(pill(trigger));
    row.appendChild(triggerLine);

    const purposeLine = document.createElement("div");
    purposeLine.className = "row-purpose";
    purposeLine.textContent = purpose;
    row.appendChild(purposeLine);

    const chainLine = document.createElement("div");
    chainLine.className = "row-chain";
    for (const key of chain) {
      chainLine.appendChild(step(key));
    }
    row.appendChild(chainLine);

    container.appendChild(row);
  }
}

renderKeymap();

const { invoke } = window.__TAURI__.core;

function closeKeymap() {
  invoke("close_keymap");
}

document.getElementById("close-btn").addEventListener("click", closeKeymap);

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    closeKeymap();
  }
});
