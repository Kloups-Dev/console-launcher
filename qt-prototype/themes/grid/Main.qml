// Grid (Qt Quick version) — minimal template: cover grid with console tabs.

import QtQuick
import QtQuick.Window

Window {
    id: win
    visible: true
    width: 1280; height: 720
    color: "#0e1016"
    title: "Console Launcher — Qt Quick"

    readonly property color accent: "#5ad1a0"
    readonly property color muted: "#8a93a6"
    readonly property int columns: Math.max(3, Math.floor((width - 56) / 200))

    property var systems: []
    property var bySystem: ({})
    property int sys: 0
    property int sel: 0
    property var info: null

    readonly property var games: systems.length ? (bySystem[systems[sys].info.id] || []) : []
    readonly property var game: games.length ? games[Math.min(sel, games.length - 1)] : null

    function rebuild() {
        const m = {};
        for (const g of Launcher.games) { if (!m[g.system]) m[g.system] = []; m[g.system].push(g); }
        bySystem = m; systems = Launcher.systems;
        sys = Math.min(sys, Math.max(0, systems.length - 1));
    }
    function move(d) { if (games.length) sel = Math.max(0, Math.min(games.length - 1, sel + d)); }
    function switchSystem(d) { if (systems.length) { sys = (sys + d + systems.length) % systems.length; sel = 0; } }

    onGameChanged: { info = null; if (game) Launcher.requestInfo(game.id); }

    Connections {
        target: Launcher
        function onLibraryChanged() { rebuild(); }
        function onFavoriteChanged() { rebuild(); }
        function onGameInfoReady(id, data) { if (game && game.id === id) info = data; }
        function onNav(a) {
            if (a === "left") move(-1);
            else if (a === "right") move(1);
            else if (a === "up") move(-columns);
            else if (a === "down") move(columns);
            else if (a === "pageLeft") switchSystem(-1);
            else if (a === "pageRight") switchSystem(1);
            else if (a === "confirm" && game) Launcher.play(game.id);
            else if (a === "favorite" && game) Launcher.toggleFavorite(game.id);
            else if (a === "menu") { const l = Launcher.themes; Launcher.setTheme(l[(l.indexOf(Launcher.theme) + 1) % l.length]); }
        }
    }
    Component.onCompleted: rebuild()

    Row {
        x: 28; y: 18; spacing: 6
        Repeater {
            model: win.systems
            Rectangle {
                height: 36; radius: 18; width: tabText.implicitWidth + 32
                color: index === win.sys ? win.accent : "transparent"
                Text { id: tabText; anchors.centerIn: parent; text: modelData.info.name
                       color: index === win.sys ? "#0e1016" : win.muted; font.pixelSize: 15; font.bold: index === win.sys }
            }
        }
    }

    GridView {
        id: grid
        x: 28; y: 72; width: parent.width - 56; height: parent.height - 72 - 56
        cellWidth: width / win.columns; cellHeight: cellWidth * 4 / 3 + 16
        clip: true; interactive: false
        model: win.games
        currentIndex: win.sel
        highlightMoveDuration: 150
        delegate: Item {
            width: grid.cellWidth; height: grid.cellHeight
            Rectangle {
                anchors.fill: parent; anchors.margins: 8; radius: 12; color: "#171b25"; clip: true
                border.width: 3; border.color: index === win.sel ? win.accent : "transparent"
                scale: index === win.sel ? 1.04 : 1
                Behavior on scale { NumberAnimation { duration: 120 } }
                Image { anchors.fill: parent; fillMode: Image.PreserveAspectCrop; asynchronous: true; sourceSize.width: 400
                        source: modelData.cover ? Launcher.fileUrl(modelData.cover) : "" }
                Text { visible: !modelData.cover; x: 10; anchors.bottom: parent.bottom; anchors.bottomMargin: 10
                       width: parent.width - 20; wrapMode: Text.Wrap; text: modelData.title; color: "#eef1f7"; font.pixelSize: 13 }
                Text { visible: modelData.favorite; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: 8
                       text: "★"; color: win.accent }
            }
        }
    }

    Text {
        x: 28; anchors.bottom: parent.bottom; anchors.bottomMargin: 18
        text: win.game ? win.game.title + "  ·  " + win.systems[win.sys].info.name : "Aucun jeu / No games — add a folder from the Horizon theme menu"
        color: "#eef1f7"; font.pixelSize: 14
    }
    Text {
        anchors.right: parent.right; anchors.rightMargin: 28; anchors.bottom: parent.bottom; anchors.bottomMargin: 18
        text: !win.info ? "" : (win.info.ready ? win.info.emulator + " · " + win.info.stepLabel : win.info.error)
        color: win.muted; font.pixelSize: 14
    }
}
