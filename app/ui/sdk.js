/*
 * Console Launcher theme SDK (v1).
 *
 * Loads the active theme (HTML + CSS + JS from its folder) into this page
 * and exposes `window.Launcher`, the only API a theme needs.
 * Full reference: docs/prototype/THEMES.en.md / THEMES.fr.md.
 */
(() => {
  "use strict";

  const T = window.__TAURI__;
  const invoke = T.core.invoke;
  const listen = T.event.listen;
  const fileUrl = (p) => (p ? T.core.convertFileSrc(p) : null);

  const navHandlers = new Set();
  const sessionHandlers = new Set();
  const padHandlers = new Set();
  let overlay = null; // built-in settings panel, when open
  let sessionRunning = false;

  const KEYS = {
    ArrowUp: "up", ArrowDown: "down", ArrowLeft: "left", ArrowRight: "right",
    Enter: "confirm", " ": "confirm", Escape: "back", Backspace: "back",
    f: "favorite", F: "favorite", Tab: "menu", m: "menu", M: "menu",
    PageUp: "pageLeft", PageDown: "pageRight", q: "pageLeft", e: "pageRight",
  };

  function emitNav(action) {
    if (sessionRunning) return;
    if (overlay) return overlay.nav(action);
    for (const h of navHandlers) {
      try { h(action); } catch (e) { console.error(e); }
    }
  }

  document.addEventListener("keydown", (e) => {
    const t = e.target;
    if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable)) {
      if (e.key === "Escape") t.blur();
      return;
    }
    if (e.key === "F11") { e.preventDefault(); Launcher.toggleFullscreen(); return; }
    if (e.key === "F5") { e.preventDefault(); location.reload(); return; }
    const action = KEYS[e.key];
    if (action) { e.preventDefault(); emitNav(action); }
  });

  listen("nav", (e) => emitNav(e.payload));
  listen("pad", (e) => padHandlers.forEach((h) => h(e.payload)));
  listen("session", (e) => {
    sessionRunning = e.payload.state === "started";
    sessionHandlers.forEach((h) => h(e.payload));
  });

  const optionValues = {};

  function applyOptions(theme) {
    const root = document.documentElement;
    for (const opt of theme.manifest.options || []) {
      const v = opt.id in theme.user_options ? theme.user_options[opt.id] : opt.default;
      optionValues[opt.id] = v;
      if (typeof v === "string" || typeof v === "number") root.style.setProperty(`--opt-${opt.id}`, String(v));
      root.dataset[`opt${opt.id[0].toUpperCase()}${opt.id.slice(1)}`] = String(v);
    }
  }

  const Launcher = {
    sdkVersion: 1,
    /** Active theme: { id, dir, manifest, user_options }. */
    theme: null,

    /** Navigation actions: up, down, left, right, confirm, back, menu, favorite, pageLeft, pageRight. */
    onNav(cb) { navHandlers.add(cb); return () => navHandlers.delete(cb); },
    /** Session events: {state: "started"|"ended"|"failed", report?, error?}. */
    onSession(cb) { sessionHandlers.add(cb); return () => sessionHandlers.delete(cb); },
    /** Controller connected / disconnected: {name, connected}. */
    onPad(cb) { padHandlers.add(cb); return () => padHandlers.delete(cb); },

    /** { systems: [...], games: [...] }, each game gets `coverUrl`. */
    async getLibrary() {
      const lib = await invoke("get_library");
      for (const g of lib.games) g.coverUrl = fileUrl(g.cover);
      return lib;
    },
    /** Emulator, quality rung, notes for a game (or {ready:false, error}). */
    gameInfo(id) { return invoke("game_info", { id }); },
    /** Starts a game; resolves when the emulator closes, with the session report. */
    play(id) { return invoke("play", { id }); },
    toggleFavorite(id) { return invoke("toggle_favorite", { id }); },
    /** Fix a quality rung by hand (0 = lightest). */
    setQuality(id, step) { return invoke("set_quality", { id, step }); },
    getSettings() { return invoke("get_settings"); },
    saveSettings(settings) { return invoke("save_settings", { settings }); },
    hardware(refresh = false) { return invoke("hardware", { refresh }); },
    listThemes() { return invoke("list_themes"); },
    async setTheme(name) { await invoke("set_theme", { name }); location.reload(); },

    /** Current value of a theme option declared in theme.json. */
    option(id) { return optionValues[id]; },
    async setOption(id, value) {
      await invoke("set_theme_option", { key: id, value });
      Launcher.theme.user_options[id] = value;
      applyOptions(Launcher.theme);
    },
    /** URL of a file inside the theme folder (images, fonts, sounds). */
    asset(relPath) { return fileUrl(`${Launcher.theme.dir}/${relPath}`); },

    async toggleFullscreen() {
      const w = T.window.getCurrentWindow();
      await w.setFullscreen(!(await w.isFullscreen()));
    },
    quit() { return invoke("quit"); },

    /** Opens the built-in settings panel (themes may also build their own). */
    openSettings() { openSettings(); },
  };
  window.Launcher = Launcher;

  /* ---------- Built-in settings panel ---------- */

  async function openSettings() {
    if (overlay) return;
    const [settings, themes, hw] = await Promise.all([
      Launcher.getSettings(), Launcher.listThemes(), Launcher.hardware(false).catch(() => null),
    ]);
    const el = document.createElement("div");
    el.className = "cl-settings";
    const gpu = hw && hw.gpus && hw.gpus[0];
    const row = (id, label, value) =>
      `<button class="cl-row" data-id="${id}"><span>${label}</span><b>${value}</b></button>`;
    const render = () => {
      el.innerHTML = `
        <div class="cl-panel" role="dialog" aria-label="Settings">
          <h2>Réglages · Settings</h2>
          ${row("theme", "Thème / Theme", themes.find((t) => t.id === settings.theme)?.name || settings.theme)}
          ${row("tune", "Optimisation automatique / Auto-optimisation", settings.perf.auto_tune ? "ON" : "OFF")}
          ${row("fullscreen", "Plein écran / Fullscreen", "F11")}
          <div class="cl-block"><span>Dossiers de jeux / Game folders</span>
            ${(settings.rom_dirs.length ? settings.rom_dirs : ["—"]).map((d) => `<code>${escapeHtml(String(d))}</code>`).join("")}
            <input id="cl-romdir" placeholder="Ajouter un dossier / Add a folder (path)"></div>
          ${row("adddir", "Ajouter ce dossier / Add this folder", "+")}
          <div class="cl-block"><span>Machine</span>
            <code>${hw ? escapeHtml(`${hw.cpu.brand} · ${hw.ram_mb} MB · ${gpu ? gpu.name : "GPU ?"} · score ${hw.bench ? hw.bench.score : "?"}`) : "—"}</code></div>
          ${row("rebench", "Refaire le test machine / Re-run hardware test", "↻")}
          ${row("quit", "Quitter / Quit", "⏻")}
          ${row("close", "Fermer / Close", "B")}
        </div>`;
      items = [...el.querySelectorAll(".cl-row")];
      items.forEach((b, i) => b.addEventListener("click", () => { index = i; activate(); }));
      focus();
    };
    let items = [];
    let index = 0;
    const focus = () => items.forEach((b, i) => b.classList.toggle("focused", i === index));
    const save = () => Launcher.saveSettings(settings);
    const activate = async () => {
      const id = items[index]?.dataset.id;
      if (id === "theme") {
        const i = themes.findIndex((t) => t.id === settings.theme);
        const next = themes[(i + 1) % themes.length];
        if (next) await Launcher.setTheme(next.id);
        return;
      }
      if (id === "tune") { settings.perf.auto_tune = !settings.perf.auto_tune; await save(); }
      if (id === "fullscreen") await Launcher.toggleFullscreen();
      if (id === "adddir") {
        const input = el.querySelector("#cl-romdir");
        const v = input.value.trim();
        if (!v) { input.focus(); return; }
        settings.rom_dirs.push(v); await save();
        document.dispatchEvent(new CustomEvent("launcher:library-changed"));
      }
      if (id === "rebench") { await Launcher.hardware(true); close(); return openSettings(); }
      if (id === "quit") return Launcher.quit();
      if (id === "close") return close();
      render();
    };
    const close = () => { el.remove(); overlay = null; };
    overlay = {
      nav(a) {
        if (a === "up") index = (index + items.length - 1) % items.length;
        else if (a === "down") index = (index + 1) % items.length;
        else if (a === "confirm") return activate();
        else if (a === "back" || a === "menu") return close();
        focus();
      },
    };
    document.body.appendChild(el);
    render();
  }

  function escapeHtml(s) {
    return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  }
  Launcher.escapeHtml = escapeHtml;

  /* ---------- Theme loading ---------- */

  function showError(message) {
    const el = document.createElement("div");
    el.className = "cl-error";
    el.innerHTML = `<b>Erreur du thème / Theme error</b><pre>${escapeHtml(String(message))}</pre>
      <p>Menu / Tab : réglages · settings — F5 : recharger · reload</p>`;
    document.body.appendChild(el);
    navHandlers.add((a) => { if (a === "menu") openSettings(); });
  }

  async function boot() {
    let theme;
    try {
      theme = await invoke("load_theme");
    } catch (e) {
      document.body.innerHTML = "";
      return showError(e);
    }
    Launcher.theme = theme;
    applyOptions(theme);
    const style = document.createElement("style");
    style.textContent = theme.css;
    document.head.appendChild(style);
    document.body.innerHTML = theme.html;
    document.body.dataset.theme = theme.id;
    try {
      // The theme script runs with the same rights as the page: only install
      // themes from people you trust.
      new Function("Launcher", theme.js)(Launcher);
    } catch (e) {
      console.error(e);
      showError(e.stack || e);
    }
  }

  window.addEventListener("error", (e) => console.error("theme:", e.error || e.message));
  boot();
})();
