(function () {
  function getBookId() {
    return window.location.pathname.split("/").pop();
  }

  function getInnerHtml(node, query) {
    const queryNode = node.querySelector(query);
    return queryNode != null ? queryNode.innerHTML : "";
  }

  function generateHoldingHtml(holding) {
    const title = getInnerHtml(holding, "title");
    const divTag = document.createElement("div");
    divTag.setAttribute("class", "holding");
    divTag.innerHTML = `<div class="holding__title">${title}</div>`;
    return divTag;
  }

  async function loadAndDisplayHoldings(bookId) {
    const url = `/catalog/v2/entries/${encodeURIComponent(bookId)}/holdings`;
    const resp = await fetch(url);
    if (!resp.ok) {
      console.error(`Failed to fetch holdings: ${resp.status}`);
      return;
    }
    const data = new window.DOMParser().parseFromString(await resp.text(), "application/xml");
    const holdings = data.querySelectorAll("entry");

    const container = document.querySelector(".holdings__list");
    if (!container) return;

    if (!holdings.length) {
      container.innerHTML = "<p>No holdings found.</p>";
      return;
    }

    holdings.forEach((holding) => {
      container.appendChild(generateHoldingHtml(holding));
    });
  }

  window.addEventListener("DOMContentLoaded", () => {
    const bookId = getBookId();
    if (!bookId) {
      console.error("No book ID found in URL");
      return;
    }
    loadAndDisplayHoldings(bookId);
  });
})();
