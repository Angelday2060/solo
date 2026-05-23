(function () {
  /** 合法 hash 视图 id（顶栏不分页标题，仅侧栏切换）。 */
  const VALID_VIEW_IDS = new Set([
    "diary",
    "ledger-stat",
    "ledger-list",
    "ledger-detail",
    "notes",
    "schedule",
    "settings",
  ]);

  /** 列表示意数据：与 #ledger-tbody 初始行一致；删除时同步删行与键。 */
  const LEDGER_RECORDS = {
    "demo-1": { amount: "88.00", currency: "cny", direction: "out", category: "ex1餐饮", date: "2026-05-12", note: "便利店" },
    "demo-2": { amount: "1200", currency: "cny", direction: "in", category: "", date: "2026-05-08", note: "工资" },
    "demo-3": { amount: "350", currency: "jpy", direction: "out", category: "ex2交通", date: "2026-05-14", note: "Suica" },
  };

  /** 日记左侧树示意数据（实现侧来自 diary_list_by_day 等）。日期均为本地日历日 YYYY-MM-DD；`id` 对应 DiaryEntry.id。 */
  const DIARY_TREE_SAMPLES = [
    { id: "demo-de-1", date: "2026-05-19", preview: "整理了产品原型里的日记时间轴……" },
    { id: "demo-de-2", date: "2026-05-14", preview: "雨天，翻了会儿文档。" },
    { id: "demo-de-3", date: "2026-05-08", preview: "" },
    { id: "demo-de-4", date: "2026-04-22", preview: "月度复盘草稿（未完成）。" },
    { id: "demo-de-5", date: "2025-12-31", preview: "跨年小结。" },
    { id: "demo-de-6", date: "2025-03-01", preview: "春游计划备忘。" },
  ];

  const WEEKDAY_ZH = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

  function parseISODateLocal(iso) {
    const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso || "");
    if (!m) return null;
    const y = Number(m[1]);
    const mo = Number(m[2]);
    const d = Number(m[3]);
    if (!y || mo < 1 || mo > 12 || d < 1 || d > 31) return null;
    return { y, m: mo, d };
  }

  function diaryWeekdayZh(iso) {
    const p = parseISODateLocal(iso);
    if (!p) return "";
    const dt = new Date(p.y, p.m - 1, p.d);
    return WEEKDAY_ZH[dt.getDay()] || "";
  }

  function diaryEditorFullLabel(iso) {
    const p = parseISODateLocal(iso);
    if (!p) return "—";
    const wd = diaryWeekdayZh(iso);
    return p.y + "年" + p.m + "月" + p.d + "日" + (wd ? " " + wd : "");
  }

  function groupDiarySamplesDesc(samples) {
    const valid = samples.filter((x) => parseISODateLocal(x.date));
    valid.sort((a, b) => (a.date < b.date ? 1 : a.date > b.date ? -1 : 0));

    /** @type {Map<number, Map<number, typeof samples>>} */
    const byYear = new Map();
    valid.forEach((row) => {
      const p = parseISODateLocal(row.date);
      if (!p) return;
      if (!byYear.has(p.y)) byYear.set(p.y, new Map());
      const byMonth = byYear.get(p.y);
      if (!byMonth.has(p.m)) byMonth.set(p.m, []);
      byMonth.get(p.m).push(row);
    });

    const years = Array.from(byYear.keys()).sort((a, b) => b - a);
    return years.map((y) => ({
      year: y,
      months: Array.from(byYear.get(y).keys())
        .sort((a, b) => b - a)
        .map((mo) => ({
          month: mo,
          days: byYear.get(y).get(mo),
        })),
    }));
  }

  /** 列表排序：日期新的在前；同一日内保持数组原有顺序（新建的 unshift 会更靠前）。 */
  function getNewestDiaryEntry(samples) {
    const valid = samples.filter((x) => parseISODateLocal(x.date));
    valid.sort((a, b) => (a.date < b.date ? 1 : a.date > b.date ? -1 : 0));
    return valid.length ? valid[0] : null;
  }

  function newestDiaryIso(samples) {
    const e = getNewestDiaryEntry(samples);
    return e ? e.date : "";
  }

  function diaryTodayISO() {
    const d = new Date();
    return (
      d.getFullYear() +
      "-" +
      String(d.getMonth() + 1).padStart(2, "0") +
      "-" +
      String(d.getDate()).padStart(2, "0")
    );
  }

  function syncDiaryChromeFromIso(iso) {
    if (!parseISODateLocal(iso)) return;
    const label = document.getElementById("diary-editor-date-label");
    if (label) label.textContent = diaryEditorFullLabel(iso);
    const metaDate = document.getElementById("diary-meta-date");
    if (metaDate) metaDate.value = iso;
  }

  function clearDiaryMetaOptionalFields() {
    const w = document.getElementById("diary-meta-weather");
    const m = document.getElementById("diary-meta-mood");
    if (w) w.value = "";
    if (m) m.value = "";
  }

  /** 原型：插入今日草稿并选中；真实实现对应 diary_create_entry（或等价 API）。 */
  function startNewDiary() {
    const iso = diaryTodayISO();
    const id = "draft-" + Date.now().toString(36) + "-" + Math.floor(Math.random() * 1e4).toString(36);
    DIARY_TREE_SAMPLES.unshift({ id, date: iso, preview: "（新建草稿）" });
    renderDiaryTree();
    clearDiaryMetaOptionalFields();
    const root = document.getElementById("diary-tree-root");
    const btn = root && root.querySelector(".diary-tree-day[data-entry-id=\"" + id + "\"]");
    if (btn) {
      root.querySelectorAll(".diary-tree-day.is-selected").forEach((el) => el.classList.remove("is-selected"));
      btn.classList.add("is-selected");
      btn.focus();
      syncDiaryChromeFromIso(iso);
    }
  }

  function renderDiaryTree() {
    const root = document.getElementById("diary-tree-root");
    if (!root) return;

    root.innerHTML = "";
    const grouped = groupDiarySamplesDesc(DIARY_TREE_SAMPLES);
    const newestEntry = getNewestDiaryEntry(DIARY_TREE_SAMPLES);
    const newestIso = newestEntry ? newestEntry.date : "";
    const newestEntryId = newestEntry ? newestEntry.id : "";
    const newestParts = parseISODateLocal(newestIso);
    const newestYear = newestParts ? newestParts.y : null;
    const newestMonth = newestParts ? newestParts.m : null;

    grouped.forEach(({ year, months }) => {
      const yearOpen = newestYear != null && year === newestYear;
      const yearNode = document.createElement("div");
      yearNode.className = "diary-tree-year diary-tree-node" + (yearOpen ? " open" : "");
      yearNode.setAttribute("role", "treeitem");
      yearNode.setAttribute("aria-expanded", yearOpen ? "true" : "false");

      const yearBtn = document.createElement("button");
      yearBtn.type = "button";
      yearBtn.className = "diary-tree-toggle";
      yearBtn.setAttribute("aria-expanded", yearOpen ? "true" : "false");
      const yChev = document.createElement("span");
      yChev.className = "diary-tree-chevron";
      yChev.setAttribute("aria-hidden", "true");
      const yLab = document.createElement("span");
      yLab.className = "diary-tree-label";
      yLab.textContent = year + "年";
      yearBtn.appendChild(yChev);
      yearBtn.appendChild(yLab);

      const yearKids = document.createElement("div");
      yearKids.className = "diary-tree-children";
      yearKids.setAttribute("role", "group");

      months.forEach(({ month, days }) => {
        const monthOpen = yearOpen && newestMonth != null && month === newestMonth;
        const monthNode = document.createElement("div");
        monthNode.className = "diary-tree-month diary-tree-node" + (monthOpen ? " open" : "");
        monthNode.setAttribute("role", "treeitem");
        monthNode.setAttribute("aria-expanded", monthOpen ? "true" : "false");

        const moBtn = document.createElement("button");
        moBtn.type = "button";
        moBtn.className = "diary-tree-toggle";
        moBtn.setAttribute("aria-expanded", monthOpen ? "true" : "false");
        const moChev = document.createElement("span");
        moChev.className = "diary-tree-chevron";
        moChev.setAttribute("aria-hidden", "true");
        const moLab = document.createElement("span");
        moLab.className = "diary-tree-label";
        moLab.textContent = month + "月";
        moBtn.appendChild(moChev);
        moBtn.appendChild(moLab);

        const moKids = document.createElement("div");
        moKids.className = "diary-tree-children";
        moKids.setAttribute("role", "group");

        const daysWrap = document.createElement("div");
        daysWrap.className = "diary-tree-days";

        days.forEach((row) => {
          const iso = row.date;
          const p = parseISODateLocal(iso);
          const dayBtn = document.createElement("button");
          dayBtn.type = "button";
          dayBtn.className = "diary-tree-day";
          dayBtn.setAttribute("data-entry-id", row.id);
          dayBtn.setAttribute("data-date", iso);
          dayBtn.setAttribute("role", "treeitem");
          const main = document.createElement("span");
          main.className = "diary-tree-day-main";
          main.textContent = p ? p.d + "日" : iso;
          const sub = document.createElement("span");
          sub.className = "diary-tree-day-sub";
          const wd = diaryWeekdayZh(iso);
          const pv = (row.preview || "").trim();
          sub.textContent = wd ? wd + (pv ? " · " + pv : "") : pv || "（无摘录）";
          dayBtn.appendChild(main);
          dayBtn.appendChild(sub);
          daysWrap.appendChild(dayBtn);
        });

        moKids.appendChild(daysWrap);
        monthNode.appendChild(moBtn);
        monthNode.appendChild(moKids);
        yearKids.appendChild(monthNode);
      });

      yearNode.appendChild(yearBtn);
      yearNode.appendChild(yearKids);
      root.appendChild(yearNode);
    });

    if (newestIso) syncDiaryChromeFromIso(newestIso);
    const firstBtn =
      newestEntryId && root.querySelector(".diary-tree-day[data-entry-id=\"" + newestEntryId + "\"]");
    if (firstBtn) firstBtn.classList.add("is-selected");
  }

  function bindDiaryTree() {
    const root = document.getElementById("diary-tree-root");
    if (!root) return;

    root.addEventListener("click", (e) => {
      const t = e.target;
      if (!(t instanceof HTMLElement)) return;

      const toggle = t.closest(".diary-tree-toggle");
      if (toggle) {
        const node = toggle.closest(".diary-tree-node");
        if (!node || !root.contains(node)) return;
        const open = !node.classList.contains("open");
        node.classList.toggle("open", open);
        toggle.setAttribute("aria-expanded", open ? "true" : "false");
        node.setAttribute("aria-expanded", open ? "true" : "false");
        return;
      }

      const dayBtn = t.closest(".diary-tree-day");
      if (!dayBtn || !root.contains(dayBtn)) return;
      const iso = dayBtn.getAttribute("data-date");
      if (!iso) return;

      root.querySelectorAll(".diary-tree-day.is-selected").forEach((el) => el.classList.remove("is-selected"));
      dayBtn.classList.add("is-selected");

      syncDiaryChromeFromIso(iso);
    });
  }

  function bindDiaryMetaDatePicker() {
    const metaDate = document.getElementById("diary-meta-date");
    if (!metaDate) return;
    metaDate.addEventListener("change", () => {
      const iso = metaDate.value;
      syncDiaryChromeFromIso(iso);
    });
  }

  const DEFAULT_LEDGER_CATEGORIES = [
    { id: "ex1餐饮", name: "餐饮（示例）" },
    { id: "ex2交通", name: "交通（示例）" },
  ];
  let ledgerCategories = DEFAULT_LEDGER_CATEGORIES.map((x) => ({ ...x }));

  function newCategoryId() {
    return "cat-" + Date.now().toString(36) + "-" + Math.floor(Math.random() * 1e5).toString(36);
  }

  function categoryNameById(cid) {
    if (!cid) return "";
    const f = ledgerCategories.find((c) => c.id === cid);
    return f ? f.name : cid;
  }

  function renderSettingsCategoryList() {
    const ul = document.getElementById("settings-category-list");
    if (!ul) return;
    ul.innerHTML = "";
    ledgerCategories.forEach((c) => {
      const li = document.createElement("li");
      li.className = "category-list-item";
      const nameSpan = document.createElement("span");
      nameSpan.className = "category-list-name";
      nameSpan.textContent = c.name;
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "btn btn-sm btn-danger category-remove-btn";
      btn.setAttribute("data-id", c.id);
      btn.setAttribute("aria-label", "删除分类 " + c.name);
      btn.textContent = "删除";
      li.appendChild(nameSpan);
      li.appendChild(btn);
      ul.appendChild(li);
    });
  }

  function syncLedgerCategorySelects() {
    const detailSel = document.getElementById("ledger-f-category");
    const statSel = document.getElementById("stat-filter-category");
    if (detailSel) {
      const cur = detailSel.value;
      detailSel.innerHTML = "";
      const empty = document.createElement("option");
      empty.value = "";
      empty.textContent = "未分类";
      detailSel.appendChild(empty);
      ledgerCategories.forEach((c) => {
        const o = document.createElement("option");
        o.value = c.id;
        o.textContent = c.name;
        detailSel.appendChild(o);
      });
      if ([...detailSel.options].some((o) => o.value === cur)) detailSel.value = cur;
      else detailSel.value = "";
    }
    if (statSel) {
      const cur = statSel.value;
      statSel.innerHTML = "";
      const all = document.createElement("option");
      all.value = "all";
      all.textContent = "全部分类";
      statSel.appendChild(all);
      ledgerCategories.forEach((c) => {
        const o = document.createElement("option");
        o.value = c.id;
        o.textContent = c.name;
        statSel.appendChild(o);
      });
      if (cur === "all" || [...statSel.options].some((o) => o.value === cur)) statSel.value = cur;
      else statSel.value = "all";
    }
  }

  function clearCategoryFromLedgerRecords(catId) {
    Object.keys(LEDGER_RECORDS).forEach((k) => {
      if (LEDGER_RECORDS[k].category === catId) LEDGER_RECORDS[k].category = "";
    });
  }

  function refreshLedgerListCategoryColumn() {
    const tb = document.getElementById("ledger-tbody");
    if (!tb) return;
    tb.querySelectorAll("tr[data-id]").forEach((tr) => {
      const id = tr.getAttribute("data-id");
      const rec = LEDGER_RECORDS[id];
      if (!rec) return;
      const tds = tr.querySelectorAll("td");
      if (tds.length < 5) return;
      tds[4].textContent = rec.category ? categoryNameById(rec.category) : "—";
    });
  }

  function bindSettingsCategories() {
    const addBtn = document.getElementById("settings-category-add");
    const input = document.getElementById("settings-category-input");
    const list = document.getElementById("settings-category-list");
    if (!addBtn || !input || !list) return;

    function tryAdd() {
      const name = input.value.trim();
      if (!name) return;
      if (ledgerCategories.some((c) => c.name === name)) return;
      ledgerCategories.push({ id: newCategoryId(), name: name });
      input.value = "";
      renderSettingsCategoryList();
      syncLedgerCategorySelects();
      refreshLedgerListCategoryColumn();
    }

    addBtn.addEventListener("click", tryAdd);
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        tryAdd();
      }
    });

    list.addEventListener("click", (e) => {
      const t = e.target;
      if (!(t instanceof HTMLElement)) return;
      const btn = t.closest(".category-remove-btn");
      if (!btn) return;
      const id = btn.getAttribute("data-id");
      if (!id) return;
      if (!confirm("确认删除该分类？已关联流水的分类字段将清空（示意）。")) return;
      ledgerCategories = ledgerCategories.filter((c) => c.id !== id);
      clearCategoryFromLedgerRecords(id);
      renderSettingsCategoryList();
      syncLedgerCategorySelects();
      refreshLedgerListCategoryColumn();
      const { path, query } = splitHash();
      if (normalizeViewId(path) === "ledger-detail") applyLedgerDetailMode(query);
    });
  }

  function isLedgerView(id) {
    return id === "ledger-stat" || id === "ledger-list" || id === "ledger-detail";
  }

  function normalizeViewId(path) {
    const id = (path || "").replace(/^#/, "").split("?")[0];
    if (VALID_VIEW_IDS.has(id)) return id;
    return "diary";
  }

  function splitHash() {
    let raw = location.hash.slice(1).trim();
    if (!raw) raw = "diary";
    const qi = raw.indexOf("?");
    const path = qi >= 0 ? raw.slice(0, qi) : raw;
    const qs = qi >= 0 ? raw.slice(qi + 1) : "";
    return { path, query: new URLSearchParams(qs) };
  }

  function buildHash(viewId, queryObj) {
    const id = normalizeViewId(viewId);
    let h = "#" + id;
    if (queryObj && Object.keys(queryObj).length > 0) {
      const p = new URLSearchParams();
      Object.entries(queryObj).forEach(([k, v]) => {
        if (v != null && String(v) !== "") p.set(k, String(v));
      });
      const s = p.toString();
      if (s) h += "?" + s;
    }
    return h;
  }

  function applyNavState(viewId) {
    const group = document.getElementById("nav-ledger-group");
    const toggle = document.getElementById("nav-ledger-toggle");
    const ledgerActive = isLedgerView(viewId);
    if (group) {
      group.classList.toggle("active", ledgerActive);
      if (ledgerActive) {
        group.classList.add("open");
        if (toggle) toggle.setAttribute("aria-expanded", "true");
      } else {
        group.classList.remove("open");
        if (toggle) toggle.setAttribute("aria-expanded", "false");
      }
    }

    document.querySelectorAll(".nav-item[data-view]").forEach((el) => {
      const v = el.getAttribute("data-view");
      el.classList.toggle("active", v === viewId);
    });
  }

  function applyWorkspace(viewId) {
    document.querySelectorAll(".content-view").forEach((panel) => {
      panel.classList.toggle("is-active", panel.getAttribute("data-view") === viewId);
    });
  }

  function getLedgerFormEls() {
    return {
      amount: document.getElementById("ledger-f-amount"),
      currency: document.getElementById("ledger-f-currency"),
      direction: document.getElementById("ledger-f-direction"),
      category: document.getElementById("ledger-f-category"),
      date: document.getElementById("ledger-tx-date"),
      note: document.getElementById("ledger-f-note"),
    };
  }

  function fillLedgerForm(rec) {
    const els = getLedgerFormEls();
    if (!els.amount) return;
    els.amount.value = rec.amount != null ? rec.amount : "";
    els.currency.value = rec.currency || "cny";
    els.direction.value = rec.direction || "out";
    els.category.value = rec.category != null ? rec.category : "";
    els.date.value = rec.date || "";
    els.note.value = rec.note != null ? rec.note : "";
  }

  function clearLedgerForm() {
    fillLedgerForm({
      amount: "",
      currency: "cny",
      direction: "out",
      category: "",
      date: "",
      note: "",
    });
  }

  function applyLedgerDetailMode(query) {
    const panelH = document.getElementById("ledger-detail-panel-h");
    const hint = document.getElementById("ledger-date-hint");
    const recordId = query.get("id");

    if (recordId) {
      if (panelH) panelH.textContent = "单笔流水 · 编辑";
      if (hint) hint.textContent = "编辑时保留原发生日期；保存由实现写入数据库（原型仅示意）。";
      const rec = LEDGER_RECORDS[recordId];
      fillLedgerForm(rec || {});
      if (!rec) {
        const d = getLedgerFormEls().date;
        if (d && !d.value) d.valueAsDate = new Date();
      }
    } else {
      if (panelH) panelH.textContent = "单笔流水 · 新建 · 发生日期默认今日";
      if (hint) hint.textContent = "新建时默认为今天。";
      clearLedgerForm();
      const d = getLedgerFormEls().date;
      if (d) d.valueAsDate = new Date();
    }
  }

  function applyViewFromHash() {
    const { path, query } = splitHash();
    const id = normalizeViewId(path);
    applyNavState(id);
    applyWorkspace(id);

    if (id === "ledger-stat" && typeof window.initLedgerChartsIfNeeded === "function") {
      window.initLedgerChartsIfNeeded();
    }
    if (id === "ledger-detail") {
      applyLedgerDetailMode(query);
    }
  }

  function navigateTo(viewId, queryObj) {
    const h = buildHash(viewId, queryObj);
    if (location.hash !== h) {
      location.hash = h;
    } else {
      applyViewFromHash();
    }
  }

  function syncLedgerSelectAll() {
    const master = document.getElementById("ledger-select-all");
    const checks = document.querySelectorAll("#ledger-tbody .ledger-row-check");
    if (!master) return;
    if (checks.length === 0) {
      master.checked = false;
      master.indeterminate = false;
      return;
    }
    const n = Array.from(checks).filter((c) => c.checked).length;
    master.checked = n === checks.length && n > 0;
    master.indeterminate = n > 0 && n < checks.length;
  }

  function removeLedgerRowFromDom(id) {
    const tb = document.getElementById("ledger-tbody");
    const tr = tb && tb.querySelector("tr[data-id=\"" + id + "\"]");
    if (tr) tr.remove();
    delete LEDGER_RECORDS[id];
    syncLedgerSelectAll();
  }

  function maybeLeaveDetailIfEditing(id) {
    const { path, query } = splitHash();
    if (normalizeViewId(path) === "ledger-detail" && query.get("id") === id) {
      navigateTo("ledger-list");
    }
  }

  function bindLedgerListActions() {
    const tbody = document.getElementById("ledger-tbody");
    const master = document.getElementById("ledger-select-all");
    const batchBtn = document.getElementById("ledger-batch-delete");
    if (!tbody) return;

    if (master) {
      master.addEventListener("change", () => {
        tbody.querySelectorAll(".ledger-row-check").forEach((c) => {
          c.checked = master.checked;
        });
        master.indeterminate = false;
      });
    }

    tbody.addEventListener("change", (e) => {
      const t = e.target;
      if (t && t.classList && t.classList.contains("ledger-row-check")) syncLedgerSelectAll();
    });

    tbody.addEventListener("click", (e) => {
      const t = e.target;
      if (!(t instanceof HTMLElement)) return;
      const edit = t.closest(".ledger-btn-edit");
      const del = t.closest(".ledger-btn-del");
      if (edit) {
        const id = edit.getAttribute("data-id");
        if (id) navigateTo("ledger-detail", { id: id });
        return;
      }
      if (del) {
        const id = del.getAttribute("data-id");
        if (!id) return;
        if (!confirm("确认删除该条账目？")) return;
        removeLedgerRowFromDom(id);
        maybeLeaveDetailIfEditing(id);
      }
    });

    if (batchBtn) {
      batchBtn.addEventListener("click", () => {
        const ids = Array.from(tbody.querySelectorAll(".ledger-row-check:checked"))
          .map((c) => c.getAttribute("data-id"))
          .filter(Boolean);
        if (ids.length === 0) return;
        if (!confirm("确认删除选中的 " + ids.length + " 条账目？")) return;
        const { path, query } = splitHash();
        const detailId = normalizeViewId(path) === "ledger-detail" ? query.get("id") : null;
        ids.forEach((id) => removeLedgerRowFromDom(id));
        if (detailId && ids.indexOf(detailId) >= 0) navigateTo("ledger-list");
      });
    }
  }

  document.addEventListener("DOMContentLoaded", () => {
    const mainEl = document.querySelector(".main");
    if (typeof window.mountShellHeader === "function") {
      window.mountShellHeader(mainEl);
    }

    document.querySelectorAll(".nav-item[data-view]").forEach((el) => {
      el.addEventListener("click", () => navigateTo(el.getAttribute("data-view")));
    });

    const ledgerGroup = document.getElementById("nav-ledger-group");
    const ledgerToggle = document.getElementById("nav-ledger-toggle");
    if (ledgerToggle && ledgerGroup) {
      ledgerToggle.addEventListener("click", () => {
        const open = !ledgerGroup.classList.contains("open");
        ledgerGroup.classList.toggle("open", open);
        ledgerToggle.setAttribute("aria-expanded", open ? "true" : "false");
      });
    }

    document.getElementById("topbar-settings")?.addEventListener("click", () => navigateTo("settings"));

    document.getElementById("ledger-btn-new")?.addEventListener("click", () => navigateTo("ledger-detail"));
    document.getElementById("ledger-filter-apply")?.addEventListener("click", () =>
      window.alert("原型示意：检索在实现中由 ledger_list_transactions 执行。"),
    );

    document.getElementById("ledger-form-cancel")?.addEventListener("click", () => navigateTo("ledger-list"));
    document.getElementById("ledger-form-save")?.addEventListener("click", () =>
      window.alert("原型示意：保存在实现中调用 ledger_create/update_transaction。"),
    );

    window.addEventListener("hashchange", () => applyViewFromHash());

    const { path } = splitHash();
    const initial = normalizeViewId(path);
    if (!location.hash || location.hash === "#") {
      history.replaceState(null, "", location.pathname + location.search + buildHash(initial));
    }
    bindLedgerListActions();
    renderSettingsCategoryList();
    syncLedgerCategorySelects();
    refreshLedgerListCategoryColumn();
    bindSettingsCategories();
    renderDiaryTree();
    bindDiaryTree();
    applyViewFromHash();
  });
})();
