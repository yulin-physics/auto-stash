export function save_to_chrome_storage(key, value) {
  if (Array.isArray(value)) {
    const obj = { formData: value };
    chrome.storage.local.set(obj, () => {
      console.log("Saved full form data", value);
    });
  } else if (typeof value === "boolean") {
    // Used for form_submitted flag
    chrome.storage.local.set({ [key]: value }, () => {
      console.log(`Saved flag ${key} = ${value}`);
    });
  } else {
    chrome.storage.local.get(["formData"], (result) => {
      let formData = result.formData || {};
      formData[value.name] = value.value;
      chrome.storage.local.set({ formData }, () => {
        console.log("Saved field", value.name, value.value);
      });
    });
  }
}

export function get_from_chrome_storage(key) {
  return new Promise((resolve) => {
    chrome.storage.local.get([key], (result) => {
      resolve(result);
    });
  });
}

export function clear_chrome_storage(key) {
  chrome.storage.local.remove([key], () => {
    console.log(`Cleared storage key: ${key}`);
  });
}

