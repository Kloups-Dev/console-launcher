// Horizon — consoles from left to right, games below.
// A theme is a folder with Main.qml whose root is a Window. It receives the
// `Launcher` object (see qt-prototype/README.*.md) and can use the built-in
// components of `import ConsoleLauncher` (the settings panel).

import QtQuick
import QtQuick.Window
import ConsoleLauncher

Window {
    id: win
    visible: true
    width: 1280
    height: 720
    minimumWidth: 800
    minimumHeight: 450
    color: bg
    title: "Console Launcher"

    // ---- Look: change these to restyle the theme ----
    readonly property color bg: "#0b0d12"
    readonly property color panel: Qt.rgba(1, 1, 1, 0.045)
    readonly property color text: "#eef1f7"
    readonly property color muted: "#8a93a6"
    readonly property color accent: "#ffc440"
    readonly property var makerColors: ({ "Nintendo": "#e4505a", "Sony": "#4f7cff", "Sega": "#20b3c9" })
    readonly property int pad: 48

    readonly property bool fr: Launcher.language.indexOf("fr") === 0
    readonly property var t: fr
        ? { games: "jeux", play: "Jouer", back: "Retour", fav: "Favori", menu: "Menu", missing: "émulateur introuvable",
            noGames: "Aucun jeu trouvé.\nOuvre le menu (Tab / Start) pour ajouter un dossier de jeux.",
            running: "Partie en cours…", runningHint: "Le lanceur revient quand l'émulateur se ferme.",
            settled: "Réglage trouvé", tuning: "Optimisation en cours", slow: "Machine en dessous du nécessaire",
            fix: "A : choisir l'émulateur", systemOnly: "Optimisations système" }
        : { games: "games", play: "Play", back: "Back", fav: "Favourite", menu: "Menu", missing: "emulator not found",
            noGames: "No game found.\nOpen the menu (Tab / Start) to add a game folder.",
            running: "Game running…", runningHint: "The launcher comes back when the emulator closes.",
            settled: "Settings found", tuning: "Optimising", slow: "Machine below requirements",
            fix: "A: choose the emulator", systemOnly: "System optimisations" }

    // ---- State ----
    property var systems: []
    property var bySystem: ({})
    property int sys: 0
    property var selected: ({})
    property string area: "systems"
    property var info: null

    readonly property var currentSystem: systems.length ? systems[sys] : null
    readonly property var currentGames: currentSystem ? (bySystem[currentSystem.info.id] || []) : []
    readonly property int gameIndex: currentSystem ? (selected[currentSystem.info.id] || 0) : 0
    readonly property var currentGame: currentGames.length ? currentGames[Math.min(gameIndex, currentGames.length - 1)] : null
    readonly property color sysColor: currentSystem ? (makerColors[currentSystem.info.manufacturer] || accent) : accent

    function rebuild() {
        const map = {};
        for (const g of Launcher.games) {
            if (!map[g.system]) map[g.system] = [];
            map[g.system].push(g);
        }
        bySystem = map;
        systems = Launcher.systems;
        sys = Math.min(sys, Math.max(0, systems.length - 1));
    }

    function setGameIndex(i) {
        const s = Object.assign({}, selected);
        s[currentSystem.info.id] = Math.max(0, Math.min(currentGames.length - 1, i));
        selected = s;
    }

    function moveSystem(d) {
        if (!systems.length) return;
        sys = (sys + d + systems.length) % systems.length;
    }

    function moveGame(d) {
        if (!currentGames.length) return;
        if (gameIndex + d < 0 && d === -1) { area = "systems"; return; }
        setGameIndex(gameIndex + d);
    }

    property bool pendingPlay: false

    function play() {
        if (!currentGame) return;
        // Details not loaded yet: act as soon as they arrive.
        if (!info) { pendingPlay = true; Launcher.requestInfo(currentGame.id); return; }
        pendingPlay = false;
        // Emulator missing: go straight to the place where it is chosen.
        if (info && !info.ready) {
            Launcher.openSettings(info.emulatorId ? { emulator: info.emulatorId } : { page: "systems" });
            return;
        }
        Launcher.play(currentGame.id);
    }

    function handle(a) {
        switch (a) {
        case "left": moveSystem(-1); break;
        case "right": moveSystem(1); break;
        case "down": if (area === "systems") area = "games"; else moveGame(1); break;
        case "up": if (area === "games") moveGame(-1); break;
        case "pageLeft": if (area === "games") moveGame(-10); else moveSystem(-1); break;
        case "pageRight": if (area === "games") moveGame(10); else moveSystem(1); break;
        case "confirm":
            if (area === "systems") area = "games";
            else play();
            break;
        case "back": if (area === "games") area = "systems"; break;
        case "favorite": if (currentGame) Launcher.toggleFavorite(currentGame.id); break;
        case "menu": Launcher.openSettings(); break;
        }
    }

    onCurrentGameChanged: { info = null; pendingPlay = false; infoDelay.restart(); }

    Timer {
        id: infoDelay
        interval: 140
        onTriggered: if (currentGame) Launcher.requestInfo(currentGame.id)
    }

    Connections {
        target: Launcher
        function onLibraryChanged() { rebuild(); }
        function onNav(a) { handle(a); }
        function onGameInfoReady(id, data) {
            if (!currentGame || currentGame.id !== id) return;
            info = data;
            if (pendingPlay) play();
        }
        function onFavoriteChanged(id, on) { rebuild(); }
        function onSessionEnded(report) { infoDelay.restart(); }
        function onSettingsOpenChanged() { if (!Launcher.settingsOpen) infoDelay.restart(); }
    }

    Component.onCompleted: rebuild()

    // ---- Background ----
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: Qt.tint(win.bg, Qt.rgba(win.sysColor.r, win.sysColor.g, win.sysColor.b, 0.22)) }
            GradientStop { position: 0.55; color: win.bg }
            GradientStop { position: 1.0; color: Qt.tint(win.bg, Qt.rgba(0.31, 0.35, 1.0, 0.10)) }
        }
    }

    // ---- Top bar ----
    Text {
        x: pad; y: 24
        text: "CONSOLE LAUNCHER"
        color: muted; font.pixelSize: 13; font.letterSpacing: 3
    }
    Row {
        anchors.right: parent.right; anchors.rightMargin: pad; y: 22
        spacing: 18
        Text { text: Launcher.padName ? "🎮 " + Launcher.padName : ""; color: muted; font.pixelSize: 14 }
        Text {
            id: clock
            color: muted; font.pixelSize: 16
            Timer { interval: 1000; running: true; repeat: true; triggeredOnStart: true
                    onTriggered: clock.text = Qt.formatTime(new Date(), "hh:mm") }
        }
    }

    // ---- Systems row ----
    ListView {
        id: systemsView
        x: pad; y: 68; width: parent.width - 2 * pad; height: 92
        orientation: ListView.Horizontal
        spacing: 14
        model: win.systems
        currentIndex: win.sys
        highlightMoveDuration: 250
        preferredHighlightBegin: width / 2 - 90; preferredHighlightEnd: width / 2 + 90
        highlightRangeMode: ListView.ApplyRange
        interactive: false
        delegate: Rectangle {
            readonly property bool current: index === win.sys
            readonly property color tone: win.makerColors[modelData.info.manufacturer] || win.accent
            width: Math.max(160, label.implicitWidth + 40, sub.implicitWidth + 40); height: 82; radius: 14
            color: win.panel
            border.width: 1
            border.color: current ? Qt.rgba(tone.r, tone.g, tone.b, 0.7) : "transparent"
            opacity: current ? 1 : 0.55
            scale: current && win.area === "systems" ? 1.06 : 1
            Behavior on scale { NumberAnimation { duration: 220; easing.type: Easing.OutCubic } }
            Behavior on opacity { NumberAnimation { duration: 180 } }
            Rectangle { x: 20; y: 16; width: 28; height: 3; radius: 2; color: tone }
            Text { id: label; x: 20; y: 28; text: modelData.info.name; color: win.text; font.pixelSize: 18; font.weight: Font.DemiBold }
            Text {
                id: sub
                x: 20; y: 54
                text: modelData.games + " " + win.t.games + " · " + (modelData.emulator_found ? modelData.emulator_name : win.t.missing)
                color: modelData.emulator_found ? win.muted : "#ff8a65"; font.pixelSize: 12
            }
        }
    }

    // ---- Games list ----
    ListView {
        id: gamesView
        x: pad; y: 180
        width: (parent.width - 2 * pad - 36) * 0.47
        height: parent.height - y - 70
        clip: true
        model: win.currentGames
        currentIndex: win.gameIndex
        highlightMoveDuration: 160
        preferredHighlightBegin: height / 2 - 24; preferredHighlightEnd: height / 2 + 24
        highlightRangeMode: ListView.ApplyRange
        interactive: false
        delegate: Rectangle {
            readonly property bool current: index === win.gameIndex
            width: gamesView.width; height: 46; radius: 10
            color: current ? (win.area === "games" ? Qt.rgba(1, 0.77, 0.25, 0.16) : win.panel) : "transparent"
            Rectangle { visible: current && win.area === "games"; width: 3; height: parent.height; color: win.accent; radius: 2 }
            Text {
                x: current && win.area === "games" ? 22 : 16
                anchors.verticalCenter: parent.verticalCenter
                text: modelData.title
                color: current ? win.text : win.muted; font.pixelSize: 18
                Behavior on x { NumberAnimation { duration: 150 } }
            }
            Text {
                visible: modelData.favorite
                anchors.right: parent.right; anchors.rightMargin: 16; anchors.verticalCenter: parent.verticalCenter
                text: "★"; color: win.accent; font.pixelSize: 16
            }
        }
        Text {
            visible: win.systems.length === 0
            text: win.t.noGames; color: win.muted; font.pixelSize: 16; lineHeight: 1.4
        }
    }

    // ---- Detail ----
    Item {
        id: detail
        visible: win.width >= 900 && win.currentGame !== null
        x: gamesView.x + gamesView.width + 36; y: 180
        width: parent.width - x - pad; height: parent.height - y - 70

        Rectangle {
            id: coverBox
            width: parent.width; height: Math.min(parent.height * 0.62, 440); radius: 14
            color: win.panel; clip: true
            gradient: cover.status === Image.Ready ? null : placeholder
            Gradient {
                id: placeholder
                orientation: Gradient.Horizontal
                GradientStop { position: 0; color: Qt.darker(win.sysColor, 2.2) }
                GradientStop { position: 0.7; color: "#111111" }
            }
            Image {
                id: cover
                anchors.fill: parent; anchors.margins: 0
                source: win.currentGame && win.currentGame.cover ? Launcher.fileUrl(win.currentGame.cover) : ""
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                sourceSize.width: 800
            }
            Text {
                visible: cover.status !== Image.Ready && win.currentGame
                anchors.centerIn: parent
                text: win.currentGame ? win.currentGame.title.split(" ").slice(0, 2).map(w => w[0]).join("").toUpperCase() : ""
                color: Qt.rgba(1, 1, 1, 0.45); font.pixelSize: 72; font.bold: true
            }
        }
        Text {
            id: title
            anchors.top: coverBox.bottom; anchors.topMargin: 16
            width: parent.width; elide: Text.ElideRight
            text: win.currentGame ? win.currentGame.title : ""
            color: win.text; font.pixelSize: 26; font.weight: Font.DemiBold
        }
        Flow {
            id: chips
            anchors.top: title.bottom; anchors.topMargin: 12; width: parent.width; spacing: 8
            Repeater {
                model: !win.info ? [] : (win.info.ready
                    ? [ { t: win.info.emulator, k: "" },
                        win.info.tunesSettings ? { t: win.info.stepLabel, k: "accent" } : { t: win.t.systemOnly, k: "" },
                        win.info.belowRequirements ? { t: win.t.slow, k: "warn" } : { t: win.info.settled ? win.t.settled : win.t.tuning, k: "" } ]
                    : [ { t: win.info.error, k: "warn" }, { t: win.t.fix, k: "" } ])
                delegate: Rectangle {
                    height: chipText.implicitHeight + 8; radius: 13; width: Math.min(chips.width, chipText.implicitWidth + 20)
                    color: modelData.k === "accent" ? win.accent : modelData.k === "warn" ? "#ff8a65" : win.panel
                    Text { id: chipText; anchors.centerIn: parent; text: modelData.t
                           width: Math.min(implicitWidth, chips.width - 20); wrapMode: Text.Wrap
                           color: modelData.k ? "#111111" : win.muted; font.pixelSize: 13 }
                }
            }
        }
        Column {
            anchors.top: chips.bottom; anchors.topMargin: 10; width: parent.width; spacing: 4
            Repeater {
                model: win.info && win.info.notes ? win.info.notes : []
                Text { width: parent.width; wrapMode: Text.Wrap; text: "• " + modelData; color: win.muted; font.pixelSize: 13 }
            }
        }
    }

    // ---- Help bar ----
    Row {
        anchors.right: parent.right; anchors.rightMargin: pad; anchors.bottom: parent.bottom; anchors.bottomMargin: 22
        spacing: 22
        Repeater {
            model: [["A", win.t.play], ["B", win.t.back], ["Y", win.t.fav], ["Start", win.t.menu]]
            Row {
                spacing: 6
                Rectangle { width: k.implicitWidth + 12; height: 20; radius: 6; color: win.panel; border.color: Qt.rgba(1, 1, 1, 0.12)
                            Text { id: k; anchors.centerIn: parent; text: modelData[0]; color: win.text; font.pixelSize: 12 } }
                Text { text: modelData[1]; color: win.muted; font.pixelSize: 13; anchors.verticalCenter: parent.verticalCenter }
            }
        }
    }

    // ---- Error banner ----
    Rectangle {
        visible: Launcher.lastError !== ""
        anchors.left: parent.left; anchors.leftMargin: pad; anchors.bottom: parent.bottom; anchors.bottomMargin: 18
        width: Math.min(parent.width * 0.5, errText.implicitWidth + 24); height: errText.implicitHeight + 12; radius: 8
        color: "#3a1418"; border.color: "#aa3333"
        Text { id: errText; anchors.centerIn: parent; width: parent.width - 24; wrapMode: Text.Wrap
               text: Launcher.lastError; color: win.text; font.pixelSize: 12 }
    }

    // ---- Session overlay ----
    Rectangle {
        visible: Launcher.running
        anchors.fill: parent
        color: Qt.rgba(0.02, 0.03, 0.05, 0.85)
        Column {
            anchors.centerIn: parent; spacing: 10
            Text { anchors.horizontalCenter: parent.horizontalCenter; text: win.t.running; color: win.text; font.pixelSize: 20 }
            Text { anchors.horizontalCenter: parent.horizontalCenter; text: win.t.runningHint; color: win.muted; font.pixelSize: 14 }
        }
    }

    // ---- Built-in settings panel (Start / Tab) ----
    SettingsPanel { anchors.fill: parent }
}
