const previewCache = new Map();

window.addEventListener("DOMContentLoaded", waitForContentElement);

function waitForContentElement() {
  if (window.content) {
    initializeAttachmentPreviews();
    observeContentChanges();
    return;
  }

  setTimeout(waitForContentElement, 100);
}

function observeContentChanges() {
  const observer = new MutationObserver(initializeAttachmentPreviews);
  observer.observe(window.content, { childList: true, subtree: true });
}

function initializeAttachmentPreviews() {
  const attachmentElements = window.content.querySelectorAll(
    ".image-attachment, .text-attachment, .pdf-attachment",
  );

  attachmentElements.forEach((element) => {
    if (element.dataset.initialized) return;

    const url = element.dataset.url;
    const id = element.dataset.id;
    if (!url || !id) return;

    if (previewCache.has(id)) {
      renderCachedPreview(element, previewCache.get(id));
    } else if (element.classList.contains("text-attachment")) {
      fetchPreview(url, id, "text");
    } else if (element.classList.contains("pdf-attachment")) {
      fetchPreview(url, id, "pdf");
    } else if (element.classList.contains("image-attachment")) {
      fetchPreview(url, id, "image", element.dataset.name || "");
    }

    element.dataset.initialized = "true";
  });
}

async function fetchPreview(url, id, type, name = "") {
  try {
    const response = await fetch(url, { cache: "force-cache" });
    if (!response.ok) {
      throw new Error(`Failed to fetch preview for ${id}: ${response.statusText}`);
    }

    if (type === "text") {
      const content = await response.text();
      const cached = { type, content };
      previewCache.set(id, cached);
      renderCachedPreview(findAttachmentElement(id, type), cached);
      return;
    }

    const blob = await response.blob();
    const objectUrl = URL.createObjectURL(blob);
    const cached = { type, objectUrl, name };
    previewCache.set(id, cached);
    renderCachedPreview(findAttachmentElement(id, type), cached);
  } catch (error) {
    console.error("Error fetching preview:", error);
  }
}

function renderCachedPreview(element, cached) {
  if (!element || !cached) return;

  element.replaceChildren();
  if (cached.type === "text") {
    const pre = document.createElement("pre");
    pre.textContent = cached.content;
    element.appendChild(pre);
  } else if (cached.type === "pdf") {
    const iframe = document.createElement("iframe");
    iframe.src = cached.objectUrl;
    iframe.type = "application/pdf";
    element.appendChild(iframe);
  } else if (cached.type === "image") {
    const image = document.createElement("img");
    image.src = cached.objectUrl;
    image.alt = cached.name || "";
    element.appendChild(image);
  }
}

function findAttachmentElement(id, type) {
  const className = {
    text: "text-attachment",
    pdf: "pdf-attachment",
    image: "image-attachment",
  }[type];

  return window.content.querySelector(`.${className}[data-id="${cssEscape(id)}"]`);
}

function cssEscape(value) {
  if (window.CSS && typeof window.CSS.escape === "function") {
    return window.CSS.escape(value);
  }

  return String(value).replace(/["\\]/g, "\\$&");
}
