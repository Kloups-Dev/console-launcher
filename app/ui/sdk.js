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

    /**
     * Opens the built-in settings panel (themes may also build their own).
     * `target` (optional): {emulator: id} or {page: "systems"|"emulators"|"folders"|"capture"}.
     */
    openSettings(target) { openSettings(target); },
  };
  window.Launcher = Launcher;

  /* ---------- Built-in settings panel ----------
   * Pages stacked like a console menu: up/down to move, A/Enter to choose,
   * left/right to change a value, B/Escape to go back. Everything can be
   * done with a controller; the file pickers also accept a mouse. */

  const L = (navigator.language || "fr").toLowerCase().startsWith("fr");
  const t = (fr, en) => (L ? fr : en);

  async function openSettings(target) {
    if (overlay) return;
    const el = document.createElement("div");
    el.className = "cl-settings";
    document.body.appendChild(el);

    let data = null;        // settings_overview
    let themes = [];
    let hw = null;
    const stack = [];       // pages: {id, arg, index}
    let toast = "";
    let toastTimer = null;
    let confirmRemove = null;

    const reload = async () => {
      [data, themes] = await Promise.all([invoke("settings_overview"), Launcher.listThemes()]);
    };
    const libraryChanged = () => document.dispatchEvent(new CustomEvent("launcher:library-changed"));
    const say = (msg) => {
      toast = msg;
      clearTimeout(toastTimer);
      toastTimer = setTimeout(() => { toast = ""; render(); }, 5000);
    };
    const run = async (fn, okMsg) => {
      try {
        const r = await fn();
        if (okMsg) say(typeof okMsg === "function" ? okMsg(r) : okMsg);
        return r;
      } catch (e) {
        say(`⚠ ${e}`);
        return undefined;
      }
    };
    const pick = (kind, title) => run(() => invoke("pick_path", { kind, title }));
    const emu = (id) => data.emulators.find((e) => e.id === id);
    const sysName = (id) => data.systems.find((c) => c.info.id === id)?.info.name || id;
    const onOff = (v) => (v ? "ON" : "OFF");
    // Readable names for the settings the launcher manages.
    const KEY_NAMES = [
      [/InternalResolution|ResolutionScale|upscale_multiplier/i, t("Résolution", "Resolution")],
      [/Renderer|GFXBackend|video_driver/i, t("Moteur graphique", "Graphics back-end")],
      [/run_ahead/i, t("Latence (run-ahead)", "Latency (run-ahead)")],
      [/video_threaded/i, t("Vidéo sur un fil séparé", "Threaded video")],
      [/frameskip/i, t("Saut d'images", "Frame skipping")],
      [/ShaderCompilation/i, t("Compilation des shaders", "Shader compilation")],
      [/CPUThread/i, t("Double cœur", "Dual core")],
    ];
    const keyName = (k) => {
      const hit = KEY_NAMES.find(([re]) => re.test(k));
      return hit ? `${hit[1]} <small>(${escapeHtml(k)})</small>` : escapeHtml(k);
    };
    const yes = '<span class="ok">✓</span>';
    const no = '<span class="bad">✕</span>';

    /* A page is a list of items: rows (focusable) and blocks (text). */
    const row = (label, value, action, extra = {}) => ({ type: "row", label, value, action, ...extra });
    const block = (title, html) => ({ type: "block", title, html });
    const back = () => row(t("Retour", "Back"), "B", () => pop());

    const pages = {
      main() {
        const theme = themes.find((x) => x.id === data.config.theme);
        const gpu = hw && hw.gpus && hw.gpus[0];
        const missing = data.systems.filter((c) => c.games > 0 && !c.options.find((o) => o.id === c.emulator)?.found).length;
        return {
          title: t("Réglages", "Settings"),
          items: [
            row(t("Consoles et émulateurs", "Consoles and emulators"), missing ? `${missing} ${no}` : yes, () => push("systems")),
            row(t("Émulateurs", "Emulators"), `${data.emulators.filter((e) => e.found).length} ${yes}`, () => push("emulators")),
            row(t("Dossiers de jeux et d'émulateurs", "Game and emulator folders"), `${data.config.rom_dirs.length + data.config.emulator_dirs.length}`, () => push("folders")),
            row(t("Optimisation automatique", "Automatic optimisation"), onOff(data.config.perf.auto_tune),
              () => prefs({ autoTune: !data.config.perf.auto_tune })),
            row(t("Optimisations système (priorité, GPU)", "System optimisations (priority, GPU)"), onOff(data.config.perf.system_tweaks),
              () => prefs({ systemTweaks: !data.config.perf.system_tweaks })),
            row(t("Mesure de fluidité", "Smoothness check"), data.capture.available ? yes : no, () => push("capture")),
            row(t("Thème", "Theme"), theme ? escapeHtml(theme.name) : data.config.theme, async () => {
              const i = themes.findIndex((x) => x.id === data.config.theme);
              const next = themes[(i + 1) % themes.length];
              if (next) await Launcher.setTheme(next.id);
            }),
            row(t("Langue des messages", "Message language"), data.config.language.startsWith("fr") ? "Français" : "English",
              () => prefs({ language: data.config.language.startsWith("fr") ? "en" : "fr" })),
            block(t("Machine", "Machine"), hw ? escapeHtml(`${hw.cpu.brand} · ${hw.ram_mb} MB · ${gpu ? gpu.name : "GPU ?"} · score ${hw.bench ? hw.bench.score : "?"}`) : "—"),
            row(t("Refaire le test machine", "Re-run hardware test"), "↻", async () => { say(t("Test en cours…", "Testing…")); render(); hw = await Launcher.hardware(true); say(t("Test terminé.", "Done.")); }),
            row(t("Plein écran", "Fullscreen"), "F11", () => Launcher.toggleFullscreen()),
            row(t("Quitter le lanceur", "Quit the launcher"), "⏻", () => Launcher.quit()),
            row(t("Fermer", "Close"), "B", () => close()),
          ],
        };
      },

      systems() {
        const items = [block("", t(
          "Choisis l'émulateur de chaque console (← → ou A). « Auto » prend le premier émulateur installé, dans l'ordre de préférence.",
          "Choose the emulator of each console (← → or A). \"Auto\" takes the first installed emulator, in order of preference."))];
        let shownEmpty = false;
        for (const c of data.systems) {
          if (c.games === 0 && !shownEmpty) {
            items.push(block(t("Consoles sans jeu", "Consoles without games"), ""));
            shownEmpty = true;
          }
          const cur = c.options.find((o) => o.id === c.emulator);
          const value = `${escapeHtml(cur ? cur.name : c.emulator)} ${cur && cur.found ? yes : no}${c.automatic ? ' <small>auto</small>' : ""}`;
          const choices = [null, ...c.options.map((o) => o.id)];
          const cycle = (d) => {
            const i = c.automatic ? 0 : Math.max(0, choices.indexOf(c.emulator));
            const next = choices[(i + d + choices.length) % choices.length];
            return run(() => invoke("set_system_emulator", { system: c.info.id, id: next }), () => {
              const o = c.options.find((x) => x.id === next);
              return next ? `${c.info.name} → ${o.name}` : `${c.info.name} → Auto`;
            }).then(async () => { await reload(); libraryChanged(); });
          };
          items.push(row(`${escapeHtml(c.info.name)} <small>${c.games} ${t("jeux", "games")}</small>`, value,
            () => cycle(1), { left: () => cycle(-1), right: () => cycle(1), dim: c.games === 0 }));
        }
        items.push(back());
        return { title: t("Consoles et émulateurs", "Consoles and emulators"), items };
      },

      emulators() {
        const list = [...data.emulators].sort((a, b) => (b.found - a.found) || a.name.localeCompare(b.name));
        const items = list.map((e) => row(
          `${escapeHtml(e.name)}${e.custom ? ` <small>${t("ajouté", "added")}</small>` : ""}`,
          `${e.user_settings.length ? `<small>${e.user_settings.length} ${t("réglage(s) perso", "own setting(s)")}</small> ` : ""}${e.found ? yes : no}`,
          () => push("emulator", e.id)));
        items.unshift(row(t("+ Ajouter un émulateur…", "+ Add an emulator…"), "", () => addEmulator()));
        if (data.profileErrors.length) {
          items.push(block(t("Fichiers de profil ignorés", "Profile files ignored"), data.profileErrors.map((e) => `<code>${escapeHtml(e)}</code>`).join("<br>")));
        }
        items.push(back());
        return { title: t("Émulateurs", "Emulators"), items };
      },

      emulator(id) {
        const e = emu(id);
        if (!e) return pages.emulators();
        const items = [];
        items.push(block(t("Exécutable", "Executable"), e.path
          ? `<code>${escapeHtml(e.path)}</code>${e.configured_path ? "" : ` <small>(${t("détecté", "detected")})</small>`}`
          : `${no} ${t("introuvable", "not found")}${e.configured_path ? `<br><code>${escapeHtml(e.configured_path)}</code>` : ""}`));
        items.push(row(t("Choisir l'exécutable…", "Choose the executable…"), "📂", async () => {
          const p = await pick("exe", e.name);
          if (!p) return;
          if (e.custom) await run(() => invoke("update_custom_emulator", { id, emulator: { ...data.config.custom_emulators[id], path: p } }), t("Chemin enregistré.", "Path saved."));
          else await run(() => invoke("set_emulator_path", { id, path: p }), t("Chemin enregistré.", "Path saved."));
          await reload(); libraryChanged();
        }));
        if (e.configured_path && !e.custom) {
          items.push(row(t("Revenir à la détection automatique", "Back to automatic detection"), "↺", async () => {
            await run(() => invoke("set_emulator_path", { id, path: null }), t("Détection automatique.", "Automatic detection."));
            await reload(); libraryChanged();
          }));
        }
        if (e.found) {
          items.push(row(t("Ouvrir l'émulateur (BIOS, manettes…)", "Open the emulator (BIOS, controllers…)"), "↗",
            () => run(() => invoke("open_emulator", { id }), t("Émulateur ouvert.", "Emulator opened."))));
        }
        items.push(block(t("Optimisation", "Optimisation"), e.tunes_settings
          ? (e.user_settings.length
            ? t(`Automatique, sauf ces réglages que tu as changés dans ${escapeHtml(e.name)} :`, `Automatic, except these settings you changed in ${escapeHtml(e.name)}:`) +
              `<br>${e.user_settings.map(keyName).join("<br>")}`
            : t("Automatique : le lanceur règle l'émulateur selon ta machine et la fluidité mesurée. Si tu changes un de ces réglages dans l'émulateur, il n'y touche plus.",
                "Automatic: the launcher sets the emulator up for your machine and the measured smoothness. If you change one of these settings in the emulator, it stops touching it."))
          : t("Le lanceur ne connaît pas encore les réglages de cet émulateur : il le lance directement, avec les optimisations système et la mesure de fluidité. Un fichier de profil peut lui apprendre ses réglages.",
              "The launcher does not know this emulator's settings yet: it starts it directly, with the system optimisations and the smoothness check. A profile file can teach it the settings.")));
        if (e.user_settings.length) {
          items.push(row(t("Réactiver l'optimisation automatique", "Re-enable automatic optimisation"), "⟳", async () => {
            await run(() => invoke("reset_emulator_tuning", { id }), t("Le lanceur reprend la main sur ces réglages au prochain lancement.", "The launcher takes these settings back at the next launch."));
            await reload();
          }));
        }
        items.push(block(t("Consoles", "Consoles"), e.systems.map(sysName).map(escapeHtml).join(", ") || "—"));
        if (e.custom) {
          const c = data.config.custom_emulators[id];
          items.push(row(t("Consoles lancées…", "Consoles it runs…"), `${c.systems.length}`, () => push("customSystems", id)));
          items.push({ type: "input", id: "cl-args", label: t("Arguments ({rom} = le jeu)", "Arguments ({rom} = the game)"),
            value: (c.args.length ? c.args : ["{rom}"]).join(" ") });
          items.push(row(t("Enregistrer les arguments", "Save the arguments"), "✓", async () => {
            const v = el.querySelector("#cl-args").value.trim();
            const args = v ? v.match(/"[^"]*"|\S+/g).map((a) => a.replace(/^"|"$/g, "")) : [];
            await run(() => invoke("update_custom_emulator", { id, emulator: { ...c, args } }), t("Arguments enregistrés.", "Arguments saved."));
            await reload();
          }));
          items.push(row(confirmRemove === id ? t("Confirmer la suppression", "Confirm removal") : t("Retirer cet émulateur", "Remove this emulator"), "🗑", async () => {
            if (confirmRemove !== id) { confirmRemove = id; return; }
            confirmRemove = null;
            await run(() => invoke("remove_custom_emulator", { id }), t("Émulateur retiré.", "Emulator removed."));
            await reload(); libraryChanged(); pop();
          }, { danger: true }));
        }
        if (e.homepage) items.push(block("", `<small>${escapeHtml(e.license || "")} · ${escapeHtml(e.homepage)}</small>`));
        items.push(back());
        return { title: escapeHtml(e.name), items };
      },

      customSystems(id) {
        const c = data.config.custom_emulators[id];
        const items = data.systems.map((s) => {
          const on = c.systems.includes(s.info.id);
          return row(escapeHtml(s.info.name), on ? yes : "—", async () => {
            const systems = on ? c.systems.filter((x) => x !== s.info.id) : [...c.systems, s.info.id];
            await run(() => invoke("update_custom_emulator", { id, emulator: { ...c, systems } }));
            await reload(); libraryChanged();
          });
        });
        items.push(back());
        return { title: t("Consoles lancées", "Consoles it runs"), items };
      },

      folders() {
        const items = [];
        const kindMsg = (k) => ({
          games: t("Dossier de jeux ajouté.", "Game folder added."),
          emulators: t("Dossier d'émulateurs ajouté : les émulateurs qu'il contient sont détectés.", "Emulator folder added: the emulators inside are detected."),
          emulator: t(`Exécutable reconnu : ${emu(k.id)?.name || k.id}.`, `Executable recognised: ${emu(k.id)?.name || k.id}.`),
          already_added: t("Déjà présent.", "Already there."),
        })[k.kind];
        const add = async (p) => {
          if (!p) return;
          await run(() => invoke("add_path", { path: p }), kindMsg);
          await reload(); libraryChanged();
        };
        const removable = (p) => row(`<code>${escapeHtml(p)}</code>`, confirmRemove === p ? t("Confirmer ?", "Confirm?") : "✕", async () => {
          if (confirmRemove !== p) { confirmRemove = p; return; }
          confirmRemove = null;
          await run(() => invoke("remove_path", { path: p }), t("Retiré.", "Removed."));
          await reload(); libraryChanged();
        }, { danger: confirmRemove === p });
        items.push(block(t("Dossiers de jeux", "Game folders"), data.config.rom_dirs.length ? "" : "—"));
        data.config.rom_dirs.forEach((p) => items.push(removable(p)));
        items.push(block(t("Dossiers d'émulateurs", "Emulator folders"), data.config.emulator_dirs.length ? "" : t("— (le dossier « emulators » à côté d'un dossier « roms » est cherché tout seul)", "— (an \"emulators\" folder next to a \"roms\" folder is searched by itself)")));
        data.config.emulator_dirs.forEach((p) => items.push(removable(p)));
        items.push(row(t("+ Ajouter un dossier…", "+ Add a folder…"), "📂", async () => add(await pick("folder", t("Dossier de jeux ou d'émulateurs", "Game or emulator folder")))));
        items.push({ type: "input", id: "cl-path", label: t("Ou colle un chemin", "Or paste a path"), value: "" });
        items.push(row(t("Ajouter ce chemin", "Add this path"), "+", () => add(el.querySelector("#cl-path").value.trim())));
        items.push(block(t("Comment c'est rangé", "How it is sorted"), t(
          "Le lanceur range chaque chemin tout seul : un dossier de jeux (rangés par console, comme RetroBat), un dossier d'émulateurs, ou l'exécutable d'un émulateur connu.",
          "The launcher sorts each path by itself: a game folder (one sub-folder per console, as in RetroBat), a folder of emulators, or the executable of a known emulator.")));
        if (data.emulatorDirsSearched.length) {
          items.push(block(t("Cherchés en ce moment", "Searched right now"), data.emulatorDirsSearched.map((p) => `<code>${escapeHtml(p)}</code>`).join("<br>")));
        }
        items.push(back());
        return { title: t("Dossiers", "Folders"), items };
      },

      capture() {
        const items = [block(t("État", "Status"), `${data.capture.available ? yes : no} ${escapeHtml(data.capture.detail)}`)];
        items.push(block("", data.windows
          ? t("Sous Windows, la fluidité est mesurée avec PresentMon (gratuit, open source). Sans lui, les réglages viennent de l'estimation de ta machine. PresentMon a besoin des droits administrateur, ou que ton compte soit dans le groupe « Utilisateurs du journal de performances ».",
              "On Windows, smoothness is measured with PresentMon (free, open source). Without it, settings come from the estimate of your machine. PresentMon needs administrator rights, or your account in the \"Performance Log Users\" group.")
          : t("Sous Linux, la fluidité est mesurée avec MangoHud : installe-le avec le gestionnaire de paquets de ta distribution.",
              "On Linux, smoothness is measured with MangoHud: install it with your distribution's package manager.")));
        if (data.windows) {
          items.push(row(t("Choisir PresentMon.exe…", "Choose PresentMon.exe…"), "📂", async () => {
            const p = await pick("exe", "PresentMon");
            if (!p) return;
            await prefs({ presentmonPath: p });
          }));
          if (data.config.perf.presentmon_path) items.push(row(t("Oublier ce chemin", "Forget this path"), "✕", () => prefs({ presentmonPath: "" })));
        }
        items.push(back());
        return { title: t("Mesure de fluidité", "Smoothness check"), items };
      },
    };

    async function prefs(update) {
      await run(() => invoke("update_preferences", { update }));
      await reload();
      if ("language" in update) libraryChanged();
    }

    async function addEmulator() {
      const p = await pick("exe", t("Exécutable de l'émulateur", "Emulator executable"));
      if (!p) return;
      const name = p.split(/[\\/]/).pop().replace(/\.(exe|appimage)$/i, "");
      const id = await run(() => invoke("add_custom_emulator", { path: p, name, systems: [], args: [] }));
      if (!id) return;
      await reload(); libraryChanged();
      const e = emu(id);
      if (e && !e.custom) {
        say(t(`Reconnu : ${e.name}. Il sera optimisé automatiquement.`, `Recognised: ${e.name}. It will be optimised automatically.`));
        push("emulator", id);
      } else {
        say(t("Ajouté. Choisis les consoles qu'il lance.", "Added. Choose the consoles it runs."));
        push("emulator", id);
        push("customSystems", id);
      }
    }

    const top = () => stack[stack.length - 1];
    function push(id, arg) { confirmRemove = null; stack.push({ id, arg, index: 0 }); render(); }
    function pop() {
      confirmRemove = null;
      stack.pop();
      if (!stack.length) return close();
      render();
    }

    let current = { items: [] };
    function render() {
      const page = top();
      current = pages[page.id](page.arg);
      const focusables = current.items.filter((x) => x.type !== "block");
      page.index = Math.min(page.index, Math.max(0, focusables.length - 1));
      let k = -1;
      const html = current.items.map((it) => {
        if (it.type === "block") {
          return `<div class="cl-block">${it.title ? `<span>${it.title}</span>` : ""}${it.html ? `<div>${it.html}</div>` : ""}</div>`;
        }
        k++;
        if (it.type === "input") {
          return `<label class="cl-input ${k === page.index ? "focused" : ""}" data-k="${k}"><span>${it.label}</span><input id="${it.id}" value="${escapeHtml(it.value)}" spellcheck="false"></label>`;
        }
        const cls = ["cl-row", k === page.index ? "focused" : "", it.dim ? "dim" : "", it.danger ? "danger" : ""].join(" ");
        const arrows = it.left ? '<i class="cl-arrows">‹ ›</i>' : "";
        return `<button class="${cls}" data-k="${k}"><span>${it.label}</span><b>${arrows}${it.value ?? ""}</b></button>`;
      }).join("");
      const crumbs = stack.length > 1 ? `<div class="cl-crumbs">${t("Réglages", "Settings")} › ${stack.slice(1).map((p) => pages[p.id](p.arg).title).join(" › ")}</div>` : "";
      el.innerHTML = `<div class="cl-panel" role="dialog" aria-label="Settings">${crumbs}<h2>${current.title}</h2>${html}
        ${toast ? `<div class="cl-toast">${escapeHtml(toast)}</div>` : ""}
        <div class="cl-help"><span><kbd>A</kbd>${t("Choisir", "Select")}</span><span><kbd>← →</kbd>${t("Changer", "Change")}</span><span><kbd>B</kbd>${t("Retour", "Back")}</span></div></div>`;
      el.querySelectorAll("[data-k]").forEach((node) => {
        node.addEventListener("click", (ev) => {
          if (ev.target.tagName === "INPUT") return;
          page.index = Number(node.dataset.k);
          activate();
        });
      });
      const f = el.querySelector(".focused");
      if (f) f.scrollIntoView({ block: "nearest" });
    }

    const focused = () => current.items.filter((x) => x.type !== "block")[top().index];
    async function activate(dir) {
      const it = focused();
      if (!it) return;
      if (it.type === "input") { el.querySelector(`#${it.id}`)?.focus(); return; }
      const fn = dir === "left" ? it.left : dir === "right" ? it.right : it.action;
      if (!fn) return;
      await fn();
      if (overlay) render();
    }

    function close() { el.remove(); overlay = null; clearTimeout(toastTimer); }

    overlay = {
      nav(a) {
        const page = top();
        const n = current.items.filter((x) => x.type !== "block").length;
        if (a === "up") { page.index = (page.index + n - 1) % n; confirmRemove = null; render(); }
        else if (a === "down") { page.index = (page.index + 1) % n; confirmRemove = null; render(); }
        else if (a === "pageLeft") { page.index = Math.max(0, page.index - 8); render(); }
        else if (a === "pageRight") { page.index = Math.min(n - 1, page.index + 8); render(); }
        else if (a === "left" || a === "right") { if (focused()?.[a]) activate(a); }
        else if (a === "confirm") activate();
        else if (a === "back") pop();
        else if (a === "menu") close();
      },
    };

    el.innerHTML = `<div class="cl-panel"><h2>${t("Réglages", "Settings")}</h2><div class="cl-block">…</div></div>`;
    await reload();
    hw = await Launcher.hardware(false).catch(() => null);
    stack.push({ id: "main", index: 0 });
    if (target && target.emulator) { push("emulator", target.emulator); return; }
    if (target && target.page && pages[target.page]) { push(target.page); return; }
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
