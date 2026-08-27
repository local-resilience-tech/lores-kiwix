(function () {
  let translations = {};

  async function loadTranslations() {
    try {
      const resp = await fetch("/skin/remote_content.i18n.json");
      if (resp.ok) {
        translations = await resp.json();
      } else {
        console.error(`Failed to load translations: ${resp.status}`);
      }
    } catch (err) {
      console.error("Failed to load translations", err);
    }
  }

  function getUserLanguage() {
    const params = new URLSearchParams(window.location.search);
    const queryLang = params.get("userlang");
    if (queryLang) return queryLang;
    const storedLang = localStorage.getItem("userlang");
    if (storedLang) return storedLang;
    return navigator.language || "en";
  }

  function translatePage() {
    const en = translations.en || {};
    const lang = getUserLanguage().toLowerCase();
    const strings =
      translations[lang] || translations[lang.split("-")[0]] || en;
    document.querySelectorAll("[data-i18n]").forEach((node) => {
      const key = node.getAttribute("data-i18n");
      const value = strings[key] || en[key];
      if (value != null) node.textContent = value;
    });
  }

  function getBookId() {
    return window.location.pathname.split("/").pop();
  }

  function getInnerHtml(node, query) {
    const queryNode = node.querySelector(query);
    return queryNode != null ? queryNode.innerHTML : "";
  }

  function generateLibraryHtml(library) {
    const title = getInnerHtml(library, "title");
    const divTag = document.createElement("div");
    divTag.setAttribute("class", "library");
    divTag.innerHTML = `<div class="library__title">${title}</div>`;
    return divTag;
  }

  async function loadAndDisplayLibraries(bookId) {
    const url = `/catalog/v2/entries/${encodeURIComponent(bookId)}/holding_libraries`;
    const resp = await fetch(url);
    if (!resp.ok) {
      console.error(`Failed to fetch holdings: ${resp.status}`);
      return;
    }
    const data = new window.DOMParser().parseFromString(await resp.text(), "application/xml");
    const holdings = data.querySelectorAll("entry");

    const container = document.querySelector(".libraries__list");
    if (!container) return;

    if (!holdings.length) {
      container.innerHTML = "<p>No holdings found.</p>";
      return;
    }

    holdings.forEach((holding) => {
      container.appendChild(generateLibraryHtml(holding));
    });
  }

  window.addEventListener("DOMContentLoaded", async () => {
    await loadTranslations();
    translatePage();
    const bookId = getBookId();
    if (!bookId) {
      console.error("No book ID found in URL");
      return;
    }
    loadAndDisplayLibraries(bookId);
  });

  // The kiwix wrapper changes the UI language by writing to localStorage;
  // re-translate live when that happens in another same-origin document.
  window.addEventListener("storage", (event) => {
    if (event.key === "userlang") {
      translatePage();
    }
  });
})();
