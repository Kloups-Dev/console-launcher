// Clean desktop foundation. This file intentionally does not inherit the
// previous workshop layout: Qt owns the menu, header, content and footer
// regions so they cannot overlap or move when content changes.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs

ApplicationWindow {
    id: app
    visible: true
    width: 1280
    height: 800
    minimumWidth: 960
    minimumHeight: 640
    title: "Console Launcher"

    readonly property bool fr: Launcher.language.indexOf("fr") === 0
    property var overview: null
    property string selectedSystem: ""
    property int selectedGame: 0
    property var gameInfo: null
    property string notice: ""
    property int pending: 0
    property int overviewRequest: 0
    property string pendingGamePath: ""
    property var compatibleSystems: []
    property bool systemChooserOpen: false
    property bool sidebarVisible: true
    property var metadataRequested: ({})
    property string metadataLoadingId: ""

    // settings_overview contains the complete catalogue. Launcher.systems only
    // contains systems found in the game library, so it must not drive this list.
    readonly property var systems: overview ? overview.systems : []
    readonly property var filteredGames: Launcher.games.filter(function(game) {
        const bySystem = !selectedSystem || game.system === selectedSystem
        const query = searchField.text.trim().toLowerCase()
        return bySystem && (!query || game.title.toLowerCase().indexOf(query) >= 0)
    })
    readonly property var currentGame: filteredGames.length
        ? filteredGames[Math.min(selectedGame, filteredGames.length - 1)] : null
    readonly property var currentSystem: {
        for (const system of systems)
            if (system.info.id === selectedSystem) return system
        return systems.length ? systems[0] : null
    }
    readonly property var currentEmulator: {
        if (!overview || !currentSystem) return null
        for (const emulator of overview.emulators)
            if (emulator.id === currentSystem.emulator) return emulator
        return null
    }

    function t(french, english) { return fr ? french : english }
    function showNotice(text) { notice = text; noticeTimer.restart() }
    function call(method, params, done, silentError) {
        pending += 1
        Launcher.internalRequest(method, params || {}, function(ok, result) {
            pending = Math.max(0, pending - 1)
            if (!ok && !silentError) showNotice(t("Erreur : ", "Error: ") + result)
            if (done) done(ok, result)
        })
    }
    function loadOverview(done) {
        const id = ++overviewRequest
        call("settings_overview", {}, function(ok, result) {
            if (ok && id === overviewRequest) {
                overview = result
                if (!selectedSystem && result.systems.length)
                    selectedSystem = result.systems[0].info.id
            }
            if (done) done(ok)
        })
    }
    function refresh() {
        Launcher.reload()
        loadOverview()
        showNotice(t("Bibliothèque actualisée.", "Library refreshed."))
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
        compatibleSystems = overview.systems.filter(function(system) {
            return system.info.extensions.indexOf(extension) >= 0
        })
        if (!compatibleSystems.length)
            showNotice(t("Format de jeu non reconnu.", "Unsupported game format."))
        else if (compatibleSystems.length === 1)
            addGame(compatibleSystems[0])
        else
            systemChooserOpen = true
    }
    function addGame(system) {
        systemChooserOpen = false
        call("add_game", { path: pendingGamePath, system: system.info.id }, function(ok) {
            if (!ok) return
            selectedSystem = system.info.id
            selectedGame = 0
            Launcher.reload()
            loadOverview()
            showNotice(t("Jeu ajouté.", "Game added."))
        })
    }
    function addFolder(path) {
        call("add_path", { path: path }, function(ok) {
            if (!ok) return
            Launcher.reload()
            loadOverview()
            showNotice(t("Dossier analysé.", "Folder scanned."))
        })
    }
    function removeGame() {
        if (!currentGame) return
        call("remove_game", { path: currentGame.path }, function(ok) {
            if (!ok) return
            selectedGame = Math.max(0, selectedGame - 1)
            Launcher.reload()
            loadOverview()
            showNotice(t("Jeu retiré de la bibliothèque. Le fichier est conservé.",
                         "Game removed from the library. The file was kept."))
        })
    }
    function inspectGame() {
        if (!currentGame) return
        gameInfo = null
        Launcher.requestInfo(currentGame.id)
        showNotice(t("Vérification de la préparation…", "Checking game preparation…"))
    }
    function ensureCurrentMetadata() {
        if (!currentGame || currentGame.metadata || metadataRequested[currentGame.id]) return
        if (pending) {
            metadataTimer.restart()
            return
        }
        metadataRequested[currentGame.id] = true
        refreshGameMetadata(true, currentGame.id)
    }
    function refreshGameMetadata(automatic, requestedId) {
        const id = requestedId || (currentGame ? currentGame.id : "")
        if (!id || (!automatic && pending)) return
        metadataLoadingId = id
        if (!automatic)
            showNotice(t("Recherche de la fiche et de la jaquette…", "Looking up game details and cover…"))
        call("refresh_game_metadata", { id: id }, function(ok, result) {
            if (metadataLoadingId === id) metadataLoadingId = ""
            if (!ok) return
            if (!result) {
                if (!automatic)
                    showNotice(t("Aucune correspondance suffisamment fiable.", "No sufficiently reliable match."))
                return
            }
            Launcher.reload()
            if (!automatic)
                showNotice(result.cover_path
                           ? t("Fiche et jaquette enregistrées hors ligne.", "Details and cover saved for offline use.")
                           : result.cover_url
                             ? t("Fiche enregistrée, mais la jaquette n'a pas pu être mise en cache. Réessaie l'actualisation.",
                                 "Details saved, but the cover could not be cached. Retry the refresh.")
                             : t("Fiche récupérée depuis ", "Details fetched from ") + result.source + ".")
        }, automatic)
    }
    function launchGame() {
        if (!currentGame) return
        if (!gameInfo) {
            inspectGame()
            return
        }
        if (!gameInfo.ready) {
            Launcher.openSettings(gameInfo.emulatorId
                                  ? { emulator: gameInfo.emulatorId }
                                  : { system: currentGame.system })
            return
        }
        Launcher.play(currentGame.id)
    }
    function selectSystem(id) {
        selectedSystem = id
        selectedGame = 0
        gameInfo = null
    }
    function installCurrentEmulator() {
        if (!currentEmulator || pending) return
        const emulator = currentEmulator
        if (emulator.found) {
            Launcher.openSettings({ emulator: emulator.id })
            return
        }
        if (emulator.automatic_install) {
            showNotice(t("Téléchargement et installation de ", "Downloading and installing ") + emulator.name + "…")
            call("install_emulator", { id: emulator.id }, function(ok, result) {
                if (!ok) return
                showNotice(result.emulator + " " + result.version + t(" est installé et prêt.", " is installed and ready."))
                Launcher.reload()
                loadOverview()
            })
            return
        }
        if (emulator.download_page) {
            Qt.openUrlExternally(emulator.download_page)
            showNotice(t("Page officielle ouverte. Une fois installé, utilise ‘Choisir l’exécutable’. ",
                         "Official page opened. Once installed, use ‘Choose executable’. "))
            return
        }
        Launcher.openSettings({ emulator: emulator.id })
    }
    function emulatorActionText() {
        if (!currentEmulator) return t("Choisir un émulateur…", "Choose an emulator…")
        if (currentEmulator.found)
            return t("Configurer ", "Configure ") + currentEmulator.name + "…"
        if (currentEmulator.automatic_install)
            return t("Installer ", "Install ") + currentEmulator.name
        if (currentEmulator.download_page)
            return t("Télécharger ", "Download ") + currentEmulator.name + "…"
        return t("Choisir ", "Choose ") + currentEmulator.name + "…"
    }
    function gameCoverSource(game) {
        if (!game) return ""
        if (game.metadata && game.metadata.cover_path)
            return localFileSource(game.metadata.cover_path)
        if (!game.cover) return ""
        return localFileSource(game.cover)
    }
    function localFileSource(value) {
        let path = String(value).replace(/\\/g, "/")
        // Rust canonical paths on Windows may use the extended-length prefix.
        // QML expects an ordinary file URL instead.
        if (path.indexOf("//?/UNC/") === 0) path = "//" + path.slice(8)
        else if (path.indexOf("//?/") === 0) path = path.slice(4)
        return path.match(/^[A-Za-z]:\//) ? "file:///" + path : "file://" + path
    }
    function handleNavigation(action) {
        if (Launcher.settingsOpen || systemChooserOpen) return
        if (action === "up" && filteredGames.length)
            selectedGame = Math.max(0, selectedGame - 1)
        else if (action === "down" && filteredGames.length)
            selectedGame = Math.min(filteredGames.length - 1, selectedGame + 1)
        else if (action === "confirm") launchGame()
        else if (action === "favorite" && currentGame) Launcher.toggleFavorite(currentGame.id)
        else if (action === "menu") Launcher.openSettings()
    }

    onCurrentGameChanged: {
        gameInfo = null
        if (currentGame && !currentGame.metadata) metadataTimer.restart()
        else metadataTimer.stop()
    }
    onFilteredGamesChanged: selectedGame = Math.min(selectedGame, Math.max(0, filteredGames.length - 1))
    onPendingChanged: {
        if (!pending && currentGame && !currentGame.metadata
                && !metadataRequested[currentGame.id]) metadataTimer.restart()
    }

    Timer { id: noticeTimer; interval: 6000; onTriggered: app.notice = "" }
    Timer { id: metadataTimer; interval: 650; onTriggered: app.ensureCurrentMetadata() }
    Connections {
        target: Launcher
        function onNav(action) { app.handleNavigation(action) }
        function onLibraryChanged() { if (!app.overview) app.loadOverview() }
        function onGameInfoReady(id, data) {
            if (app.currentGame && app.currentGame.id === id) app.gameInfo = data
        }
        function onFavoriteChanged() { Launcher.reload() }
        function onSettingsOpenChanged() {
            if (!Launcher.settingsOpen) { app.loadOverview(); Launcher.reload() }
        }
    }
    Component.onCompleted: loadOverview()

    FileDialog {
        id: gameDialog
        title: app.t("Ajouter un jeu", "Add a game")
        nameFilters: [app.t("Jeux pris en charge (*.rvz *.iso *.gcm *.wbfs *.ciso *.wia *.chd *.cue *.m3u *.zip *.7z *.nes *.sfc *.smc *.n64 *.z64 *.gba *.gb *.gbc *.nds *.md *.gen *.sms *.gg *.pbp *.cso *.wua *.wux *.wud)",
                            "Supported games (*.rvz *.iso *.gcm *.wbfs *.ciso *.wia *.chd *.cue *.m3u *.zip *.7z *.nes *.sfc *.smc *.n64 *.z64 *.gba *.gb *.gbc *.nds *.md *.gen *.sms *.gg *.pbp *.cso *.wua *.wux *.wud)"), app.t("Tous les fichiers (*)", "All files (*)")]
        onAccepted: app.gameChosen(Launcher.localPath(selectedFile))
    }
    FolderDialog {
        id: folderDialog
        title: app.t("Ajouter un dossier de jeux", "Add a game folder")
        onAccepted: app.addFolder(Launcher.localPath(selectedFolder))
    }

    menuBar: MenuBar {
        Menu {
            title: app.t("&Fichier", "&File")
            Action { text: app.t("Ajouter un jeu…", "Add a game…"); shortcut: StandardKey.Open; onTriggered: app.chooseGame() }
            Action { text: app.t("Ajouter un dossier…", "Add a folder…"); shortcut: "Ctrl+Shift+O"; onTriggered: folderDialog.open() }
            MenuSeparator { }
            Action { text: app.t("Actualiser", "Refresh"); shortcut: StandardKey.Refresh; onTriggered: app.refresh() }
            MenuSeparator { }
            Action { text: app.t("Quitter", "Quit"); shortcut: StandardKey.Quit; onTriggered: Launcher.quit() }
        }
        Menu {
            title: app.t("&Modifier", "&Edit")
            Action { text: app.t("Ajouter aux favoris", "Add to favourites"); shortcut: "Ctrl+D"; enabled: app.currentGame !== null; onTriggered: Launcher.toggleFavorite(app.currentGame.id) }
            Action { text: app.t("Retirer de la bibliothèque", "Remove from library"); shortcut: StandardKey.Delete; enabled: app.currentGame !== null; onTriggered: app.removeGame() }
            MenuSeparator { }
            Action { text: app.t("Réglages…", "Settings…"); shortcut: "Ctrl+,"; onTriggered: Launcher.openSettings() }
        }
        Menu {
            title: app.t("&Affichage", "&View")
            Action { text: app.t("Rechercher", "Search"); shortcut: StandardKey.Find; onTriggered: searchField.forceActiveFocus() }
            Action { text: app.t("Catalogue des consoles", "System catalogue"); checkable: true; checked: app.sidebarVisible; onTriggered: app.sidebarVisible = checked }
            MenuSeparator { }
            Action { text: app.t("Plein écran", "Fullscreen"); shortcut: "F11"; onTriggered: Launcher.toggleFullscreen() }
        }
        Menu {
            title: app.t("&Aide", "&Help")
            Action { text: app.t("Commandes", "Controls"); onTriggered: Launcher.openSettings({ page: "controls" }) }
            Action { text: app.t("Diagnostic", "Diagnostics"); onTriggered: Launcher.openSettings({ page: "diagnostics" }) }
            MenuSeparator { }
            Action { text: app.t("À propos", "About"); onTriggered: aboutDialog.open() }
        }
    }

    header: ToolBar {
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 8
            anchors.rightMargin: 8
            spacing: 4

            ToolButton { text: app.t("Ajouter…", "Add…"); onClicked: app.chooseGame() }
            ToolButton { text: app.t("Dossier…", "Folder…"); onClicked: folderDialog.open() }
            ToolButton { text: app.t("Actualiser", "Refresh"); onClicked: app.refresh() }
            ToolSeparator { }
            ToolButton { text: app.t("Lancer", "Launch"); font.bold: true; enabled: app.currentGame !== null && app.pending === 0; onClicked: app.launchGame() }
            Item { Layout.fillWidth: true }
            TextField {
                id: searchField
                Layout.preferredWidth: 270
                placeholderText: app.t("Rechercher un jeu", "Search games")
                Accessible.name: placeholderText
            }
            Label {
                Layout.leftMargin: 8
                Layout.rightMargin: 4
                text: Launcher.padName
                      ? app.t("Manette : ", "Controller: ") + Launcher.padName
                      : app.t("Clavier", "Keyboard")
                color: app.palette.mid
            }
        }
    }

    footer: ToolBar {
        implicitHeight: 28
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 10
            anchors.rightMargin: 10
            Label {
                Layout.fillWidth: true
                text: app.notice || (app.pending
                      ? app.t("Traitement en cours…", "Working…")
                      : app.t("Prêt", "Ready"))
                elide: Text.ElideRight
                font.pixelSize: 11
            }
            Label { text: "Console Launcher " + Launcher.version; color: app.palette.mid; font.pixelSize: 11 }
        }
    }

    SplitView {
        anchors.fill: parent
        orientation: Qt.Horizontal

        Pane {
            id: sidebar
            visible: app.sidebarVisible
            SplitView.preferredWidth: 250
            SplitView.minimumWidth: 210
            SplitView.maximumWidth: 360
            padding: 0

            ColumnLayout {
                anchors.fill: parent
                spacing: 0
                Label {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 48
                    leftPadding: 14
                    verticalAlignment: Text.AlignVCenter
                    text: app.t("Consoles", "Systems")
                    font.bold: true
                    font.pixelSize: 16
                }
                Rectangle { Layout.fillWidth: true; height: 1; color: app.palette.mid }
                ListView {
                    id: systemList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: app.systems
                    delegate: ItemDelegate {
                        required property var modelData
                        width: systemList.width
                        height: 48
                        highlighted: app.selectedSystem === modelData.info.id
                        onClicked: app.selectSystem(modelData.info.id)
                        contentItem: RowLayout {
                            spacing: 8
                            Rectangle {
                                width: 8; height: 8; radius: 4
                                color: modelData.options.some(function(option) { return option.found }) ? "#287a34" : "#9a9a96"
                            }
                            Label {
                                Layout.fillWidth: true
                                text: modelData.info.name
                                elide: Text.ElideRight
                            }
                            Label {
                                text: String(modelData.games || 0)
                                color: app.palette.mid
                            }
                        }
                    }
                }
            }
        }

        Pane {
            SplitView.fillWidth: true
            padding: 0

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                Frame {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 92
                    padding: 12
                    RowLayout {
                        anchors.fill: parent
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2
                            Label { text: app.currentSystem ? app.currentSystem.info.name : app.t("Bibliothèque", "Library"); font.pixelSize: 20; font.bold: true }
                            Label {
                                text: !app.currentEmulator
                                      ? app.t("Choisis un émulateur", "Choose an emulator")
                                      : app.currentEmulator.found
                                        ? app.t("Prêt avec ", "Ready with ") + app.currentEmulator.name
                                        : app.currentEmulator.name + app.t(" n’est pas installé", " is not installed")
                                color: app.currentEmulator && app.currentEmulator.found ? "#287a34" : "#a15c12"
                            }
                            Label { text: app.filteredGames.length + app.t(" jeu(x) affiché(s)", " game(s) shown"); color: app.palette.mid; font.pixelSize: 11 }
                        }
                        Button {
                            text: app.emulatorActionText()
                            enabled: app.currentSystem !== null && app.pending === 0
                            font.bold: app.currentEmulator !== null && !app.currentEmulator.found
                            onClicked: app.installCurrentEmulator()
                        }
                        Button {
                            text: app.t("Options de la console…", "System options…")
                            enabled: app.currentSystem !== null
                            onClicked: Launcher.openSettings({ system: app.selectedSystem })
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 34
                    color: app.palette.alternateBase
                    border.color: app.palette.mid
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 12
                        anchors.rightMargin: 12
                        Label { Layout.preferredWidth: parent.width * 0.50; text: app.t("Jeu", "Game"); font.bold: true }
                        Label { Layout.preferredWidth: parent.width * 0.16; text: app.t("État", "Status"); font.bold: true }
                        Label { Layout.fillWidth: true; text: app.t("Fichier", "File"); font.bold: true; horizontalAlignment: Text.AlignRight }
                    }
                }

                ListView {
                    id: gameList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: app.filteredGames
                    currentIndex: app.selectedGame
                    delegate: ItemDelegate {
                        required property int index
                        required property var modelData
                        width: gameList.width
                        height: 42
                        highlighted: index === app.selectedGame
                        onClicked: app.selectedGame = index
                        onDoubleClicked: { app.selectedGame = index; app.launchGame() }
                        contentItem: RowLayout {
                            spacing: 12
                            Label { Layout.preferredWidth: parent.width * 0.50; text: (modelData.favorite ? "★  " : "") + modelData.title; elide: Text.ElideRight }
                            Label { Layout.preferredWidth: parent.width * 0.14; text: app.t("Disponible", "Available"); color: "#287a34" }
                            Label { Layout.fillWidth: true; text: modelData.path; elide: Text.ElideMiddle; horizontalAlignment: Text.AlignRight; color: app.palette.mid }
                        }
                    }
                    ColumnLayout {
                        visible: app.filteredGames.length === 0
                        anchors.centerIn: parent
                        width: Math.min(480, parent.width - 40)
                        spacing: 12
                        Label {
                            Layout.fillWidth: true
                            text: searchField.text
                                  ? app.t("Aucun jeu ne correspond à la recherche.", "No game matches the search.")
                                  : app.currentEmulator && !app.currentEmulator.found
                                    ? app.t("Cette console est disponible. Installe d’abord ", "This system is available. First install ") + app.currentEmulator.name + "."
                                    : app.t("Aucun jeu. Utilise Ajouter ou Dossier dans la barre d'outils.",
                                            "No games. Use Add or Folder in the toolbar.")
                            horizontalAlignment: Text.AlignHCenter
                            wrapMode: Text.Wrap
                            color: app.palette.mid
                        }
                        Button {
                            visible: !searchField.text && app.currentEmulator !== null && !app.currentEmulator.found
                            Layout.alignment: Qt.AlignHCenter
                            text: app.emulatorActionText()
                            font.bold: true
                            enabled: app.pending === 0
                            onClicked: app.installCurrentEmulator()
                        }
                    }
                }

                Frame {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 176
                    padding: 12
                    RowLayout {
                        anchors.fill: parent
                        spacing: 14
                        Rectangle {
                            Layout.preferredWidth: 96
                            Layout.fillHeight: true
                            color: app.palette.alternateBase
                            border.color: app.palette.mid
                            Image {
                                anchors.fill: parent
                                anchors.margins: 1
                                source: app.gameCoverSource(app.currentGame)
                                fillMode: Image.PreserveAspectFit
                                asynchronous: true
                            }
                            Label {
                                visible: !app.currentGame || app.gameCoverSource(app.currentGame) === ""
                                anchors.centerIn: parent
                                width: parent.width - 12
                                text: app.t("Aucune\njaquette", "No cover")
                                horizontalAlignment: Text.AlignHCenter
                                color: app.palette.mid
                            }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 5
                            Label { text: app.currentGame ? app.currentGame.title : app.t("Jeu sélectionné", "Selected game"); font.bold: true; font.pixelSize: 17 }
                            Label {
                                Layout.fillWidth: true
                                text: !app.currentGame ? app.t("Sélectionne un jeu.", "Select a game.")
                                      : app.metadataLoadingId === app.currentGame.id
                                        ? app.t("Recherche automatique de la fiche et de la jaquette…", "Automatically fetching details and cover…")
                                      : app.currentGame.metadata
                                        ? [app.currentGame.metadata.developer, app.currentGame.metadata.publisher, app.currentGame.metadata.release_date].filter(Boolean).join(" · ")
                                        : app.t("Informations détaillées pas encore récupérées.", "Detailed information has not been fetched yet.")
                                color: app.palette.mid
                                elide: Text.ElideRight
                            }
                            Label {
                                Layout.fillWidth: true
                                visible: app.currentGame && app.currentGame.metadata
                                         && app.currentGame.metadata.cover_url
                                         && !app.currentGame.metadata.cover_path
                                text: app.t("Jaquette indisponible hors ligne — actualise la fiche pour réessayer.",
                                            "Cover unavailable offline — refresh details to retry.")
                                color: "#a15c00"
                                wrapMode: Text.Wrap
                                font.pixelSize: 11
                            }
                            Label {
                                Layout.fillWidth: true
                                text: !app.currentGame ? ""
                                      : !app.gameInfo ? app.t("Préparation non vérifiée", "Preparation not checked")
                                      : app.gameInfo.ready ? app.t("Prêt avec ", "Ready with ") + app.gameInfo.emulator
                                      : app.gameInfo.error
                                color: app.gameInfo && !app.gameInfo.ready ? "#a12622" : app.palette.text
                                wrapMode: Text.Wrap
                            }
                            Label {
                                visible: app.currentGame && app.currentGame.identity && (app.currentGame.identity.serial || app.currentGame.identity.crc32)
                                text: app.currentGame && app.currentGame.identity
                                      ? (app.currentGame.identity.serial || ("CRC32 " + app.currentGame.identity.crc32)) : ""
                                color: app.palette.mid
                                font.pixelSize: 11
                            }
                            RowLayout {
                                Button {
                                    text: app.currentGame && app.currentGame.metadata
                                          ? app.t("Actualiser la fiche", "Refresh details")
                                          : app.metadataLoadingId
                                            ? app.t("Recherche…", "Searching…")
                                            : app.t("Réessayer la fiche", "Retry details")
                                    enabled: app.currentGame !== null && app.pending === 0
                                    onClicked: app.refreshGameMetadata(false)
                                }
                                Button { text: app.t("Vérifier", "Check"); enabled: app.currentGame !== null && app.pending === 0; onClicked: app.inspectGame() }
                                Button { text: app.t("Lancer", "Launch"); enabled: app.currentGame !== null && app.pending === 0; font.bold: true; onClicked: app.launchGame() }
                                Item { Layout.fillWidth: true }
                            }
                        }
                    }
                }
            }
        }
    }

    Dialog {
        id: aboutDialog
        anchors.centerIn: parent
        modal: true
        title: app.t("À propos", "About")
        standardButtons: Dialog.Ok
        width: 440
        contentItem: ColumnLayout {
            Label { text: "Console Launcher " + Launcher.version; font.pixelSize: 20; font.bold: true }
            Label { Layout.fillWidth: true; text: app.t("Lanceur d'émulation pour Windows et Linux. Aucun jeu ni BIOS n'est fourni.", "Emulation launcher for Windows and Linux. No games or BIOS files are provided."); wrapMode: Text.Wrap }
            Label { text: "Qt Quick + Rust"; color: app.palette.mid }
            Label {
                text: '<a href="https://titlescreen.org">' + app.t("Données : Title Screen (CC BY 4.0)", "Data: Title Screen (CC BY 4.0)") + '</a>'
                textFormat: Text.RichText
                onLinkActivated: function(link) { Qt.openUrlExternally(link) }
            }
            Label {
                text: '<a href="https://www.gametdb.com">' + app.t("Jaquettes GameCube/Wii : GameTDB", "GameCube/Wii artwork: GameTDB") + '</a>'
                textFormat: Text.RichText
                onLinkActivated: function(link) { Qt.openUrlExternally(link) }
            }
        }
    }

    Dialog {
        id: systemChooser
        visible: app.systemChooserOpen
        anchors.centerIn: parent
        modal: true
        closePolicy: Popup.NoAutoClose
        title: app.t("Choisir la console", "Choose the system")
        standardButtons: Dialog.Cancel
        onRejected: app.systemChooserOpen = false
        width: 500
        height: 420
        contentItem: ListView {
            clip: true
            spacing: 4
            model: app.compatibleSystems
            delegate: Button {
                required property var modelData
                width: ListView.view.width
                text: modelData.info.name
                onClicked: app.addGame(modelData)
            }
        }
    }

    Rectangle {
        visible: Launcher.running
        anchors.fill: parent
        z: 900
        color: app.palette.window
        ColumnLayout {
            anchors.centerIn: parent
            Label { Layout.alignment: Qt.AlignHCenter; text: app.t("Partie en cours", "Game running"); font.pixelSize: 22; font.bold: true }
            Label { Layout.alignment: Qt.AlignHCenter; text: app.t("Le launcher revient lorsque l'émulateur se ferme.", "The launcher returns when the emulator closes.") }
        }
    }

    SettingsPanel { anchors.fill: parent; z: 1000 }
}
