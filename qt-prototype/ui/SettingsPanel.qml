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
    property color text: "#171717"
    property color accent: "#2864c7"
    property color panelColor: "#f4f4f2"

    // ---- State ----
    property var ov: null            // settings_overview
    property var hw: null
    property var stack: []           // [{id, arg, index}]
    property var items: []           // current page
    property string pageTitle: ""
    property string crumbs: ""
    property string toast: ""
    property var confirmRemove: null
    property string pendingGamePath: ""
    property bool busy: false
    property bool controlTesterOpen: false

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
    function onOff(v) { return v ? t("Activé", "On") : t("Désactivé", "Off"); }

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
        Launcher.internalRequest(method, params || {}, function (ok, res) {
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
    function system(id) {
        if (!ov) return null;
        for (const c of ov.systems) if (c.info.id === id) return c;
        return null;
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

    readonly property var sections: [
        {id:"folders", fr:"Jeux et dossiers", en:"Games and folders", descriptionFr:"Ajouter des jeux et choisir où les chercher.", descriptionEn:"Add games and choose which folders to scan."},
        {id:"controls", fr:"Manettes et touches", en:"Controllers and keys", descriptionFr:"Vérifier ta manette et comprendre les commandes en jeu.", descriptionEn:"Check your controller and game controls."},
        {id:"image", fr:"Image et fluidité", en:"Picture and smoothness", descriptionFr:"Choisir une image nette ou lissée et les réglages automatiques.", descriptionEn:"Choose sharp or smooth image and automatic settings."},
        {id:"saves", fr:"Sauvegardes et reprise", en:"Saves and resume", descriptionFr:"Garder ta progression et choisir la reprise instantanée.", descriptionEn:"Keep your progress and choose instant resume."},
        {id:"systems", fr:"Consoles et installation", en:"Systems and installation", descriptionFr:"Voir les consoles et les moteurs qui les font fonctionner.", descriptionEn:"Browse systems and their emulation engines."},
        {id:"general", fr:"Application", en:"Application", descriptionFr:"Langue, affichage et informations du logiciel.", descriptionEn:"Language, display and application information."},
        {id:"help", fr:"Aide et dépannage", en:"Help and troubleshooting", descriptionFr:"Une manette muette, un jeu lent ou une sauvegarde introuvable ?", descriptionEn:"Controller not responding, slow game or missing save?"},
        {id:"advanced", fr:"Options avancées", en:"Advanced options", descriptionFr:"Émulateurs externes, BIOS, matériel et diagnostic.", descriptionEn:"External emulators, BIOS, hardware and diagnostics."}
    ]
    function switchSection(id) {
        if (busy) return
        stack = [{id:"main",index:0},{id:id,index:0}]
        confirmRemove = null
        render()
        list.positionViewAtBeginning()
    }
    function activeSection() {
        if (stack.length < 2) return ""
        const id = stack[1].id
        if (["performance","diagnostics","emulators","emulator","storage","capture","customSystems"].indexOf(id) >= 0) return "advanced"
        if (id === "system") return "systems"
        return id
    }
    function dataFolder() {
        const path = String(ov.dataDir).replace(/\\/g, "/").replace(/^\/\/\?\//, "")
        if (!Qt.openUrlExternally("file:///" + path))
            say(t("Impossible d’ouvrir ce dossier.", "Unable to open this folder."))
    }
    function pageMain() {
        return {title:t("Paramètres", "Settings"),items:[
            block(t("Que souhaites-tu régler ?", "What would you like to change?"),
                  t("Choisis une rubrique à gauche. Les changements sont enregistrés dès que tu les sélectionnes.", "Choose a section on the left. Changes are saved as soon as you select them.")),
            ...sections.map(s => row(t(s.fr,s.en), "›", () => switchSection(s.id), {description:t(s.descriptionFr,s.descriptionEn)}))
        ]}
    }
    function pageGeneral() {
        return {title:t("Application", "Application"),items:[
            block(t("Personnaliser le logiciel", "Personalise the application"),t("Ces choix concernent le lanceur, pas les commandes ni les sauvegardes des jeux.", "These choices affect the launcher, not game controls or saves.")),
            row(t("Langue", "Language"),fr ? "Français" : "English",() => prefs({language:fr ? "en" : "fr"})),
            row(t("Passer en plein écran / en fenêtre", "Switch fullscreen / windowed"),"F11",() => Launcher.toggleFullscreen()),
            block(t("Version du logiciel", "Application version"),"Console Launcher " + esc(Launcher.version)),
            backRow()
        ]}
    }
    function pageImage() {
        const c = ov.config
        const modes = ["responsive", "balanced", "quality"]
        const mode = c.perf.experience || "balanced"
        const labels = [t("Réactivité", "Responsiveness"),t("Équilibré — conseillé", "Balanced — recommended"),t("Qualité", "Quality")]
        const index = Math.max(0,modes.indexOf(mode))
        const cycle = d => prefs({experience:modes[(index+d+3)%3]})
        const descriptions = [
            t("Son plus réactif, résolution plafonnée à 1080p et synchronisation verticale désactivée. Possibles craquements ou déchirures de l’image.", "Shorter audio buffering, up to 1080p and VSync off. Audio crackling or tearing may occur."),
            t("Compromis son/image : tampon audio modéré, synchronisation verticale et résolution jusqu’à 1440p selon le profil du jeu.", "Audio/video balance: moderate audio buffering, VSync and up to 1440p according to the game profile."),
            t("Résolution jusqu’à 4K selon les résultats, synchronisation verticale et marge audio plus large. Demande davantage au PC.", "Up to 4K according to results, VSync and more audio buffering. More demanding on the PC.")
        ]
        return {title:t("Image et fluidité", "Picture and smoothness"),items:[
            row(t("Priorité son et image", "Audio/video priority"),labels[index],() => cycle(1),{left:()=>cycle(-1),right:()=>cycle(1),description:descriptions[index]}),
            block(t("Ce que ce choix règle", "What this choice controls"),t("Préréglages son/image pour Dolphin et le moteur rétro interne. Les plafonds de résolution concernent les moteurs compatibles. Les FPS natifs du jeu sont conservés : un écran 144 Hz ne transforme pas automatiquement un jeu 30 FPS en 144 FPS.", "Audio/video presets for Dolphin and the internal retro engine. Resolution ceilings apply to compatible engines. Native game FPS are preserved; a 144 Hz display does not automatically turn a 30 FPS game into 144 FPS.")),
            block(t("Pour jouer simplement", "For straightforward play"),t("Garde les réglages automatiques pour commencer. Une image plus lissée change son apparence, pas la puissance de ton PC.", "Start with automatic settings. Smoothing changes the picture, not your PC's performance.")),
            row(t("Adapter les réglages au PC", "Adapt settings to this PC"),onOff(c.perf.auto_tune),() => prefs({autoTune:!c.perf.auto_tune}),{description:t("Le lanceur estime un profil puis peut l’ajuster avec les mesures disponibles.", "Estimate a profile and adjust it using available measurements.")}),
            row(t("Image des jeux rétro", "Retro game picture"),c.retroarch.smooth_image ? t("Lissée", "Smooth") : t("Nette", "Sharp"),() => prefs({retroarchSmoothImage:!c.retroarch.smooth_image}),{description:t("Moteur rétro interne uniquement. « Nette » garde les pixels visibles ; « Lissée » adoucit les contours.", "Internal retro engine only. Sharp preserves pixels; smooth softens edges.")}),
            block(t("Son et image en jeu", "In-game audio and video"),t("Le moteur rétro utilise une base de synchronisation son/image. Les réglages propres à une console restent gérés par son moteur.", "The retro engine uses synchronised audio/video defaults. Console-specific settings remain managed by its engine.")),
            row(t("Afficher les réglages techniques", "Show technical settings"),"›",() => push("performance"),{description:t("Mesures de fluidité, informations du PC et panneau en jeu.", "Frame measurements, PC information and in-game panel.")}),
            backRow()
        ]}
    }
    function pageSaves() {
        const c = ov.config
        return {title:t("Sauvegardes et reprise", "Saves and resume"),items:[
            block(t("1. Ta progression dans le jeu", "1. Your in-game progress"),t("Sauvegarde quand le jeu le propose, comme sur la console d’origine. Revenir au menu de départ au prochain lancement ne signifie pas que cette progression est perdue.", "Save when the game offers it, as on the original console. Starting at the title screen next time does not mean your progress is lost.")),
            block(t("2. Reprendre exactement au même endroit", "2. Resume at the same point"),t("La reprise instantanée conserve l’état du jeu à la fermeture. Elle concerne le moteur rétro interne et dépend du cœur utilisé. Elle ne remplace pas une sauvegarde faite dans le jeu.", "Instant resume keeps the game state when closing. It applies to the internal retro engine and depends on the core. It does not replace in-game saves.")),
            row(t("Reprendre les jeux rétro à la réouverture", "Resume retro games on reopening"),onOff(c.retroarch.resume),() => prefs({retroarchResume:!c.retroarch.resume}),{description:t("Option du moteur rétro. Elle n’active pas la reprise de Mario dans Dolphin.", "Retro engine option. This does not enable Mario resume in Dolphin.")}),
            block(t("Raccourcis du moteur rétro", "Retro engine shortcuts"),t("F2 : créer une sauvegarde instantanée · F4 : la recharger · Échap : quitter proprement.", "F2: create a save state · F4: load it · Escape: quit cleanly.")),
            row(t("Ouvrir le dossier des données", "Open application data folder"),"↗",dataFolder,{description:t("Les sous-dossiers saves et states contiennent les sauvegardes rétro. Dolphin utilise generated/dolphin-user/GC.", "The saves and states subfolders contain retro saves. Dolphin uses generated/dolphin-user/GC.")}),
            block(t("À conserver", "Keep these files"),t("Les sauvegardes sont séparées des versions du moteur. Les émulateurs installés manuellement peuvent utiliser un autre dossier.", "Saves are separate from engine versions. Manually installed emulators may use a different folder.")),
            backRow()
        ]}
    }
    function pageHelp() {
        return {title:t("Aide et dépannage", "Help and troubleshooting"),items:[
            row(t("Ma manette ne répond pas", "My controller does not respond"),"›",() => push("controls"),{description:t("Tester la détection et distinguer le testeur des commandes du jeu.", "Check detection and distinguish tester input from game controls.")}),
            row(t("Je ne retrouve pas ma sauvegarde", "I cannot find my save"),"›",() => push("saves"),{description:t("Comprendre la progression du jeu et la reprise instantanée.", "Understand in-game progress and save-state resume.")}),
            row(t("Le jeu est lent ou l’image saccade", "The game is slow or stutters"),"›",() => push("image")),
            row(t("Un jeu demande un BIOS ou un émulateur", "A game asks for a BIOS or emulator"),"›",() => push("systems")),
            row(t("Voir les erreurs et le diagnostic", "View errors and diagnostics"),"›",() => push("diagnostics")),
            backRow()
        ]}
    }
    function pageAdvanced() {
        return {title:t("Options avancées", "Advanced options"),items:[
            block(t("À utiliser si nécessaire", "Use when needed"),t("Le parcours normal reste : choisir un jeu, puis Lancer. Ces réglages servent à personnaliser une installation ou résoudre un problème.", "The normal flow remains: choose a game, then Launch. These settings customise an installation or help diagnose a problem.")),
            row(t("Gérer les moteurs et émulateurs", "Manage engines and emulators"),"›",() => push("emulators"),{description:t("Installation, choix d’un programme externe et options propres au moteur.", "Installation, external executable selection and engine-specific options.")}),
            row(t("BIOS et stockage avancé", "BIOS and advanced storage"),"›",() => push("storage")),
            row(t("Matériel et mesures de fluidité", "Hardware and frame measurements"),"›",() => push("performance")),
            row(t("Erreurs et état du logiciel", "Errors and application status"),"›",() => push("diagnostics")),
            backRow()
        ]}
    }
    function pageLibrary() {
        const c = ov.config
        const add = function(path) { if (!path) return; call("add_path",{path:path},function(ok) { if(ok) { libraryChanged(); reload(render); say(t("Dossier ajouté.", "Folder added.")) } }) }
        const items = [
            block(t("Tes jeux", "Your games"),t("Ajoute un fichier ou un dossier. Les jeux restent à leur emplacement ; le lanceur ne les déplace pas.", "Add a file or folder. Games stay where they are; the launcher does not move them.")),
            row(t("Ajouter un jeu", "Add a game"),"＋",() => pickFile(t("Choisir un jeu", "Choose a game"),function(path) { if(path) { pendingGamePath=path; push("gameSystem",path) } })),
            row(t("Ajouter un dossier de jeux", "Add a game folder"),"＋",() => pickFolder(t("Choisir un dossier de jeux", "Choose a game folder"),add)),
            block(t("Dossiers suivis", "Watched folders"),c.rom_dirs.length ? "" : t("Aucun dossier ajouté pour le moment.", "No folders added yet."))
        ]
        for (const path of c.rom_dirs) items.push(row(code(path),confirmRemove===path ? t("Confirmer le retrait", "Confirm removal") : t("Retirer", "Remove"),function() {
            if(confirmRemove!==path) { confirmRemove=path; return }
            confirmRemove=null
            call("remove_path",{path:path},function(ok) { if(ok) { libraryChanged(); reload(render) } })
        },{description:t("Retire le dossier de la liste. Les fichiers de jeux sont conservés.", "Remove the folder from this list. Game files are kept."),danger:confirmRemove===path}))
        items.push(block(t("Jeux ajoutés un par un", "Individually added games"),c.imported_games.length ? t(c.imported_games.length + " jeu(x) ajouté(s).",c.imported_games.length + " added game(s).") : t("Aucun pour le moment.", "None yet.")))
        items.push(row(t("Gérer les chemins et les jeux ajoutés", "Manage paths and added games"),"›",() => push("storage")))
        items.push(backRow())
        return {title:t("Jeux et dossiers", "Games and folders"),items:items}
    }
    function pageControls() {
        const device = Launcher.padName
            ? yes + " " + esc(Launcher.padName)
            : small(t("Aucune manette détectée — le clavier reste disponible.", "No controller detected — the keyboard remains available."));
        return {
            title: t("Manettes et touches", "Controllers and keys"),
            items: [
                block(t("Périphérique actif", "Active device"), device),
                row(t("Tester et visualiser les contrôles", "Test and visualise controls"), "›", function () {
                    controlTesterOpen = true;
                    Launcher.controlCapture = true;
                }),
                block(t("Navigation dans le launcher", "Launcher navigation"), t(
                    "Déplacement : flèches ou croix directionnelle<br>Valider : Entrée ou A<br>Retour : Échap ou B<br>Réglages : Tab ou Start<br>Favori : F ou Y<br>Plein écran : F11",
                    "Move: arrow keys or D-pad<br>Confirm: Enter or A<br>Back: Escape or B<br>Settings: Tab or Start<br>Favourite: F or Y<br>Fullscreen: F11")),
                block(t("Contrôles en jeu", "In-game controls"), t(
                    "Moteur rétro : les manettes reconnues sont configurées automatiquement ; F1 ouvre les réglages en jeu. Pour GameCube avec Dolphin, utilise « Utiliser dans Dolphin » dans le testeur. Voir une réaction dans le testeur ne suffit pas à configurer tous les émulateurs.",
                    "Game controls depend on the console and selected emulator. Configure them from that emulator's page.")),
                backRow()
            ]
        };
    }

    function pagePerformance() {
        const c = ov.config;
        const gpu = hw && hw.main_gpu ? hw.main_gpu.name
                  : hw && hw.gpus && hw.gpus.length ? hw.gpus[0].name : "GPU ?";
        return {
            title: t("Performance et machine", "Performance and hardware"),
            items: [
                block(t("Réglages automatiques", "Automatic settings"), t(
                    "Ces options servent de base à toutes les consoles. Les adaptations précises restent liées au jeu et à son émulateur.",
                    "These options are the base for every console. Precise adaptations remain tied to each game and emulator.")),
                row(t("Panneau de diagnostic en jeu", "In-game diagnostics panel"), onOff(Launcher.performance.enabled), function () {
                    Launcher.performance.enabled = !Launcher.performance.enabled; render();
                }),
                row(t("Touche du panneau", "Panel shortcut"), "F" + Launcher.performance.functionKey, function () {
                    Launcher.performance.functionKey = Launcher.performance.functionKey % 11 + 1; render();
                }, {
                    left: function () { Launcher.performance.functionKey = (Launcher.performance.functionKey + 9) % 11 + 1; render(); },
                    right: function () { Launcher.performance.functionKey = Launcher.performance.functionKey % 11 + 1; render(); }
                }),
                block(t("Diagnostic en jeu", "In-game diagnostics"), t(
                    "Windows : vue compacte en haut à droite au lancement. La touche choisie alterne compact, diagnostic complet et masqué. Ctrl+Alt+F8 sert de secours. Mode fenêtré/sans bordure requis. Les capteurs indisponibles sont signalés.",
                    "Windows: compact view at the top right when playing. The selected key cycles compact, full diagnostics and hidden. Ctrl+Alt+F8 is the fallback. Requires windowed/borderless mode. Missing sensors are reported.")),
                row(t("Optimisation automatique", "Automatic optimisation"), onOff(c.perf.auto_tune), () => prefs({ autoTune: !c.perf.auto_tune })),
                row(t("Optimisations système (priorité, GPU)", "System optimisations (priority, GPU)"), onOff(c.perf.system_tweaks), () => prefs({ systemTweaks: !c.perf.system_tweaks })),
                row(t("Mesure de fluidité", "Smoothness check"), ov.capture.available ? yes : no, () => push("capture")),
                block(t("Machine détectée", "Detected hardware"), hw ? esc(hw.cpu.brand + " · " + hw.ram_mb + " MB · " + gpu + " · score " + (hw.bench ? hw.bench.score : "?")) : "—"),
                row(t("Refaire le test machine", "Re-run hardware test"), "↻", function () {
                    say(t("Test en cours…", "Testing…"));
                    call("hardware", { refresh: true }, function (ok, res) { if (ok) { hw = res; say(t("Test terminé.", "Done.")); render(); } });
                }),
                backRow()
            ]
        };
    }

    function pageDiagnostics() {
        const found = ov.emulators.filter(e => e.found).length;
        return {
            title: t("État et diagnostic", "Status and diagnostics"),
            items: [
                block(t("Application", "Application"), yes + " " + t("Moteur prêt", "Engine ready") + "<br>" +
                      esc(ov.systems.length) + " " + t("consoles prises en charge", "supported consoles") + "<br>" +
                      esc(found) + " " + t("émulateur(s) détecté(s)", "emulator(s) detected")),
                block(t("Dernière erreur", "Last error"), Launcher.lastError ? esc(Launcher.lastError) : yes + " " + t("Aucune", "None")),
                row(t("Mesure de fluidité", "Smoothness check"), ov.capture.available ? yes : no, () => push("capture")),
                row(t("Émulateurs", "Emulators"), "›", () => push("emulators")),
                block(t("Dossier de données", "Data folder"), code(ov.dataDir)),
                backRow()
            ]
        };
    }

    function pageSystems() {
        const items = [block("", t(
            "Ouvre une console pour retrouver sa bibliothèque, son émulateur et ses réglages adaptés.",
            "Open a console to find its library, emulator and tailored settings."))];
        let shownEmpty = false;
        for (const c of ov.systems) {
            if (c.games === 0 && !shownEmpty) {
                items.push(block(t("Consoles sans jeu", "Consoles without games"), ""));
                shownEmpty = true;
            }
            const cur = c.options.find(o => o.id === c.emulator);
            const value = esc(cur ? cur.name : c.emulator) + " " + (cur && cur.found ? yes : no) + (c.automatic ? " " + small("auto") : "");
            items.push(row(esc(c.info.name) + " " + small(c.games + " " + t("jeux", "games")), value,
                           (id => () => push("system", id))(c.info.id), { dim: c.games === 0 }));
        }
        items.push(backRow());
        return { title: t("Consoles", "Consoles"), items: items };
    }

    function pageSystem(id) {
        const c = system(id);
        if (!c) return pageSystems();
        const current = c.options.find(o => o.id === c.emulator);
        const choices = [null].concat(c.options.map(o => o.id));
        const cycle = function (d) {
            const i = c.automatic ? 0 : Math.max(0, choices.indexOf(c.emulator));
            const next = choices[(i + d + choices.length) % choices.length];
            call("set_system_emulator", { system: c.info.id, id: next }, function (ok) {
                if (!ok) return;
                const option = c.options.find(x => x.id === next);
                say(c.info.name + " → " + (next && option ? option.name : "Auto"));
                reload(render);
            });
        };
        const emulatorValue = esc(current ? current.name : c.emulator) + " " +
            (current && current.found ? yes : no) + (c.automatic ? " " + small("auto") : "");
        const items = [
            block(t("Bibliothèque", "Library"), esc(c.games) + " " + t("jeu(x)", "game(s)") + " · " +
                  small((c.info.extensions || []).map(x => "." + esc(x)).join("  "))),
            block(t("Émulation", "Emulation"), t(
                "« Auto » choisit le premier émulateur compatible installé. Utilise gauche/droite ou valide pour changer.",
                "Auto selects the first installed compatible emulator. Use left/right or confirm to change.")),
            row(t("Émulateur utilisé", "Selected emulator"), emulatorValue,
                () => cycle(1), { left: () => cycle(-1), right: () => cycle(1) })
        ];
        if (current)
            items.push(row(t("Configurer " + current.name + "…", "Configure " + current.name + "…"), "›",
                           () => push("emulator", current.id)));
        items.push(block(t("Réglages adaptés à cette console", "Settings tailored to this console"), t(
            "Le launcher applique au lancement le profil compatible avec cette console, puis l'adapte à la puissance de la machine et aux mesures du jeu. Les options très spécifiques restent dans la fiche de l'émulateur pour éviter de surcharger cet écran.",
            "At launch, the launcher applies the profile compatible with this console, then adapts it to the machine and the game's measurements. Very specific options stay on the emulator page to keep this screen clear.")));
        items.push(row(t("Jeux et dossiers", "Games and folders"), "›", () => push("folders")));
        items.push(backRow());
        return { title: esc(c.info.name), items: items };
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
        if (e.managed_version)
            items.push(block(t("Version installée par le launcher", "Version installed by the launcher"),
                             yes + " " + esc(e.managed_version) + " " + small(t("vérifiée et active", "verified and active"))));
        if (e.automatic_install && (!e.managed_version || !e.install_version || e.managed_version !== e.install_version)) {
            const version = e.install_version ? " " + e.install_version : "";
            items.push(row(t("Installer directement " + e.name + version, "Install " + e.name + version + " directly"), "↓", function () {
                if (busy) return;
                busy = true;
                say(t("Téléchargement et vérification en cours…", "Downloading and verifying…"));
                render();
                call("install_emulator", { id: id }, function (ok, result) {
                    busy = false;
                    if (!ok) { render(); return; }
                    say(t(result.emulator + " " + result.version + " est installé et prêt.",
                          result.emulator + " " + result.version + " is installed and ready."));
                    reload(render);
                });
            }));
        }
        if (!e.found && e.download_page) {
            items.push(row(t("Télécharger " + e.name + " sur le site officiel…",
                             "Download " + e.name + " from the official website…"), "↗", function () {
                Qt.openUrlExternally(e.download_page);
                say(t("Page officielle ouverte. Après l’installation, choisis l’exécutable ci-dessous.",
                      "Official page opened. After installation, choose the executable below."));
            }));
        }
        items.push(row(t("Choisir l'exécutable…", "Choose the executable…"), "📂", function () {
            pickFile(e.name, function (p) {
                if (!p) return;
                const done = function (ok) { if (ok) { say(t("Chemin enregistré.", "Path saved.")); reload(render); } };
                if (e.custom) {
                    const c = Object.assign({}, ov.config.custom_emulators[id], { path: p });
                    call("update_custom_emulator", { id: id, emulator: c }, done);
                } else call("set_emulator_path", { id: id, path: p }, done);
            });
        }));
        if (e.configured_path && !e.custom)
            items.push(row(t("Revenir à la détection automatique", "Back to automatic detection"), "↺", function () {
                call("set_emulator_path", { id: id, path: null }, function (ok) {
                    if (ok) { say(t("Détection automatique.", "Automatic detection.")); reload(render); }
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
        const removeGame = g => row(code(g.path) + " " + small(sysName(g.system)), confirmRemove === g.path ? t("Confirmer ?", "Confirm?") : "✕", function () {
            if (confirmRemove !== g.path) { confirmRemove = g.path; return; }
            confirmRemove = null;
            call("remove_game", { path: g.path }, function (ok) { if (ok) { say(t("Jeu retiré.", "Game removed.")); reload(render); libraryChanged(); } });
        }, { danger: confirmRemove === g.path });
        const items = [block(t("Jeux ajoutés directement", "Games added directly"), c.imported_games.length ? "" : "—")];
        for (const g of c.imported_games) items.push(removeGame(g));
        items.push(row(t("+ Ajouter un jeu…", "+ Add a game…"), "📄", function () {
            pickFile(t("Choisir un fichier de jeu", "Choose a game file"), function (p) {
                if (!p) return;
                pendingGamePath = p;
                push("gameSystem", p);
            });
        }));
        items.push(block(t("Dossiers de jeux", "Game folders"), c.rom_dirs.length ? "" : "—"));
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

    function pageGameSystem(path) {
        const dot = path.lastIndexOf(".");
        const ext = dot >= 0 ? path.slice(dot + 1).toLowerCase() : "";
        const choices = ov.systems.filter(s => s.info.extensions.indexOf(ext) >= 0);
        const items = [block(t("Ce fichier appartient à quelle console ?", "Which console is this file for?"), code(path))];
        for (const s of choices) {
            items.push(row(esc(s.info.name), ext.toUpperCase(), function () {
                call("add_game", { path: path, system: s.info.id }, function (ok) {
                    if (!ok) return;
                    pendingGamePath = "";
                    say(t("Jeu ajouté.", "Game added."));
                    libraryChanged();
                    reload(pop);
                });
            }));
        }
        if (!choices.length)
            items.push(block(t("Format non reconnu", "Unknown format"), t(
                "Aucune console prise en charge ne correspond à ce fichier.",
                "No supported console matches this file.")));
        items.push(backRow());
        return { title: t("Console du jeu", "Game console"), items: items };
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
        main: pageMain, general: pageGeneral, controls: pageControls, performance: pagePerformance,
        image: pageImage, saves: pageSaves, help: pageHelp, advanced: pageAdvanced, storage: pageFolders,
        diagnostics: pageDiagnostics, systems: pageSystems, system: pageSystem,
        emulators: pageEmulators, emulator: pageEmulator,
        customSystems: pageCustomSystems, folders: pageLibrary, gameSystem: pageGameSystem, capture: pageCapture
    })

    function prefs(update) {
        call("update_preferences", update, function (ok) {
            if (!ok) return;
            reload(render);
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
        if (busy) return;
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
        if (busy) return;
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
            else if (target && target.system) push("system", target.system);
            else if (target && target.page && pages[target.page]) push(target.page);
            else render();
        });
    }
    function close() {
        controlTesterOpen = false;
        Launcher.controlCapture = false;
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
    Rectangle { anchors.fill: parent; color: Qt.rgba(0, 0, 0, 0.18) }
    MouseArea { anchors.fill: parent }   // blocks clicks to the theme

    Rectangle {
        id: panel
        anchors.fill: parent
        anchors.margins: 14
        radius: 0
        color: root.panelColor
        border.color: "#bcbcb7"

        Item {
            id: header
            x: 24; y: 14; width: parent.width - 48; height: 70

            Rectangle {
                id: backButton
                visible: root.stack.length > 1
                x: 0; y: 18; width: 42; height: 38
                radius: 6
                color: backMouse.containsMouse ? "#dededb" : "transparent"
                border.color: "#bcbcb7"
                Accessible.role: Accessible.Button
                Accessible.name: root.t("Retour", "Back")
                Text {
                    anchors.centerIn: parent
                    text: "←"
                    color: root.text
                    font.pixelSize: 22
                }
                MouseArea {
                    id: backMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.pop()
                }
            }

            Item {
                x: backButton.visible ? 54 : 0
                y: 0
                width: header.width - x - closeButton.width - 14
                height: header.height
                Text { x: 0; y: 0; width: parent.width; height: 18; visible: root.crumbs !== ""; text: root.crumbs; textFormat: Text.StyledText; elide: Text.ElideLeft; color: root.text; opacity: 0.55; font.pixelSize: 12 }
                Text { x: 0; y: 24; width: parent.width; text: root.busy ? "…" : root.pageTitle; textFormat: Text.StyledText; elide: Text.ElideRight; color: root.text; font.pixelSize: 26; font.weight: Font.DemiBold }
            }

            Rectangle {
                id: closeButton
                anchors.right: parent.right
                y: 18; width: 42; height: 38
                radius: 6
                color: closeMouse.containsMouse ? "#dededb" : "transparent"
                border.color: "#bcbcb7"
                Accessible.role: Accessible.Button
                Accessible.name: root.t("Fermer les réglages", "Close settings")
                Text {
                    anchors.centerIn: parent
                    text: "×"
                    color: root.text
                    font.pixelSize: 22
                }
                MouseArea {
                    id: closeMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.close()
                }
            }
        }


        Rectangle {
            x: 16; y: 92; width: 226
            anchors.bottom: footer.top; anchors.bottomMargin: 8
            radius: 12; color: "#e8edf5"
            Column {
                x: 8; y: 8; width: parent.width - 16; spacing: 4
                Repeater {
                    model: root.sections
                    Rectangle {
                        required property var modelData
                        width: 210; height: 48; radius: 8
                        color: root.activeSection() === modelData.id ? "#d1e0f7" : navMouse.containsMouse ? "#dde5f0" : "transparent"
                        Text {
                            anchors.fill: parent; anchors.margins: 12
                            text: root.t(modelData.fr,modelData.en)
                            color: "#17324d"; font.pixelSize: 14
                            font.bold: root.activeSection() === modelData.id
                            verticalAlignment: Text.AlignVCenter
                            wrapMode: Text.WordWrap
                        }
                        MouseArea { id: navMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.switchSection(modelData.id) }
                    }
                }
            }
        }

        ListView {
            id: list
            anchors.top: header.bottom
            anchors.bottom: footer.top; anchors.bottomMargin: 8
            x: 266; width: parent.width - 290
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
            x: 1; width: parent.width - 2; height: root.toast !== "" ? 92 : 48
            color: root.panelColor; radius: 0
            Rectangle {
                id: toastBox
                visible: root.toast !== ""
                x: 23; y: 0; width: parent.width - 46; height: 40; radius: 10
                color: "#dce8fa"
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
                        Rectangle { width: k.implicitWidth + 12; height: 20; radius: 6; color: "#dededb"
                                    Text { id: k; anchors.centerIn: parent; text: modelData[0]; color: root.text; font.pixelSize: 12 } }
                        Text { text: modelData[1]; color: root.text; opacity: 0.8; font.pixelSize: 13; anchors.verticalCenter: parent.verticalCenter }
                    }
                }
            }
        }
    }

    ControlVisualizer {
        anchors.fill: parent
        visible: root.controlTesterOpen
        z: 20
        onCloseRequested: {
            root.controlTesterOpen = false
            Launcher.controlCapture = false
        }
    }
}
