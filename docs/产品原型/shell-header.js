/**
 * 产品原型 · 共用顶栏（仅「设置」入口）。
 * 在 common.js 初始化前加载；由 mountShellHeader() 挂载到 .main 内、.workspace 之前。
 */
(function () {
  function mountShellHeader(mainEl) {
    if (!mainEl || mainEl.querySelector(".shell-header")) return;
    var header = document.createElement("header");
    header.className = "topbar shell-header";
    header.setAttribute("aria-label", "顶栏");
    var btn = document.createElement("button");
    btn.type = "button";
    btn.className = "btn";
    btn.id = "topbar-settings";
    btn.textContent = "设置";
    header.appendChild(btn);
    var workspace = mainEl.querySelector(".workspace");
    if (workspace) mainEl.insertBefore(header, workspace);
    else mainEl.prepend(header);
  }

  window.mountShellHeader = mountShellHeader;
})();
