import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ConsoleLauncher

Item {
    id: root
    property var sourceState: ({})
    property var targetState: ({})
    property string lastControl: ""
    property real lastValue: 0
    signal closeRequested()

    readonly property bool keyboard: device.currentIndex === 0
    readonly property string system: ["gamecube", "ps2", "xbox", "ps5"][systemBox.currentIndex]
    readonly property var xboxTargetState: {
        const out = {}
        const face = {"face-bottom":"a", "face-right":"b", "face-left":"x", "face-top":"y"}
        for (const code in targetState) out[face[code] || code] = targetState[code]
        return out
    }
    readonly property var keyModel: [
        { c: "q", l: "Q", x: 18, y: 20 }, { c: "w", l: "W", x: 72, y: 20 },
        { c: "i", l: "I", x: 180, y: 20 }, { c: "t", l: "T", x: 288, y: 20 },
        { c: "a", l: "A", x: 18, y: 74 }, { c: "s", l: "S", x: 72, y: 74 }, { c: "d", l: "D", x: 126, y: 74 },
        { c: "j", l: "J", x: 180, y: 74 }, { c: "k", l: "K", x: 234, y: 74 }, { c: "l", l: "L", x: 288, y: 74 },
        { c: "x", l: "X", x: 18, y: 128 }, { c: "z", l: "Z", x: 72, y: 128 }, { c: "c", l: "C", x: 126, y: 128 },
        { c: "f", l: "F", x: 180, y: 128 }, { c: "g", l: "G", x: 234, y: 128 }, { c: "h", l: "H", x: 288, y: 128 },
        { c: "up", l: "↑", x: 126, y: 202 }, { c: "left", l: "←", x: 72, y: 256 },
        { c: "down", l: "↓", x: 126, y: 256 }, { c: "right", l: "→", x: 180, y: 256 },
        { c: "enter", l: "ENTRÉE", x: 250, y: 224, w: 92 }
    ]

    function active(code) { return Math.abs(Number(sourceState[code] || 0)) > 0.18 }
    function setTarget(out, code, value) {
        if (Math.abs(Number(value)) > Math.abs(Number(out[code] || 0))) out[code] = value
    }
    function rebuildTarget() {
        const out = {}
        for (const code in sourceState) {
            const value = Number(sourceState[code] || 0)
            if (keyboard) {
                if (code === "left") setTarget(out, "leftx", -value)
                else if (code === "right") setTarget(out, "leftx", value)
                else if (code === "up") setTarget(out, "lefty", -value)
                else if (code === "down") setTarget(out, "lefty", value)
                else if (code === "j") setTarget(out, "rightx", -value)
                else if (code === "l") setTarget(out, "rightx", value)
                else if (code === "i") setTarget(out, "righty", -value)
                else if (code === "k") setTarget(out, "righty", value)
                else if (code === "x") setTarget(out, "face-bottom", value)
                else if (code === "z") setTarget(out, "face-right", value)
                else if (code === "c") setTarget(out, "face-left", value)
                else if (code === "s") setTarget(out, "face-top", value)
                else if (code === "q") setTarget(out, "leftshoulder", value)
                else if (code === "w") setTarget(out, "rightshoulder", value)
                else if (code === "a") setTarget(out, "lefttrigger", value)
                else if (code === "d") setTarget(out, "righttrigger", value)
                else if (code === "enter") setTarget(out, "start", value)
                else if (["t", "f", "g", "h"].indexOf(code) >= 0)
                    setTarget(out, "dpad-" + ({t:"up", f:"left", g:"down", h:"right"})[code], value)
            } else {
                const face = {a:"face-bottom", b:"face-right", x:"face-left", y:"face-top"}
                setTarget(out, face[code] || code, value)
            }
        }
        targetState = out
    }
    function accept(event) {
        if ((keyboard && event.source !== "keyboard") || (!keyboard && event.source !== "controller")) return
        if (event.source === "keyboard" && event.control === "escape" && Number(event.value) > 0) {
            closeRequested(); return
        }
        const copy = {}
        for (const key in sourceState) copy[key] = sourceState[key]
        copy[event.control] = Number(event.value)
        sourceState = copy
        lastControl = event.control
        lastValue = Number(event.value)
        rebuildTarget()
    }
    function reset() { sourceState = {}; targetState = {}; lastControl = ""; lastValue = 0 }
    function close() { Launcher.controlCapture = false; reset(); closeRequested() }

    onKeyboardChanged: reset()
    onSystemChanged: rebuildTarget()
    onVisibleChanged: {
        Launcher.controlCapture = visible
        if (!visible) reset()
    }
    Component.onDestruction: Launcher.controlCapture = false

    Connections {
        target: Launcher
        function onControlInput(event) { root.accept(event) }
        function onPadChanged() { root.reset() }
    }

    Rectangle { anchors.fill: parent; color: "#f4f4f2"; border.color: "black"; border.width: 1 }

    ColumnLayout {
        anchors.fill: parent; anchors.margins: 24; spacing: 14
        RowLayout {
            Layout.fillWidth: true
            ColumnLayout {
                Layout.fillWidth: true; spacing: 2
                Label { text: root.keyboard ? "TESTEUR CLAVIER" : "TESTEUR MANETTE"; font.pixelSize: 22; font.bold: true; color: "black" }
                Label { text: "Boutons, gâchettes et direction des sticks en direct"; color: "#555" }
            }
            ComboBox {
                id: device; currentIndex: 1; model: ["Clavier", "Xbox", "PS5"]
                onCurrentIndexChanged: root.reset()
                enabled: true; Layout.preferredWidth: 220
            }
            ComboBox { id: systemBox; model: ["GameCube", "PlayStation 2", "Xbox", "PS5"]; currentIndex: 1; Layout.preferredWidth: 160 }
            CheckBox { id: animations; text: "Animations"; checked: true }
            Button { text: "Fermer"; onClicked: root.close() }
        }

        RowLayout {
            Layout.fillWidth: true; Layout.fillHeight: true; spacing: 18
            Rectangle {
                Layout.fillWidth: true; Layout.fillHeight: true; color: "white"; border.color: "black"; border.width: 2
                Label { x: 18; y: 14; text: root.keyboard ? "CLAVIER" : (device.currentText.toUpperCase() + (Launcher.padName ? " — " + Launcher.padName : " — NON CONNECTÉE")); font.bold: true; color: "black" }
                Item {
                    anchors.centerIn: parent; width: 380; height: 330; visible: root.keyboard
                    Repeater {
                        model: root.keyModel
                        Rectangle {
                            required property var modelData
                            x: modelData.x; y: modelData.y; width: modelData.w || 46; height: 46; radius: 5
                            color: root.active(modelData.c) ? "#1769aa" : "white"
                            border.color: "black"; border.width: 2
                            Text { anchors.centerIn: parent; text: modelData.l; color: root.active(modelData.c) ? "white" : "black"; font.bold: true; font.pixelSize: modelData.l.length > 2 ? 11 : 17 }
                        }
                    }
                }

                XboxControlPad { anchors.centerIn: parent; width: Math.min(parent.width - 30, 704); height: Math.max(0, parent.height - 85); visible: device.currentIndex === 1; active: root.sourceState; animationsEnabled: animations.checked }
                PS5ControlPad {
                    anchors.centerIn: parent
                    width: Math.max(0, parent.width - 30); height: Math.max(0, parent.height - 95)
                    visible: device.currentIndex === 2
                    active: root.targetState; animationsEnabled: animations.checked
                }
            }
            Rectangle {
                Layout.fillWidth: true; Layout.fillHeight: true
                color: "white"; border.color: "black"; border.width: 2
                Label { x: 18; y: 14; text: systemBox.currentText.toUpperCase(); font.bold: true; color: "black" }
                GameCubeControlPad {
                    visible: root.system === "gamecube"
                    anchors.centerIn: parent
                    width: Math.max(0, parent.width - 30); height: Math.max(0, parent.height - 95)
                    active: root.targetState; animationsEnabled: animations.checked
                }
                PS2ControlPad {
                    anchors.centerIn: parent
                    width: Math.max(0, parent.width - 30); height: Math.max(0, parent.height - 95)
                    visible: root.system === "ps2"
                    active: root.targetState; animationsEnabled: animations.checked
                }
                XboxControlPad {
                    anchors.centerIn: parent
                    width: Math.max(0, parent.width - 30); height: Math.max(0, parent.height - 95)
                    visible: root.system === "xbox"
                    active: root.xboxTargetState; animationsEnabled: animations.checked
                }
                PS5ControlPad {
                    anchors.centerIn: parent
                    width: Math.max(0, parent.width - 30); height: Math.max(0, parent.height - 95)
                    visible: root.system === "ps5"
                    active: root.targetState; animationsEnabled: animations.checked
                }
                Label {
                    anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter
                    anchors.bottomMargin: 12; width: parent.width - 30
                    text: "Correspondance visuelle — affectations non appliquées"
                    horizontalAlignment: Text.AlignHCenter; wrapMode: Text.WordWrap; color: "#555"; font.pixelSize: 11
                }
            }
        }

        GridLayout {
            Layout.fillWidth: true; visible: !root.keyboard
            columns: width > 740 ? 4 : 2; columnSpacing: 24; rowSpacing: 6
            Repeater {
                model: [
                    {label: "STICK GAUCHE", x: "leftx", y: "lefty"},
                    {label: "STICK DROIT", x: "rightx", y: "righty"},
                    {label: "GÂCHETTE LT", trigger: "lefttrigger"},
                    {label: "GÂCHETTE RT", trigger: "righttrigger"}
                ]
                ColumnLayout {
                    required property var modelData
                    Layout.fillWidth: true; spacing: 2
                    Label { text: modelData.label; color: "#555"; font.pixelSize: 11 }
                    Label {
                        text: modelData.trigger
                            ? Math.round(Math.max(0, Number(root.sourceState[modelData.trigger] || 0)) * 100) + " %"
                            : "X " + Number(root.sourceState[modelData.x] || 0).toFixed(2) + "   Y " + Number(root.sourceState[modelData.y] || 0).toFixed(2)
                        color: "#111"; font.pixelSize: 15; font.family: "Consolas"
                    }
                }
            }
        }
        RowLayout {
            Layout.fillWidth: true
            Label { Layout.fillWidth: true; text: lastControl ? "Reçu : " + lastControl + "  " + lastValue.toFixed(2) : "Appuie sur une touche, un bouton ou bouge un joystick."; color: "black" }
            Label { text: root.keyboard ? "Échap pour fermer" : "Utilise le bouton Fermer"; color: "#555" }
        }
    }
}
