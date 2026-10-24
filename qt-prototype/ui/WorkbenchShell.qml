// Conventional desktop shell: menu bar, command toolbar, work area and
// status bar. The library is the primary workspace; settings and support
// stay out of the normal game-launching flow.

import QtQuick
import QtQuick.Window
import QtQuick.Controls
import QtQuick.Dialogs

Window {
    id: win
    visible: true
    width: 1366
    height: 880
    minimumWidth: 1024
    minimumHeight: 680
    title: "Console Launcher"
    color: "#f5f5f5"

    readonly property bool fr: Launcher.language.indexOf("fr") === 0
    readonly property color ink: "#202020"
    readonly property color muted: "#666666"
    readonly property color line: "#c7c7c7"
    readonly property color selected: "#ededed"
    readonly property color action: "#202020"

    property var overview: null
    property string selectedSystem: ""
    property int gameIndex: 0
    property string searchText: ""
    property var gameInfo: null
    property int pendingRequests: 0
    readonly property bool busy: pendingRequests > 0
    property int overviewRequestId: 0
    property string message: ""
    property string pendingGamePath: ""
    property var compatibleSystems: []
    property bool chooseSystemOpen: false
    property bool catalogueVisible: true

    readonly property var catalogue: overview ? overview.systems : Launcher.systems
    readonly property var selectedSystemData: {
        for (const system of catalogue)
            if (system.info.id === selectedSystem) return system
        return catalogue.length ? catalogue[0] : null
    }
    readonly property var shownGames: Launcher.games.filter(function(game) {
        const systemMatches = !selectedSystem || game.system === selectedSystem
        const textMatches = !searchText || game.title.toLowerCase().indexOf(searchText.toLowerCase()) >= 0
        return systemMatches && textMatches
    })
    readonly property var currentGame: shownGames.length ? shownGames[Math.min(gameIndex, shownGames.length - 1)] : null

    function t(french, english) { return fr ? french : english }
    function status(text) { message = text; statusTimer.restart() }
    function request(method, params, done) {
        pendingRequests += 1
        Launcher.internalRequest(method, params || {}, function(ok, result) {
            pendingRequests = Math.max(0, pendingRequests - 1)
            if (!ok) status(t("Erreur : ", "Error: ") + result)
            if (done) done(ok, result)
        })
    }
    function loadOverview(done) {
        const requestId = ++overviewRequestId
        request("settings_overview", {}, function(ok, result) {
            if (ok && requestId === overviewRequestId) {
                overview = result
                if (!selectedSystem && result.systems.length) selectedSystem = result.systems[0].info.id
            }
            if (done) done(ok)
        })
    }
    function emulatorName(id) {
        if (!overview) return id || t("Automatique", "Automatic")
        for (const emulator of overview.emulators) if (emulator.id === id) return emulator.name
        return id || t("Automatique", "Automatic")
    }
    function chooseGame() {
        if (overview) gameDialog.open()
        else loadOverview(function(ok) { if (ok) gameDialog.open() })
    }
    function gameChosen(path) {
        pendingGamePath = path
        const name = String(path).split(/[\\/]/).pop()
        const dot = name.lastIndexOf(".")
        const extension = dot < 0 ? "" : name.slice(dot + 1).toLowerCase()
        compatibleSystems = overview.systems.filter(s => s.info.extensions.indexOf(extension) >= 0)
        if (!compatibleSystems.length) status(t("Format de jeu non reconnu.", "Unsupported game format."))
        else if (compatibleSystems.length === 1) addGame(compatibleSystems[0])
        else chooseSystemOpen = true
    }
    function addGame(system) {
        chooseSystemOpen = false
        request("add_game", { path: pendingGamePath, system: system.info.id }, function(ok) {
            if (!ok) return
            selectedSystem = system.info.id
            gameIndex = 0
            Launcher.reload()
            loadOverview()
            status(t("Jeu ajouté.", "Game added."))
        })
    }
    function addFolder(path) {
        request("add_path", { path: path }, function(ok) {
            if (!ok) return
            Launcher.reload()
            loadOverview()
            status(t("Dossier analysé.", "Folder scanned."))
        })
    }
    function removeCurrentGame() {
        if (!currentGame) return
        request("remove_game", { path: currentGame.path }, function(ok) {
            if (!ok) return
            gameIndex = Math.max(0, gameIndex - 1)
            Launcher.reload()
            loadOverview()
            status(t("Entrée retirée sans supprimer le fichier.", "Entry removed without deleting the file."))
        })
    }
    function inspectCurrentGame() {
        if (!currentGame) return
        gameInfo = null
        Launcher.requestInfo(currentGame.id)
        status(t("Préparation vérifiée.", "Preparation checked."))
    }
    function launchCurrentGame() {
        if (!currentGame) return
        if (!gameInfo) {
            inspectCurrentGame()
            status(t("Vérifie d'abord la préparation.", "Check preparation first."))
            return
        }
        if (!gameInfo.ready) {
            Launcher.openSettings(gameInfo.emulatorId ? { emulator: gameInfo.emulatorId } : { system: currentGame.system })
            return
        }
        Launcher.play(currentGame.id)
    }
    function selectSystem(id) {
        selectedSystem = id
        gameIndex = 0
        gameInfo = null
    }
    function refreshLibrary() {
        Launcher.reload()
        loadOverview()
        status(t("Bibliothèque actualisée.", "Library refreshed."))
    }
    function toggleCurrentFavorite() {
        if (currentGame) Launcher.toggleFavorite(currentGame.id)
    }
    function handle(actionName) {
        if (Launcher.settingsOpen || chooseSystemOpen) return
        if (actionName === "up" && shownGames.length) gameIndex = Math.max(0, gameIndex - 1)
        else if (actionName === "down" && shownGames.length) gameIndex = Math.min(shownGames.length - 1, gameIndex + 1)
        else if (actionName === "confirm") launchCurrentGame()
        else if (actionName === "favorite" && currentGame) Launcher.toggleFavorite(currentGame.id)
        else if (actionName === "menu") Launcher.openSettings()
    }

    onCurrentGameChanged: gameInfo = null
    onShownGamesChanged: gameIndex = Math.min(gameIndex, Math.max(0, shownGames.length - 1))

    Timer { id: statusTimer; interval: 6000; onTriggered: win.message = "" }
    Connections {
        target: Launcher
        function onNav(actionName) { win.handle(actionName) }
        function onLibraryChanged() { if (!win.overview) win.loadOverview() }
        function onGameInfoReady(id, data) {
            if (win.currentGame && win.currentGame.id === id) win.gameInfo = data
        }
        function onFavoriteChanged() { Launcher.reload() }
        function onSettingsOpenChanged() {
            if (!Launcher.settingsOpen) { win.loadOverview(); Launcher.reload() }
        }
    }
    Component.onCompleted: loadOverview()

    FileDialog {
        id: gameDialog
        title: win.t("Ajouter un jeu", "Add a game")
        nameFilters: [win.t("Jeux pris en charge (*.rvz *.iso *.gcm *.wbfs *.ciso *.wia *.chd *.cue *.m3u *.zip *.7z *.nes *.sfc *.smc *.n64 *.z64 *.gba *.gb *.gbc *.nds *.md *.gen *.sms *.gg *.pbp *.cso *.wua *.wux *.wud)",
                            "Supported games (*.rvz *.iso *.gcm *.wbfs *.ciso *.wia *.chd *.cue *.m3u *.zip *.7z *.nes *.sfc *.smc *.n64 *.z64 *.gba *.gb *.gbc *.nds *.md *.gen *.sms *.gg *.pbp *.cso *.wua *.wux *.wud)"), win.t("Tous les fichiers (*)", "All files (*)")]
        onAccepted: win.gameChosen(Launcher.localPath(selectedFile))
    }
    FolderDialog {
        id: folderDialog
        title: win.t("Scanner un dossier", "Scan a folder")
        onAccepted: win.addFolder(Launcher.localPath(selectedFolder))
    }

    MenuBar {
        id: menuBar
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 30
        background: Rectangle { color: "#f2f2f2"; border.color: win.line }

        Menu {
            title: win.t("&Fichier", "&File")
            Action { text: win.t("Ajouter un jeu…", "Add game…"); shortcut: "Ctrl+O"; onTriggered: win.chooseGame() }
            Action { text: win.t("Scanner un dossier…", "Scan folder…"); shortcut: "Ctrl+Shift+O"; onTriggered: folderDialog.open() }
            MenuSeparator { }
            Action { text: win.t("Actualiser la bibliothèque", "Refresh library"); shortcut: "F5"; onTriggered: win.refreshLibrary() }
            MenuSeparator { }
            Action { text: win.t("Quitter", "Exit"); shortcut: "Ctrl+Q"; onTriggered: Launcher.quit() }
        }
        Menu {
            title: win.t("&Modifier", "&Edit")
            Action {
                text: win.currentGame && win.currentGame.favorite
                      ? win.t("Retirer des favoris", "Remove from favourites")
                      : win.t("Ajouter aux favoris", "Add to favourites")
                shortcut: "Ctrl+D"
                enabled: win.currentGame !== null
                onTriggered: win.toggleCurrentFavorite()
            }
            Action { text: win.t("Retirer de la bibliothèque", "Remove from library"); shortcut: "Delete"; enabled: win.currentGame !== null; onTriggered: win.removeCurrentGame() }
            MenuSeparator { }
            Action { text: win.t("Configurer la console sélectionnée…", "Configure selected system…"); enabled: win.selectedSystemData !== null; onTriggered: Launcher.openSettings({ system: win.selectedSystem }) }
            Action { text: win.t("Réglages…", "Settings…"); shortcut: "Ctrl+,"; onTriggered: Launcher.openSettings() }
        }
        Menu {
            title: win.t("&Affichage", "&View")
            Action { text: win.t("Rechercher", "Search"); shortcut: "Ctrl+F"; onTriggered: searchField.forceActiveFocus() }
            Action { text: win.t("Catalogue des consoles", "System catalogue"); checkable: true; checked: win.catalogueVisible; onTriggered: win.catalogueVisible = checked }
            MenuSeparator { }
            Action { text: win.t("Plein écran", "Fullscreen"); shortcut: "F11"; onTriggered: Launcher.toggleFullscreen() }
        }
        Menu {
            title: win.t("&Aide", "&Help")
            Action { text: win.t("Commandes clavier et manette", "Keyboard and controller commands"); onTriggered: Launcher.openSettings({ page: "controls" }) }
            Action { text: win.t("État et diagnostic", "Status and diagnostics"); onTriggered: Launcher.openSettings({ page: "diagnostics" }) }
            MenuSeparator { }
            Action { text: win.t("À propos de Console Launcher", "About Console Launcher"); onTriggered: aboutDialog.open() }
        }
    }

    ToolBar {
        id: commandBar
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: menuBar.bottom
        height: 48
        background: Rectangle { color: "#ffffff"; border.color: win.line }

        Row {
            anchors.left: parent.left
            anchors.leftMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            spacing: 4
            ToolButton { text: win.t("Ajouter…", "Add…"); onClicked: win.chooseGame(); ToolTip.visible: hovered; ToolTip.text: win.t("Ajouter un fichier de jeu (Ctrl+O)", "Add a game file (Ctrl+O)") }
            ToolButton { text: win.t("Dossier…", "Folder…"); onClicked: folderDialog.open(); ToolTip.visible: hovered; ToolTip.text: win.t("Scanner un dossier de jeux", "Scan a game folder") }
            ToolButton { text: win.t("Actualiser", "Refresh"); onClicked: win.refreshLibrary(); ToolTip.visible: hovered; ToolTip.text: win.t("Actualiser la bibliothèque (F5)", "Refresh library (F5)") }
            Rectangle { width: 1; height: 28; color: win.line; anchors.verticalCenter: parent.verticalCenter }
            ToolButton { text: win.t("Lancer", "Launch"); font.bold: true; enabled: win.currentGame !== null && !win.busy; onClicked: win.launchCurrentGame(); ToolTip.visible: hovered; ToolTip.text: win.t("Lancer le jeu sélectionné", "Launch selected game") }
        }

        TextField {
            id: searchField
            anchors.right: inputStatus.left
            anchors.rightMargin: 10
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(300, Math.max(190, parent.width * 0.24))
            height: 34
            placeholderText: win.t("Rechercher un jeu", "Search games")
            Accessible.name: placeholderText
            onTextChanged: { win.searchText = text; win.gameIndex = 0 }
        }

        Rectangle {
            id: inputStatus
            anchors.right: parent.right
            anchors.rightMargin: 10
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(220, inputStatusText.implicitWidth + 24)
            height: 30
            radius: 3
            color: "#f4f4f4"
            border.color: win.line
            Text {
                id: inputStatusText
                anchors.centerIn: parent
                text: Launcher.padName ? win.t("Manette : ", "Controller: ") + Launcher.padName : win.t("Clavier actif", "Keyboard active")
                color: win.muted
                font.pixelSize: 11
                elide: Text.ElideRight
                width: Math.min(196, implicitWidth)
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }

    Dialog {
        id: aboutDialog
        anchors.centerIn: parent
        modal: true
        title: win.t("À propos de Console Launcher", "About Console Launcher")
        standardButtons: Dialog.Ok
        width: 460
        contentItem: Column {
            spacing: 10
            Text { text: "Console Launcher " + Launcher.version; color: win.ink; font.pixelSize: 20; font.bold: true }
            Text { width: 410; wrapMode: Text.Wrap; text: win.t("Bibliothèque et lanceur d'émulation pour Windows et Linux. Aucun jeu ni BIOS n'est fourni.", "Emulation library and launcher for Windows and Linux. No games or BIOS files are provided."); color: win.muted; font.pixelSize: 13 }
            Text { width: 410; wrapMode: Text.Wrap; text: win.t("Interface : Qt Quick · Moteur : Rust", "Interface: Qt Quick · Engine: Rust"); color: win.muted; font.pixelSize: 12 }
        }
    }

    Rectangle {
        id: cataloguePane
        visible: win.catalogueVisible
        anchors.left: parent.left
        anchors.top: commandBar.bottom
        anchors.bottom: statusBar.top
        width: 260
        color: "#f4f4f4"
        border.color: win.line

        Text { x: 16; y: 15; text: win.t("CATALOGUE DES CONSOLES", "SYSTEM CATALOGUE"); color: win.ink; font.pixelSize: 12; font.bold: true }
        Text { x: 16; y: 38; width: parent.width - 32; text: win.t("Une console listée n'est pas forcément déjà configurée.", "A listed system is not necessarily configured yet."); wrapMode: Text.Wrap; color: win.muted; font.pixelSize: 11 }

        ListView {
            id: systemList
            x: 10
            y: 78
            width: parent.width - 20
            height: parent.height - 88
            clip: true
            spacing: 1
            model: win.catalogue
            delegate: Rectangle {
                required property var modelData
                width: systemList.width
                height: 50
                color: win.selectedSystem === modelData.info.id ? win.selected : "#ffffff"
                border.color: win.selectedSystem === modelData.info.id ? win.action : "#d7d7d7"
                Text { x: 10; y: 8; width: parent.width - 65; text: modelData.info.name; elide: Text.ElideRight; color: win.ink; font.pixelSize: 14; font.bold: true }
                Text { x: 10; y: 28; text: modelData.info.family || ""; color: win.muted; font.pixelSize: 11 }
                Text { anchors.right: parent.right; anchors.rightMargin: 10; anchors.verticalCenter: parent.verticalCenter; text: modelData.games || 0; color: win.muted; font.pixelSize: 12 }
                MouseArea { anchors.fill: parent; onClicked: win.selectSystem(modelData.info.id) }
            }
        }
    }

    Item {
        id: workspace
        anchors.left: win.catalogueVisible ? cataloguePane.right : parent.left
        anchors.right: parent.right
        anchors.top: commandBar.bottom
        anchors.bottom: statusBar.top

        Rectangle {
            id: systemHeading
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            height: 70
            color: "#ffffff"
            border.color: win.line
            Text { x: 16; y: 10; width: parent.width - configureSystemButton.width - 48; text: win.selectedSystemData ? win.selectedSystemData.info.name : win.t("Aucune console", "No system"); elide: Text.ElideRight; color: win.ink; font.pixelSize: 23; font.bold: true }
            Text { x: 16; y: 40; width: parent.width - configureSystemButton.width - 48; text: win.selectedSystemData ? win.selectedSystemData.games + " " + win.t("jeu(x) dans la bibliothèque", "game(s) in the library") : ""; elide: Text.ElideRight; color: win.muted; font.pixelSize: 12 }
            Button {
                id: configureSystemButton
                anchors.right: parent.right
                anchors.rightMargin: 14
                anchors.verticalCenter: parent.verticalCenter
                text: win.t("Configurer cette console…", "Configure this system…")
                enabled: win.selectedSystemData !== null
                onClicked: Launcher.openSettings({ system: win.selectedSystem })
            }
        }

        Item {
            id: pages
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: systemHeading.bottom
            anchors.bottom: parent.bottom

            Item {
                anchors.fill: parent

                Rectangle {
                    id: preparation
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 138
                    color: "#fafafa"
                    border.color: win.line
                    Text { x: 14; y: 10; text: win.t("Préparation de la partie", "Game preparation"); color: win.ink; font.pixelSize: 14; font.bold: true }
                    Text {
                        x: 18; y: 38; width: parent.width - 36; height: 42
                        text: !win.currentGame ? win.t("Sélectionne un jeu.", "Select a game.")
                              : !win.gameInfo ? win.t("Clique sur « Vérifier la préparation » avant le premier lancement.", "Click “Check preparation” before the first launch.")
                              : win.gameInfo.ready ? win.t("Prêt avec ", "Ready with ") + win.gameInfo.emulator
                              : win.gameInfo.error
                        wrapMode: Text.Wrap
                        color: win.gameInfo && !win.gameInfo.ready ? "#a12622" : win.ink
                        font.pixelSize: 13
                    }
                    Row {
                        x: 14; anchors.bottom: parent.bottom; anchors.bottomMargin: 14; spacing: 8
                        Button { text: win.t("Vérifier la préparation", "Check preparation"); enabled: win.currentGame !== null && !win.busy; onClicked: win.inspectCurrentGame() }
                        Button { text: win.t("Lancer le jeu", "Launch game"); enabled: win.currentGame !== null && !win.busy; font.bold: true; onClicked: win.launchCurrentGame() }
                    }
                }

                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.bottom: preparation.top
                    color: "#ffffff"
                    border.color: win.line
                    Rectangle {
                        id: tableHeader
                        anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
                        height: 34; color: "#ededed"; border.color: win.line
                        Text { x: 12; anchors.verticalCenter: parent.verticalCenter; text: win.t("Jeu", "Game"); color: win.ink; font.bold: true }
                        Text { x: parent.width * 0.55; anchors.verticalCenter: parent.verticalCenter; text: win.t("État", "Status"); color: win.ink; font.bold: true }
                        Text { anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; text: win.t("Fichier local", "Local file"); color: win.ink; font.bold: true }
                    }
                    ListView {
                        id: gamesList
                        anchors.left: parent.left; anchors.right: parent.right; anchors.top: tableHeader.bottom; anchors.bottom: parent.bottom
                        model: win.shownGames
                        currentIndex: win.gameIndex
                        clip: true
                        delegate: Rectangle {
                            required property int index
                            required property var modelData
                            width: gamesList.width; height: 40
                            color: index === win.gameIndex ? win.selected : (index % 2 ? "#f8f8f8" : "#ffffff")
                            border.color: "#dedede"
                            Text { x: 12; width: parent.width * 0.52; anchors.verticalCenter: parent.verticalCenter; text: (modelData.favorite ? "★  " : "") + modelData.title; elide: Text.ElideRight; color: win.ink; font.pixelSize: 13 }
                            Text { x: parent.width * 0.55; anchors.verticalCenter: parent.verticalCenter; text: win.t("Présent", "Available"); color: "#237a30"; font.pixelSize: 12 }
                            Text { anchors.right: parent.right; anchors.rightMargin: 12; width: parent.width * 0.35; horizontalAlignment: Text.AlignRight; anchors.verticalCenter: parent.verticalCenter; text: modelData.path; elide: Text.ElideMiddle; color: win.muted; font.pixelSize: 11 }
                            MouseArea { anchors.fill: parent; onClicked: win.gameIndex = index; onDoubleClicked: { win.gameIndex = index; win.inspectCurrentGame() } }
                        }
                    }
                    Column {
                        visible: win.shownGames.length === 0
                        anchors.centerIn: parent
                        width: Math.min(440, parent.width - 40)
                        spacing: 10
                        Text { width: parent.width; horizontalAlignment: Text.AlignHCenter; text: win.searchText ? win.t("Aucun résultat", "No result") : win.t("Aucun jeu pour cette console", "No game for this system"); color: win.ink; font.pixelSize: 18; font.bold: true }
                        Text { width: parent.width; horizontalAlignment: Text.AlignHCenter; wrapMode: Text.Wrap; text: win.searchText ? win.t("Modifie la recherche.", "Change the search.") : win.t("Utilise « Ajouter » ou « Dossier » dans la barre d'outils.", "Use “Add” or “Folder” in the toolbar."); color: win.muted; font.pixelSize: 13 }
                    }
                }
            }

        }
    }

    Rectangle {
        id: statusBar
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: 30
        color: "#eeeeee"
        border.color: win.line
        Text { x: 8; anchors.verticalCenter: parent.verticalCenter; text: win.message || (win.busy ? win.t("Traitement en cours…", "Working…") : win.t("Disponible — Aucune partie en cours.", "Ready — No game running.")); color: win.message ? win.action : win.muted; font.pixelSize: 11 }
        Text { anchors.right: parent.right; anchors.rightMargin: 8; anchors.verticalCenter: parent.verticalCenter; text: "Console Launcher " + Launcher.version; color: win.muted; font.pixelSize: 11 }
    }

    Rectangle {
        visible: win.chooseSystemOpen
        anchors.fill: parent
        z: 900
        color: Qt.rgba(0, 0, 0, 0.30)
        MouseArea { anchors.fill: parent }
        Rectangle {
            anchors.centerIn: parent
            width: 520
            height: 420
            color: "#ffffff"
            border.color: win.line
            Text { x: 18; y: 16; text: win.t("Choisir la console", "Choose the system"); color: win.ink; font.pixelSize: 20; font.bold: true }
            Text { x: 18; y: 48; text: win.t("Ce format est compatible avec plusieurs consoles.", "This format is compatible with several systems."); color: win.muted; font.pixelSize: 12 }
            ListView {
                id: choices
                x: 18; y: 80; width: parent.width - 36; height: 276
                model: win.compatibleSystems; spacing: 6; clip: true
                delegate: Button { required property var modelData; width: choices.width; text: modelData.info.name; onClicked: win.addGame(modelData) }
            }
            Button { x: 18; anchors.bottom: parent.bottom; anchors.bottomMargin: 14; text: win.t("Annuler", "Cancel"); onClicked: win.chooseSystemOpen = false }
        }
    }

    Rectangle {
        visible: Launcher.running
        anchors.fill: parent
        z: 950
        color: "#ffffff"
        Column {
            anchors.centerIn: parent; spacing: 8
            Text { anchors.horizontalCenter: parent.horizontalCenter; text: win.t("Partie en cours", "Game running"); color: win.ink; font.pixelSize: 22; font.bold: true }
            Text { anchors.horizontalCenter: parent.horizontalCenter; text: win.t("Le launcher revient quand l'émulateur se ferme.", "The launcher returns when the emulator closes."); color: win.muted; font.pixelSize: 13 }
        }
    }

    SettingsPanel { anchors.fill: parent; text: win.ink; accent: win.action; panelColor: "#f5f5f5" }
}
