(function () {
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

  window.addEventListener("DOMContentLoaded", () => {
    const bookId = getBookId();
    if (!bookId) {
      console.error("No book ID found in URL");
      return;
    }
    loadAndDisplayLibraries(bookId);
  });
})();
