import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window
import ConsoleLauncher

Window {
    id: overlay
    readonly property var monitor: Launcher.performance
    readonly property var stats: monitor.metrics
    width: 390
    height: content.implicitHeight + 30
    x: monitor.panelX; y: monitor.panelY
    visible: monitor.display
    color: "transparent"
    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
           | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    title: "Console Launcher — Diagnostic en jeu"

    Rectangle {
        anchors.fill: parent; radius: 10
        color: "#ed141b25"; border.color: "#647287"; border.width: 1
    }
    function number(key, decimals, suffix) {
        const v = stats[key]
        return v !== undefined && isFinite(Number(v)) ? Number(v).toFixed(decimals) + suffix : "Non mesuré"
    }
    ColumnLayout {
        id: content
        anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
        anchors.margins: 15; spacing: 8
        RowLayout {
            Layout.fillWidth: true
            Text { text: "DIAGNOSTIC EN JEU"; color: "#f2f5fa"; font.pixelSize: 14; font.bold: true; Layout.fillWidth: true }
            Text { text: "F" + monitor.functionKey; color: "#91baff"; font.pixelSize: 14; font.bold: true }
        }
        Text { text: stats.emulator || "Émulateur"; color: "#b4c0d2"; font.pixelSize: 12; elide: Text.ElideRight; Layout.fillWidth: true }
        Rectangle { Layout.fillWidth: true; height: 1; color: "#3c485b" }
        GridLayout {
            columns: 2; columnSpacing: 18; rowSpacing: 6; Layout.fillWidth: true
            Repeater {
                model: [
                    {label: "FPS présentés", value: stats.measured ? overlay.number("fps", 1, " FPS") : "Non mesuré"},
                    {label: "Temps d'image moyen", value: stats.measured ? overlay.number("frameMs", 2, " ms") : "Non mesuré"},
                    {label: "Images lentes · p99", value: stats.measured ? overlay.number("p99Ms", 2, " ms") : "Non mesuré"},
                    {label: "1 % bas · selon p99", value: stats.measured ? overlay.number("low1", 1, " FPS") : "Non mesuré"},
                    {label: "Pics > 1,5 × médiane", value: stats.measured ? String(stats.spikes) + " / 2 s" : "Non mesuré"},
                    {label: "CPU émulateur · PC total", value: overlay.number("cpu", 1, " %")},
                    {label: "RAM émulateur", value: overlay.number("ramMb", 0, " Mo")},
                    {label: "Vitesse d'émulation", value: "Non mesurée"},
                    {label: "Son · pertes / tampon", value: "Non mesuré"},
                    {label: "GPU / températures", value: "Non mesurés"}
                ]
                delegate: RowLayout {
                    // Each delegate supplies a full row through a two-column span.
                    required property var modelData
                    Layout.columnSpan: 2; Layout.fillWidth: true
                    Text { text: modelData.label; color: "#c4cede"; font.pixelSize: 12; Layout.fillWidth: true }
                    Text { text: modelData.value; color: "#f2f5fa"; font.family: "Consolas"; font.pixelSize: 12 }
                }
            }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: "#3c485b" }
        Text {
            text: monitor.status
            color: stats.measured ? "#91baff" : "#ffcf8a"
            font.pixelSize: 11; wrapMode: Text.WordWrap; Layout.fillWidth: true
        }
        Text {
            text: "FPS ≠ vitesse d'émulation. Le son n'est pas validé."
            color: "#b4c0d2"; font.pixelSize: 11; Layout.fillWidth: true; wrapMode: Text.WordWrap
        }
    }
}
