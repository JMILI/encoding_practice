(function () {
  const DEFAULT_PORT = "3000";

  function serverBase() {
    if (window.location.protocol === "file:") {
      const port = localStorage.getItem("protocol_tutorial_port") || DEFAULT_PORT;
      return "http://127.0.0.1:" + port;
    }
    return window.location.origin;
  }

  function currentPath() {
    if (window.location.protocol === "file:") {
      return window.location.pathname.endsWith("tet.html") ? "/guide" : "/";
    }
    const p = window.location.pathname;
    if (p === "/guide" || p === "/guide/") return "/guide";
    return "/";
  }

  const base = serverBase();
  const here = currentPath();

  document.querySelectorAll("a[data-path]").forEach(function (a) {
    const path = a.getAttribute("data-path");
    a.href = base + path;
    a.classList.toggle("active", path === here);
  });

  document.querySelectorAll(".route-link").forEach(function (el) {
    const path = el.getAttribute("data-path") || "/";
    const url = base + path;
    if (el.tagName === "A") {
      el.href = url;
      el.textContent = url;
    } else {
      el.textContent = url;
    }
    el.classList.toggle("active", path === here);
  });

  var fileHint = document.getElementById("file-protocol-hint");
  if (fileHint && window.location.protocol === "file:") {
    fileHint.hidden = false;
  }
})();
