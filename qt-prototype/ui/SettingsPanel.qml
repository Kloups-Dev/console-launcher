// Built-in settings panel, shared by every theme:
//
//     import ConsoleLauncher
//     SettingsPanel { anchors.fill: parent }
//
// It opens with Launcher.openSettings(target) (Start / Tab in the themes).
// Pages are stacked like a console menu: up/down to move, A/Enter to choose,
// left/right to change a value, B/Escape to go back. Everything works with a
// controller; file pickers also accept a mouse.

import QtQuick
import QtQuick.Dialogs

Item {
    id: root
    anchors.fill: parent
    visible: Launcher.settingsOpen
    z: 1000

    // ---- Look ----
    property color text: "#e8ecf4"
    property color accent: "#ffc440"
    property color panelColor: "#151a24"

    // ---- State ----
    property var ov: null            // settings_overview
    property var hw: null
    property var stack: []           // [{id, arg, index}]
    property var items: []           // current page
    property string pageTitle: ""
    property string crumbs: ""
    property string toast: ""
    property var confirmRemove: null
    property bool busy: false

    readonly property bool fr: Launcher.language.indexOf("fr") === 0
    function t(f, e) { return fr ? f : e; }
    function esc(s) {
        return String(s === undefined || s === null ? "" : s)
            .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }
    readonly property string yes: "<font color='#6ee7a0'>✓</font>"
    readonly property string no: "<font color='#ff8a70'>✕</font>"
    function small(s) { return "<font color='#8a93a6'><small>" + s + "</small></font>"; }
    function code(s) { return "<tt>" + esc(s) + "</tt>"; }
    function onOff(v) { return v ? "ON" : "OFF"; }

    function row(label, value, action, extra) {
        const r = { type: "row", label: label, value: value, action: action };
        for (const k in (extra || {})) r[k] = extra[k];
        return r;
    }
    function block(title, html) { return { type: "block", title: title, html: html }; }
    function input(label, value, submit) { return { type: "input", label: label, value: value, submit: submit }; }
    function backRow() { return row(t("Retour", "Back"), "B", pop); }

    // Readable names for the settings the launcher manages.
    readonly property var keyNames: [
        [/InternalResolution|ResolutionScale|upscale_multiplier/i, ["Résolution", "Resolution"]],
        [/Renderer|GFXBackend|video_driver/i, ["Moteur graphique", "Graphics back-end"]],
        [/run_ahead/i, ["Latence (run-ahead)", "Latency (run-ahead)"]],
        [/video_threaded/i, ["Vidéo sur un fil séparé", "Threaded video"]],
        [/frameskip/i, ["Saut d'images", "Frame skipping"]],
        [/ShaderCompilation/i, ["Compilation des shaders", "Shader compilation"]],
        [/CPUThread/i, ["Double cœur", "Dual core"]]
    ]
    function keyName(k) {
        for (const kn of keyNames)
            if (kn[0].test(k)) return t(kn[1][0], kn[1][1]) + " " + small("(" + esc(k) + ")");
        return esc(k);
    }

    // ---- Engine ----
    function call(method, params, done) {
        Launcher.request(method, params || {}, function (ok, res) {
            if (!ok) say("⚠ " + res);
            if (done) done(ok, res);
        });
    }
    function reload(done) {
        call("settings_overview", {}, function (ok, res) {
            if (ok) ov = res;
            if (done) done();
        });
    }
    function libraryChanged() { Launcher.reload(); }
    function say(msg) { toast = msg; toastTimer.restart(); }
    Timer { id: toastTimer; interval: 5000; onTriggered: root.toast = "" }

    function emu(id) {
        if (!ov) return null;
        for (const e of ov.emulators) if (e.id === id) return e;
        return null;
    }
    function sysName(id) {
        for (const c of ov.systems) if (c.info.id === id) return c.info.name;
        return id;
    }

    // ---- File pickers ----
    property var pickDone: null
    FileDialog {
        id: fileDialog
        onAccepted: if (root.pickDone) root.pickDone(Launcher.localPath(selectedFile))
    }
    FolderDialog {
        id: folderDialog
        onAccepted: if (root.pickDone) root.pickDone(Launcher.localPath(selectedFolder))
    }
    function pickFile(title, done) { pickDone = done; fileDialog.title = title; fileDialog.open(); }
    function pickFolder(title, done) { pickDone = done; folderDialog.title = title; folderDialog.open(); }

    // ---- Pages ----
    function pageMain() {
        const c = ov.config;
        const gpu = hw && hw.gpus && hw.gpus.length ? hw.gpus[0].name : "GPU ?";
        let missing = 0;
        for (const s of ov.systems) {
            if (s.games === 0) continue;
            const o = s.options.find(x => x.id === s.emulator);
            if (!o || !o.found) missing++;
        }
        const found = ov.emulators.filter(e => e.found).length;
        const themes = Launcher.themes;
        return {
            title: t("Réglages", "Settings"),
            items: [
                row(t("Consoles et émulateurs", "Consoles and emulators"), missing ? missing + " " + no : yes, () => push("systems")),
                row(t("Émulateurs", "Emulators"), found + " " + yes, () => push("emulators")),
                row(t("Dossiers de jeux et d'émulateurs", "Game and emulator folders"), String(c.rom_dirs.length + c.emulator_dirs.length), () => push("folders")),
                row(t("Optimisation automatique", "Automatic optimisation"), onOff(c.perf.auto_tune), () => prefs({ autoTune: !c.perf.auto_tune })),
                row(t("Optimisations système (priorité, GPU)", "System optimisations (priority, GPU)"), onOff(c.perf.system_tweaks), () => prefs({ systemTweaks: !c.perf.system_tweaks })),
                row(t("Mesure de fluidité", "Smoothness check"), ov.capture.available ? yes : no, () => push("capture")),
                row(t("Thème", "Theme"), esc(Launcher.theme), function () {
                    const i = themes.indexOf(Launcher.theme);
                    Launcher.setTheme(themes[(i + 1) % themes.length]);
                }),
                row(t("Langue", "Language"), fr ? "Français" : "English", () => prefs({ language: fr ? "en" : "fr" })),
                block(t("Machine", "Machine"), hw ? esc(hw.cpu.brand + " · " + hw.ram_mb + " MB · " + gpu + " · score " + (hw.bench ? hw.bench.score : "?")) : "—"),
                row(t("Refaire le test machine", "Re-run hardware test"), "↻", function () {
                    say(t("Test en cours…", "Testing…"));
                    call("hardware", { refresh: true }, function (ok, res) { if (ok) { hw = res; say(t("Test terminé.", "Done.")); render(); } });
                }),
                row(t("Plein écran", "Fullscreen"), "F11", () => Launcher.toggleFullscreen()),
                row(t("Quitter le lanceur", "Quit the launcher"), "⏻", () => Launcher.quit()),
                row(t("Fermer", "Close"), "B", close),
                block("", small("Console Launcher " + Launcher.version + " · " + esc(ov.dataDir)))
            ]
        };
    }

    function pageSystems() {
        const items = [block("", t(
            "Choisis l'émulateur de chaque console (← → ou A). « Auto » prend le premier émulateur installé, dans l'ordre de préférence.",
            "Choose the emulator of each console (← → or A). \"Auto\" takes the first installed emulator, in order of preference."))];
        let shownEmpty = false;
        for (const c of ov.systems) {
            if (c.games === 0 && !shownEmpty) {
                items.push(block(t("Consoles sans jeu", "Consoles without games"), ""));
                shownEmpty = true;
            }
            const cur = c.options.find(o => o.id === c.emulator);
            const value = esc(cur ? cur.name : c.emulator) + " " + (cur && cur.found ? yes : no) + (c.automatic ? " " + small("auto") : "");
            const choices = [null].concat(c.options.map(o => o.id));
            const cycle = function (d) {
                const i = c.automatic ? 0 : Math.max(0, choices.indexOf(c.emulator));
                const next = choices[(i + d + choices.length) % choices.length];
                call("set_system_emulator", { system: c.info.id, id: next }, function (ok) {
                    if (!ok) return;
                    const o = c.options.find(x => x.id === next);
                    say(c.info.name + " → " + (next ? o.name : "Auto"));
                    reload(render);
                    libraryChanged();
                });
            };
            items.push(row(esc(c.info.name) + " " + small(c.games + " " + t("jeux", "games")), value,
                           () => cycle(1), { left: () => cycle(-1), right: () => cycle(1), dim: c.games === 0 }));
        }
        items.push(backRow());
        return { title: t("Consoles et émulateurs", "Consoles and emulators"), items: items };
    }

    function pageEmulators() {
        const list = ov.emulators.slice().sort((a, b) => (b.found - a.found) || a.name.localeCompare(b.name));
        const items = [row(t("+ Ajouter un émulateur…", "+ Add an emulator…"), "", addEmulator)];
        for (const e of list) {
            items.push(row(esc(e.name) + (e.custom ? " " + small(t("ajouté", "added")) : ""),
                           (e.user_settings.length ? small(e.user_settings.length + " " + t("réglage(s) perso", "own setting(s)")) + " " : "") + (e.found ? yes : no),
                           (id => () => push("emulator", id))(e.id)));
        }
        if (ov.profileErrors.length)
            items.push(block(t("Fichiers de profil ignorés", "Profile files ignored"), ov.profileErrors.map(code).join("<br>")));
        items.push(backRow());
        return { title: t("Émulateurs", "Emulators"), items: items };
    }

    function pageEmulator(id) {
        const e = emu(id);
        if (!e) return pageEmulators();
        const items = [];
        items.push(block(t("Exécutable", "Executable"), e.path
            ? code(e.path) + (e.configured_path ? "" : " " + small("(" + t("détecté", "detected") + ")"))
            : no + " " + t("introuvable", "not found") + (e.configured_path ? "<br>" + code(e.configured_path) : "")));
        items.push(row(t("Choisir l'exécutable…", "Choose the executable…"), "📂", function () {
            pickFile(e.name, function (p) {
                if (!p) return;
                const done = function (ok) { if (ok) { say(t("Chemin enregistré.", "Path saved.")); reload(render); libraryChanged(); } };
                if (e.custom) {
                    const c = Object.assign({}, ov.config.custom_emulators[id], { path: p });
                    call("update_custom_emulator", { id: id, emulator: c }, done);
                } else call("set_emulator_path", { id: id, path: p }, done);
            });
        }));
        if (e.configured_path && !e.custom)
            items.push(row(t("Revenir à la détection automatique", "Back to automatic detection"), "↺", function () {
                call("set_emulator_path", { id: id, path: null }, function (ok) {
                    if (ok) { say(t("Détection automatique.", "Automatic detection.")); reload(render); libraryChanged(); }
                });
            }));
        if (e.found)
            items.push(row(t("Ouvrir l'émulateur (BIOS, manettes…)", "Open the emulator (BIOS, controllers…)"), "↗",
                           () => call("open_emulator", { id: id }, ok => { if (ok) say(t("Émulateur ouvert.", "Emulator opened.")); })));
        let opt;
        if (!e.tunes_settings)
            opt = t("Le lanceur ne connaît pas encore les réglages de cet émulateur : il le lance directement, avec les optimisations système et la mesure de fluidité. Un fichier de profil peut lui apprendre ses réglages.",
                    "The launcher does not know this emulator's settings yet: it starts it directly, with the system optimisations and the smoothness check. A profile file can teach it the settings.");
        else if (e.user_settings.length)
            opt = t("Automatique, sauf ces réglages que tu as changés dans " + esc(e.name) + " :",
                    "Automatic, except these settings you changed in " + esc(e.name) + ":") + "<br>" + e.user_settings.map(keyName).join("<br>");
        else
            opt = t("Automatique : le lanceur règle l'émulateur selon ta machine et la fluidité mesurée. Si tu changes un de ces réglages dans l'émulateur, il n'y touche plus.",
                    "Automatic: the launcher sets the emulator up for your machine and the measured smoothness. If you change one of these settings in the emulator, it stops touching it.");
        items.push(block(t("Optimisation", "Optimisation"), opt));
        if (e.user_settings.length)
            items.push(row(t("Réactiver l'optimisation automatique", "Re-enable automatic optimisation"), "⟳", function () {
                call("reset_emulator_tuning", { id: id }, function (ok) {
                    if (ok) { say(t("Le lanceur reprend la main sur ces réglages au prochain lancement.", "The launcher takes these settings back at the next launch.")); reload(render); }
                });
            }));
        items.push(block(t("Consoles", "Consoles"), e.systems.map(s => esc(sysName(s))).join(", ") || "—"));
        if (e.custom) {
            const c = ov.config.custom_emulators[id];
            items.push(row(t("Consoles lancées…", "Consoles it runs…"), String(c.systems.length), () => push("customSystems", id)));
            items.push(input(t("Arguments ({rom} = le jeu), puis Entrée", "Arguments ({rom} = the game), then Enter"),
                             (c.args.length ? c.args : ["{rom}"]).join(" "), function (v) {
                const args = v.trim() ? v.match(/"[^"]*"|\S+/g).map(a => a.replace(/^"|"$/g, "")) : [];
                call("update_custom_emulator", { id: id, emulator: Object.assign({}, c, { args: args }) }, function (ok) {
                    if (ok) { say(t("Arguments enregistrés.", "Arguments saved.")); reload(render); }
                });
            }));
            items.push(row(confirmRemove === id ? t("Confirmer la suppression", "Confirm removal") : t("Retirer cet émulateur", "Remove this emulator"), "🗑", function () {
                if (confirmRemove !== id) { confirmRemove = id; return; }
                confirmRemove = null;
                call("remove_custom_emulator", { id: id }, function (ok) {
                    if (ok) { say(t("Émulateur retiré.", "Emulator removed.")); libraryChanged(); reload(pop); }
                });
            }, { danger: true }));
        }
        if (e.homepage) items.push(block("", small(esc(e.license || "") + " · " + esc(e.homepage))));
        items.push(backRow());
        return { title: esc(e.name), items: items };
    }

    function pageCustomSystems(id) {
        const c = ov.config.custom_emulators[id];
        const items = ov.systems.map(function (s) {
            const on = c.systems.indexOf(s.info.id) >= 0;
            return row(esc(s.info.name), on ? yes : "—", function () {
                const systems = on ? c.systems.filter(x => x !== s.info.id) : c.systems.concat([s.info.id]);
                call("update_custom_emulator", { id: id, emulator: Object.assign({}, c, { systems: systems }) }, function (ok) {
                    if (ok) { reload(render); libraryChanged(); }
                });
            });
        });
        items.push(backRow());
        return { title: t("Consoles lancées", "Consoles it runs"), items: items };
    }

    function pageFolders() {
        const c = ov.config;
        const kindMsg = function (k) {
            if (k.kind === "games") return t("Dossier de jeux ajouté.", "Game folder added.");
            if (k.kind === "emulators") return t("Dossier d'émulateurs ajouté : les émulateurs qu'il contient sont détectés.", "Emulator folder added: the emulators inside are detected.");
            if (k.kind === "emulator") { const e = emu(k.id); return t("Exécutable reconnu : ", "Executable recognised: ") + (e ? e.name : k.id) + "."; }
            return t("Déjà présent.", "Already there.");
        };
        const add = function (p) {
            if (!p) return;
            call("add_path", { path: p }, function (ok, res) {
                if (ok) { say(kindMsg(res)); reload(render); libraryChanged(); }
            });
        };
        const removable = p => row(code(p), confirmRemove === p ? t("Confirmer ?", "Confirm?") : "✕", function () {
            if (confirmRemove !== p) { confirmRemove = p; return; }
            confirmRemove = null;
            call("remove_path", { path: p }, function (ok) { if (ok) { say(t("Retiré.", "Removed.")); reload(render); libraryChanged(); } });
        }, { danger: confirmRemove === p });
        const items = [block(t("Dossiers de jeux", "Game folders"), c.rom_dirs.length ? "" : "—")];
        for (const p of c.rom_dirs) items.push(removable(p));
        items.push(block(t("Dossiers d'émulateurs", "Emulator folders"), c.emulator_dirs.length ? "" : t(
            "— (le dossier « emulators » à côté d'un dossier « roms » est cherché tout seul)",
            "— (an \"emulators\" folder next to a \"roms\" folder is searched by itself)")));
        for (const p of c.emulator_dirs) items.push(removable(p));
        items.push(row(t("+ Ajouter un dossier…", "+ Add a folder…"), "📂",
                       () => pickFolder(t("Dossier de jeux ou d'émulateurs", "Game or emulator folder"), add)));
        items.push(input(t("Ou colle un chemin, puis Entrée", "Or paste a path, then Enter"), "", add));
        items.push(block(t("Comment c'est rangé", "How it is sorted"), t(
            "Le lanceur range chaque chemin tout seul : un dossier de jeux (rangés par console, comme RetroBat), un dossier d'émulateurs, ou l'exécutable d'un émulateur connu.",
            "The launcher sorts each path by itself: a game folder (one sub-folder per console, as in RetroBat), a folder of emulators, or the executable of a known emulator.")));
        if (ov.emulatorDirsSearched.length)
            items.push(block(t("Cherchés en ce moment", "Searched right now"), ov.emulatorDirsSearched.map(code).join("<br>")));
        items.push(backRow());
        return { title: t("Dossiers", "Folders"), items: items };
    }

    function pageCapture() {
        const cap = ov.capture;
        let status;
        if (cap.tool)
            status = yes + " " + code(cap.tool) + (cap.bundled ? " " + small("(" + t("fourni avec le lanceur", "shipped with the launcher") + ")") : "");
        else
            status = no + " " + esc(cap.detail);
        const items = [block(t("Outil", "Tool"), status)];
        if (ov.windows) {
            items.push(block(t("Autorisation", "Permission"), cap.permission
                ? yes + " " + t("Windows autorise la mesure.", "Windows allows the measurement.")
                : no + " " + t("PresentMon a besoin d'une autorisation de Windows pour lire les images affichées. Le bouton ci-dessous ajoute ton compte au groupe « Utilisateurs du journal de performances » (une fenêtre administrateur s'ouvre). Il faut ensuite fermer puis rouvrir ta session Windows.",
                               "PresentMon needs a Windows permission to read the displayed frames. The button below adds your account to the \"Performance Log Users\" group (an administrator prompt opens). Sign out of Windows and back in afterwards.")));
            if (!cap.permission)
                items.push(row(t("Autoriser la mesure…", "Allow the measurement…"), "🛡", function () {
                    call("allow_capture", {}, function (ok) {
                        if (ok) say(t("C'est fait. Ferme puis rouvre ta session Windows pour l'activer.", "Done. Sign out of Windows and back in to enable it."));
                    });
                }));
            items.push(row(t("Utiliser un autre PresentMon.exe…", "Use another PresentMon.exe…"), "📂",
                           () => pickFile("PresentMon", p => { if (p) prefs({ presentmonPath: p }); })));
            if (ov.config.perf.presentmon_path)
                items.push(row(t("Revenir à celui fourni", "Back to the shipped one"), "↺", () => prefs({ presentmonPath: "" })));
        } else {
            items.push(block("", t(
                "Sous Linux, la fluidité est mesurée avec MangoHud. Celui installé sur la machine est utilisé s'il existe, sinon celui fourni avec le lanceur.",
                "On Linux, smoothness is measured with MangoHud. The copy installed on the machine is used when present, otherwise the one shipped with the launcher.")));
        }
        items.push(block("", t(
            "Sans mesure, les réglages viennent de l'estimation de ta machine.",
            "Without measurement, settings come from the estimate of your machine.")));
        items.push(backRow());
        return { title: t("Mesure de fluidité", "Smoothness check"), items: items };
    }

    readonly property var pages: ({
        main: pageMain, systems: pageSystems, emulators: pageEmulators, emulator: pageEmulator,
        customSystems: pageCustomSystems, folders: pageFolders, capture: pageCapture
    })

    function prefs(update) {
        call("update_preferences", update, function (ok) {
            if (!ok) return;
            reload(render);
            if ("language" in update) libraryChanged();
        });
    }

    function addEmulator() {
        pickFile(t("Exécutable de l'émulateur", "Emulator executable"), function (p) {
            if (!p) return;
            const name = p.split(/[\\/]/).pop().replace(/\.(exe|appimage)$/i, "");
            call("add_custom_emulator", { path: p, name: name, systems: [], args: [] }, function (ok, id) {
                if (!ok) return;
                libraryChanged();
                reload(function () {
                    const e = emu(id);
                    push("emulator", id);
                    if (e && !e.custom) {
                        say(t("Reconnu : " + e.name + ". Il sera optimisé automatiquement.", "Recognised: " + e.name + ". It will be optimised automatically."));
                    } else {
                        say(t("Ajouté. Choisis les consoles qu'il lance.", "Added. Choose the consoles it runs."));
                        push("customSystems", id);
                    }
                });
            });
        });
    }

    // ---- Navigation ----
    function currentPage() { return stack[stack.length - 1]; }
    function push(id, arg) {
        confirmRemove = null;
        const s = stack.slice(); s.push({ id: id, arg: arg, index: 0 }); stack = s;
        render();
    }
    function pop() {
        confirmRemove = null;
        const s = stack.slice(); s.pop(); stack = s;
        if (!stack.length) return close();
        render();
    }
    function focusables() { return items.filter(x => x.type !== "block"); }
    function render() {
        if (!ov || !stack.length) return;
        const page = currentPage();
        const p = pages[page.id](page.arg);
        const focusable = p.items.filter(x => x.type !== "block").length;
        page.index = Math.min(page.index, Math.max(0, focusable - 1));
        items = p.items;
        pageTitle = p.title;
        crumbs = stack.length > 1
            ? t("Réglages", "Settings") + " › " + stack.slice(1).map(s => pages[s.id](s.arg).title).join(" › ")
            : "";
        const k = indexOfFocusable(page.index);
        if (k >= 0) list.positionViewAtIndex(k, ListView.Contain);
    }
    function indexOfFocusable(n) {
        let seen = -1;
        for (let i = 0; i < items.length; i++)
            if (items[i].type !== "block" && ++seen === n) return i;
        return -1;
    }
    function activate(dir) {
        const page = currentPage();
        const it = focusables()[page.index];
        if (!it) return;
        if (it.type === "input") {
            const d = list.itemAtIndex(indexOfFocusable(page.index));
            if (d) d.focusInput();
            return;
        }
        const fn = dir === "left" ? it.left : dir === "right" ? it.right : it.action;
        if (!fn) return;
        fn();
        if (Launcher.settingsOpen) render();
    }
    function nav(a) {
        const page = currentPage();
        if (!page) return;
        const n = focusables().length;
        if (a === "up") { page.index = (page.index + n - 1) % n; confirmRemove = null; render(); }
        else if (a === "down") { page.index = (page.index + 1) % n; confirmRemove = null; render(); }
        else if (a === "pageLeft") { page.index = Math.max(0, page.index - 8); render(); }
        else if (a === "pageRight") { page.index = Math.min(n - 1, page.index + 8); render(); }
        else if (a === "left" || a === "right") { const it = focusables()[page.index]; if (it && it[a]) activate(a); }
        else if (a === "confirm") activate();
        else if (a === "back") pop();
        else if (a === "menu") close();
    }

    function open(target) {
        Launcher.settingsOpen = true;
        stack = [{ id: "main", index: 0 }];
        busy = true;
        call("hardware", { refresh: false }, function (ok, res) { if (ok) hw = res; render(); });
        reload(function () {
            busy = false;
            if (target && target.emulator) push("emulator", target.emulator);
            else if (target && target.page && pages[target.page]) push(target.page);
            else render();
        });
    }
    function close() {
        Launcher.settingsOpen = false;
        stack = [];
        toast = "";
    }

    Connections {
        target: Launcher
        function onSettingsRequested(target) { root.open(target); }
        function onSettingsNav(a) { root.nav(a); }
    }

    // ---- Drawing ----
    Rectangle { anchors.fill: parent; color: Qt.rgba(0.02, 0.03, 0.05, 0.78) }
    MouseArea { anchors.fill: parent }   // blocks clicks to the theme

    Rectangle {
        id: panel
        anchors.centerIn: parent
        width: Math.min(760, root.width * 0.92)
        height: Math.min(root.height * 0.9, header.height + list.contentHeight + footer.height + 48)
        radius: 16
        color: root.panelColor
        border.color: "#2a3244"

        Column {
            id: header
            x: 24; y: 20; width: parent.width - 48; spacing: 2
            Text { visible: root.crumbs !== ""; text: root.crumbs; textFormat: Text.StyledText; color: root.text; opacity: 0.55; font.pixelSize: 12 }
            Text { text: root.busy ? "…" : root.pageTitle; textFormat: Text.StyledText; color: root.text; font.pixelSize: 21; font.weight: Font.DemiBold; bottomPadding: 10 }
        }

        ListView {
            id: list
            anchors.top: header.bottom
            anchors.bottom: footer.top; anchors.bottomMargin: 8
            x: 24; width: parent.width - 48
            clip: true
            model: root.items.length
            boundsBehavior: Flickable.StopAtBounds
            delegate: SettingsRow {
                required property int index
                item: root.items[index]
                focused: root.stack.length > 0 && index === root.indexOfFocusable(root.currentPage().index)
                text: root.text
                accent: root.accent
                onActivated: {
                    let n = -1;
                    for (let i = 0; i <= index; i++) if (root.items[i].type !== "block") n++;
                    root.currentPage().index = n;
                    root.activate();
                }
                onSubmitted: function (value) { if (item.submit) item.submit(value); }
            }
        }

        Rectangle {
            id: footer
            anchors.bottom: parent.bottom; anchors.bottomMargin: 1
            x: 1; width: parent.width - 2; height: toastBox.visible ? 92 : 44
            color: root.panelColor; radius: 16
            Rectangle {
                id: toastBox
                visible: root.toast !== ""
                x: 23; y: 0; width: parent.width - 46; height: 40; radius: 10
                color: "#2b3550"
                Text { anchors.fill: parent; anchors.margins: 10; text: root.toast; color: root.text; font.pixelSize: 13; elide: Text.ElideRight; verticalAlignment: Text.AlignVCenter }
            }
            Row {
                anchors.right: parent.right; anchors.rightMargin: 24
                anchors.bottom: parent.bottom; anchors.bottomMargin: 12
                spacing: 18
                Repeater {
                    model: [["A", root.t("Choisir", "Select")], ["← →", root.t("Changer", "Change")], ["B", root.t("Retour", "Back")]]
                    Row {
                        spacing: 6
                        Rectangle { width: k.implicitWidth + 12; height: 20; radius: 6; color: "#2a3244"
                                    Text { id: k; anchors.centerIn: parent; text: modelData[0]; color: root.text; font.pixelSize: 12 } }
                        Text { text: modelData[1]; color: root.text; opacity: 0.8; font.pixelSize: 13; anchors.verticalCenter: parent.verticalCenter }
                    }
                }
            }
        }
    }
}
