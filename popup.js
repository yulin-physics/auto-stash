const toggle = document.getElementById("enableToggle");
    const statusText = document.getElementById("toggleStatus");
    const storageList = document.getElementById("storageList");

    chrome.storage.local.get("enabled", (data) => {
      toggle.checked = data.enabled ?? false;
      statusText.textContent = toggle.checked ? "Auto-save Enabled" : "Auto-save Disabled";
    });

    toggle.addEventListener("change", () => {
      const enabled = toggle.checked;
      chrome.storage.local.set({ enabled: enabled }, () => {
        statusText.textContent = enabled ? "Auto-save Enabled" : "Auto-save Disabled";
      });
    });

    chrome.storage.local.get(null, (items) => {
      storageList.innerHTML = "";
      const keys = Object.keys(items).filter(k => k !== "enabled");

      if (keys.length === 0) {
        storageList.innerHTML = "<p id=\"no-data\" class=\"empty\">No saved form data.</p>";
        return;
      }

      keys.forEach((key) => {
        const label = document.createElement("span");
        label.className = "entry-label";
        label.textContent = key;

        const li = document.createElement("li");

        const btn = document.createElement("button");
        btn.textContent = "Clear";
        btn.addEventListener("click", () => {
          chrome.storage.local.remove(key, () => {
            li.remove();
          });
        });

        li.appendChild(label);
        li.appendChild(btn);
        storageList.appendChild(li);
      });
    });