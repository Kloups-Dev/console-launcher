import QtQuick
import QtQuick.Layouts
import QtQuick.Window
import ConsoleLauncher

Window {
    id: overlay
    readonly property var monitor: Launcher.performance
    readonly property var stats: monitor.metrics
    readonly property var context: stats.context || ({})
    readonly property var hardware: context.hardware || ({})
    readonly property var settings: context.settings || ({})
    readonly property bool detailed: monitor.viewMode === 2
    readonly property int textSize: monitor.panelHeight < 800 ? 11 : 13
    width: monitor.panelWidth
    height: detailed ? monitor.panelHeight : compact.implicitHeight + 28
    x: monitor.panelX; y: monitor.panelY
    visible: monitor.display
    color: "transparent"
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
           | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    title: "Console Launcher — Statistiques en direct"
    onVisibleChanged: if (visible) raise()

    function number(key, decimals, suffix) {
        return format(stats[key], decimals, suffix)
    }
    function format(value, decimals, suffix) {
        return value !== undefined && value !== null && isFinite(Number(value))
               ? Number(value).toFixed(decimals) + suffix : "Non disponible"
    }
    function setting(key) {
        const kept = context.kept || []
        if (kept.indexOf(key) >= 0) return "Choix manuel conservé"
        return settings[key] !== undefined ? String(settings[key]) : "Géré par l’émulateur"
    }
    function problems() {
        let lines = []
        if (!stats.measured) lines.push("FPS non mesurés : vérifier le collecteur et son autorisation Windows.")
        if (stats.measured && stats.p99Ms > stats.medianMs * 2) lines.push("Pics de durée d’image : saccades possibles sur les 2 dernières secondes.")
        if (stats.gpuProcessBusy >= 95) lines.push("Moteur GPU de l’émulateur très chargé : la résolution est une piste à examiner.")
        if (stats.systemRamPercent >= 90) lines.push("Mémoire du PC presque pleine.")
        const cards = stats.gpuCards || []
        for (let c of cards) if (c.temperature >= 85) lines.push("GPU chaud : " + c.temperature + " °C. Ce seuil seul ne prouve pas un bridage.")
        if (!stats.shortcutAvailable) lines.push("F" + monitor.functionKey + " indisponible ; secours Ctrl+Alt+F8 " + (stats.fallbackAvailable ? "actif." : "indisponible."))
        return lines.length ? lines.join("  •  ") : "Aucune alerte dans les mesures disponibles. Cela ne valide ni le son ni la vitesse d’émulation."
    }
    function gpuText() {
        const cards = stats.gpuCards || []
        if (!cards.length) return "Capteurs du pilote non disponibles"
        return cards.map(c => (c.name || "GPU") + " · " + format(c.busy,0," %") + " · "
                         + format(c.temperature,0," °C") + " · " + format(c.clockMhz,0," MHz")
                         + " · " + format(c.powerW,1," W") + " · VRAM "
                         + format(c.usedMb,0,"") + "/" + format(c.totalMb,0," Mo")).join("\n")
    }
    component Card: Rectangle {
        property string heading
        property var rows: []
        radius: 8; color: "#d9182433"; border.color: "#40556c"
        implicitHeight: cardContent.implicitHeight + 24
        ColumnLayout {
            id: cardContent
            anchors.fill: parent; anchors.margins: 12; spacing: 5
            Text { text: heading; color: "#7ed8e8"; font.pixelSize: overlay.textSize + 1; font.bold: true; Layout.fillWidth: true }
            Repeater {
                model: rows
                delegate: RowLayout {
                    required property var modelData
                    Layout.fillWidth: true; spacing: 8
                    Text { text: modelData[0]; color: "#b1c2d6"; font.pixelSize: overlay.textSize; Layout.fillWidth: true; elide: Text.ElideRight }
                    Text { text: modelData[1]; color: "#ffffff"; font.family: "Consolas"; font.pixelSize: overlay.textSize; Layout.maximumWidth: cardContent.width * 0.61; elide: Text.ElideRight }
                }
            }
            Item { Layout.fillHeight: true }
        }
    }
    Rectangle { anchors.fill: parent; radius: detailed ? 10 : 7; color: detailed ? "#f00a111b" : "#e5101925"; border.color: "#56718f" }
    ColumnLayout {
        id: compact
        visible: !overlay.detailed
        anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
        anchors.margins: 14; spacing: 5
        RowLayout {
            Layout.fillWidth: true
            Text { text: stats.measured ? overlay.number("fps",1," FPS") : "FPS —"; color: "#ffffff"; font.pixelSize: 25; font.bold: true; Layout.fillWidth: true }
            Text { text: stats.measured ? overlay.number("frameMs",1," ms") : "En attente"; color: "#7ed8e8"; font.pixelSize: 15 }
        }
        Text { text: "GPU jeu " + overlay.number("gpuProcessBusy",0," %") + "  ·  RAM jeu " + overlay.number("ramMb",0," Mo"); color: "#ced9e8"; font.pixelSize: 11 }
        Text { text: stats.measured ? "p99 " + overlay.number("p99Ms",1," ms") + "  ·  pics " + stats.spikes + "/2 s" : monitor.status; color: stats.measured ? "#bac9dd" : "#ffd399"; font.pixelSize: 11; wrapMode: Text.WordWrap; Layout.fillWidth: true }
        Text { text: "F" + monitor.functionKey + " : diagnostic / masquer · Ctrl+Alt+F8 : secours"; color: "#829bb9"; font.pixelSize: 10; Layout.fillWidth: true; wrapMode: Text.WordWrap }
    }
    ColumnLayout {
        visible: overlay.detailed
        anchors.fill: parent; anchors.margins: 18; spacing: 10
        RowLayout {
            Layout.fillWidth: true
            ColumnLayout {
                Layout.fillWidth: true; spacing: 2
                Text { text: "CONSOLE LAUNCHER  /  DIAGNOSTIC EN DIRECT"; color: "#7ed8e8"; font.pixelSize: overlay.textSize + 3; font.bold: true }
                Text { text: (context.game || "Jeu") + "  ·  " + (context.system || "") + "  ·  " + (stats.emulator || "Émulateur") + "  ·  PID " + (stats.pid || "?"); color: "white"; font.pixelSize: overlay.textSize + 1; Layout.fillWidth: true; elide: Text.ElideRight }
            }
            ColumnLayout {
                Text { text: stats.timestamp || ""; color: "#bac9dd"; font.pixelSize: overlay.textSize }
                Text { text: "F" + monitor.functionKey + " : masquer  ·  Ctrl+Alt+F8 : secours"; color: "#bac9dd"; font.pixelSize: overlay.textSize }
            }
        }
        Text {
            Layout.fillWidth: true; wrapMode: Text.WordWrap
            text: overlay.problems(); color: stats.measured ? "#d8e4a3" : "#ffd399"; font.pixelSize: overlay.textSize
        }
        RowLayout {
            Layout.fillWidth: true; Layout.fillHeight: true; spacing: 10
            Card {
                Layout.fillWidth: true; Layout.fillHeight: true; Layout.preferredWidth: 1
                heading: "01  IMAGES · PRÉSENTATIONS WINDOWS"
                rows: [
                    ["FPS présentés", stats.measured ? overlay.number("fps",1," FPS") : "Non mesurés"],
                    ["Durée moyenne", overlay.number("frameMs",2," ms")],
                    ["Médiane", overlay.number("medianMs",2," ms")],
                    ["p95 / p99", overlay.number("p95Ms",1,"") + " / " + overlay.number("p99Ms",1," ms")],
                    ["1 % bas · 1000/p99", overlay.number("low1",1," FPS")],
                    ["Durée min / max", overlay.number("minMs",1,"") + " / " + overlay.number("maxMs",1," ms")],
                    ["Pics > 1,5× médiane", stats.measured ? stats.spikes + " / 2 s" : "Non mesurés"],
                    ["Images non affichées", overlay.number("droppedPercent",1," %")],
                    ["Travail GPU par image", overlay.number("gpuActiveMs",2," ms")],
                    ["Present → fin GPU", overlay.number("renderDelayMs",2," ms")],
                    ["Present → affichage", overlay.number("displayDelayMs",2," ms")],
                    ["Échantillon / fraîcheur", (stats.frames || 0) + " images · " + overlay.number("ageMs",0," ms")],
                    ["Écran Windows", stats.screen || "Non disponible"]
                ]
            }
            Card {
                Layout.fillWidth: true; Layout.fillHeight: true; Layout.preferredWidth: 1
                heading: "02  PC · CHARGE EN TEMPS RÉEL"
                rows: [
                    ["CPU émulateur / PC", overlay.number("cpuPcPercent",1," %")],
                    ["CPU jeu · 100 % = 1 cœur", overlay.number("cpu",1," %")],
                    ["CPU total du PC", overlay.number("systemCpu",1," %")],
                    ["RAM émulateur", overlay.number("ramMb",0," Mo")],
                    ["RAM utilisée / PC", overlay.number("systemRamUsedMb",0,"") + " / " + overlay.number("systemRamTotalMb",0," Mo")],
                    ["RAM occupée", overlay.number("systemRamPercent",0," %")],
                    ["GPU jeu · moteur le + actif", overlay.number("gpuProcessBusy",0," %")],
                    ["VRAM dédiée du jeu", overlay.number("gpuProcessMemoryMb",0," Mo")],
                    ["Température CPU", "Capteur non disponible"],
                    ["Latence manette → écran", "Non mesurée"],
                    ["Audio · délai / craquements", "Non mesurés"],
                    ["Vitesse interne émulation", "Non mesurée"]
                ]
            }
            Card {
                Layout.fillWidth: true; Layout.fillHeight: true; Layout.preferredWidth: 1
                heading: "03  CONTEXTE · RÉGLAGES DEMANDÉS"
                rows: [
                    ["Profil / priorité", (context.profile || "?") + " / " + (context.priority || "?")],
                    ["API choisie", context.api || "Non disponible"],
                    ["VSync Dolphin", overlay.setting("GFX.Hardware.VSync")],
                    ["Mode shaders Dolphin", overlay.setting("GFX.Settings.ShaderCompilationMode")],
                    ["Cache shaders", overlay.setting("GFX.Settings.ShaderCache")],
                    ["Précompilation", overlay.setting("GFX.Settings.WaitForShadersBeforeStarting")],
                    ["Tampon audio demandé", overlay.setting(stats.emulator && stats.emulator.toLowerCase().indexOf("dolphin") >= 0 ? "Dolphin.Core.AudioBufferSize" : "audio_latency") + " ms"],
                    ["Réglages manuels gardés", String((context.kept || []).length)],
                    ["CPU détecté", hardware.cpu ? hardware.cpu.brand : "Non disponible"],
                    ["Score CPU · estimation", hardware.bench ? String(hardware.bench.score) : "Non disponible"],
                    ["Cœurs physiques / logiques", hardware.cpu ? hardware.cpu.physical_cores + " / " + hardware.cpu.logical_cores : "Non disponible"],
                    ["Système", hardware.os || "Non disponible"],
                    ["Version lanceur", stats.launcherVersion || "?"]
                ]
            }
        }
        RowLayout {
            Layout.fillWidth: true; spacing: 10
            Rectangle {
                Layout.fillWidth: true; Layout.preferredWidth: 2; height: overlay.monitor.panelHeight < 800 ? 95 : 130
                color: "#d9182433"; radius: 8
                Text { x: 12; y: 7; text: "DURÉE DES IMAGES · 2 DERNIÈRES SECONDES · ms (plus régulier = mieux)"; color: "#7ed8e8"; font.pixelSize: overlay.textSize }
                Canvas {
                    id: plot
                    anchors.fill: parent; anchors.margins: 12; anchors.topMargin: 28
                    property var values: stats.timeline || []
                    onValuesChanged: requestPaint()
                    onWidthChanged: requestPaint()
                    onHeightChanged: requestPaint()
                    onPaint: {
                        const c = getContext("2d"); c.clearRect(0,0,width,height)
                        if (values.length < 2) return
                        let max = Math.max(40, Number(stats.maxMs || 40))
                        c.strokeStyle = "#40556c"; c.lineWidth = 1
                        for (let ms of [16.67,33.33]) { let y=height*(1-ms/max); c.beginPath(); c.moveTo(0,y); c.lineTo(width,y); c.stroke() }
                        c.strokeStyle = "#7ed8e8"; c.lineWidth = 1.5; c.beginPath()
                        for (let i=0;i<values.length;i++) { let x=i*width/(values.length-1); let y=height*(1-Number(values[i])/max); if(i===0)c.moveTo(x,y);else c.lineTo(x,y) }
                        c.stroke(); c.fillStyle="#bac9dd"; c.font="10px sans-serif"; c.fillText(max.toFixed(0)+" ms",3,10)
                    }
                }
            }
            Rectangle {
                Layout.fillWidth: true; Layout.preferredWidth: 1; height: plot.parent.height
                color: "#d9182433"; radius: 8
                Text { anchors.fill: parent; anchors.margins: 12; text: "CAPTEURS GPU · CARTE ENTIÈRE\n" + overlay.gpuText(); color: "#d9e6f6"; font.pixelSize: overlay.textSize; wrapMode: Text.WordWrap }
            }
        }
        Text {
            Layout.fillWidth: true; color: stats.measured ? "#9db1cb" : "#ffd399"; font.pixelSize: overlay.textSize; wrapMode: Text.WordWrap
            text: monitor.status + (stats.collectorError ? " · " + stats.collectorError : "")
        }
        Text {
            Layout.fillWidth: true; color: "#9db1cb"; font.pixelSize: overlay.textSize; wrapMode: Text.WordWrap
            text: "Sources : PresentMon (images), Windows (processus / GPU / RAM), pilote NVIDIA si disponible (capteurs). Fenêtre glissante 2 s, actualisation 0,5 s. FPS présentés ≠ FPS natifs ni vitesse d’émulation. Réglages demandés ≠ mesures internes du moteur."
        }
        Text {
            Layout.fillWidth: true; color: "#ffd399"; font.pixelSize: overlay.textSize; wrapMode: Text.WordWrap
            text: "Notes de session : " + ((context.notes || []).join(" · ") || "Aucune")
            maximumLineCount: 3; elide: Text.ElideRight
        }
    }
}
