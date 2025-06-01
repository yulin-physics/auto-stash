(async () => {
  const wasm = await import("./pkg/auto_stash.js");
  await wasm.default();
})();
