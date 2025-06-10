let wasm = null;

async function initWasm() {
  const wasm = await import("./pkg/auto_stash.js");
  await wasm.default();

  return wasm.api();
}

(async () => {
  wasm = await initWasm()
  await wasm.start();
})();

chrome.storage.onChanged.addListener(async (changes, area) => {
  if (area === "local" && "enabled" in changes) {
  await wasm.start();
  }
});
