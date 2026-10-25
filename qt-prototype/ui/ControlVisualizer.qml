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
    property string selectedTarget: "face-bottom"
    property bool assignmentMode: false
    property var assignments: ({})
    property string notice: ""
    property bool saving: false
    readonly property bool canApply: system === "gamecube" && device.currentIndex === 2 && Qt.platform.os === "windows" && player.currentIndex === 0
    readonly property var targetLabels: ["A", "B", "X", "Y", "Gâchette L", "Gâchette R", "Start / Pause", "Croix : haut", "Croix : bas", "Croix : gauche", "Croix : droite"]
    signal closeRequested()

    readonly property bool keyboard: device.currentIndex === 0
    readonly property bool mouse: device.currentIndex === 1
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
    function calibrated(code, value) {
        let out = Number(value)
        const deadzone = Number(targetState.__deadzone || 0.12)
        const sensitivity = Number(targetState.__sensitivity || 1.0)
        if (code === "leftx" || code === "lefty" || code === "rightx" || code === "righty"
            || code === "mousex" || code === "mousey") {
            if (Math.abs(out) <= deadzone) return 0
            out = Math.sign(out) * Math.min(1, (Math.abs(out) - deadzone) / (1 - deadzone) * sensitivity)
            if ((code === "leftx" || code === "rightx" || code === "mousex") && targetState.__invertx)
                out = -out
            if ((code === "lefty" || code === "righty" || code === "mousey") && targetState.__inverty)
                out = -out
        }
        return out
    }
    function setTarget(out, code, value) {
        if (Math.abs(Number(value)) > Math.abs(Number(out[code] || 0))) out[code] = value
    }
    function rebuildTarget() {
        const out = {}
        for (const code in sourceState) {
            const value = calibrated(code, sourceState[code] || 0)
            let assigned = false
            for (const target in assignments) {
                if (assignments[target] === code) { setTarget(out, target, value); assigned = true }
            }
            if (assigned) continue
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
            } else if (mouse) {
                if (code === "mousex") setTarget(out, "rightx", value)
                else if (code === "mousey") setTarget(out, "righty", value)
                else if (code === "mouse-left") setTarget(out, "face-bottom", value)
                else if (code === "mouse-right") setTarget(out, "face-right", value)
                else if (code === "mouse-middle") setTarget(out, "start", value)
            } else {
                const face = {a:"face-bottom", b:"face-right", x:"face-left", y:"face-top"}
                setTarget(out, face[code] || code, value)
            }
        }
        targetState = out
    }
    function accept(event) {
        if (!visible) return
        // Escape must work regardless of the selected input device.
        if (event.source === "keyboard" && event.control === "escape" && Number(event.value) > 0) {
            Qt.callLater(root.close); return
        }
        if ((keyboard && event.source !== "keyboard")
            || (mouse && event.source !== "mouse")
            || (!keyboard && !mouse && event.source !== "controller")) return
        if (assignmentMode && Number(event.value) > 0) {
            const next = Object.assign({}, assignments)
            next[selectedTarget] = event.control
            assignments = next
            assignmentMode = false
        }
        const copy = {}
        for (const key in sourceState) copy[key] = sourceState[key]
        copy[event.control] = Number(event.value)
        sourceState = copy
        lastControl = event.control
        lastValue = Number(event.value)
        rebuildTarget()
    }
    function reset() {
        sourceState = {}; targetState = {}; assignments = {}
        lastControl = ""; lastValue = 0; assignmentMode = false
    }
    function mappingKey() {
        return (keyboard ? "keyboard" : mouse ? "mouse" : device.currentText.toLower())
            + ":p" + (player.currentIndex + 1) + ":" + system
    }
    function saveMapping() {
        notice = "Réglages du test enregistrés. Pour jouer, configure les commandes dans l’émulateur."

        Launcher.internalRequest("save_control_mapping",
                                 {key: mappingKey(), mapping: targetState},
                                 function(ok, result) {
                                     if (!ok) console.warn("mapping save failed:", result)
                                 })
    }
    function applyXbox() {
        if (!canApply || saving) return
        const keys = {
            "Device": "XInput/" + xboxSlot.currentIndex + "/Gamepad",
            "Buttons/A": "`Button A`", "Buttons/B": "`Button B`",
            "Buttons/X": "`Button X`", "Buttons/Y": "`Button Y`",
            "Buttons/Z": "`Shoulder R`", "Buttons/Start": "Start",
            "Main Stick/Up": "`Left Y+`", "Main Stick/Down": "`Left Y-`",
            "Main Stick/Left": "`Left X-`", "Main Stick/Right": "`Left X+`",
            "C-Stick/Up": "`Right Y+`", "C-Stick/Down": "`Right Y-`",
            "C-Stick/Left": "`Right X-`", "C-Stick/Right": "`Right X+`",
            "Triggers/L": "`Trigger L` > 0.9", "Triggers/R": "`Trigger R` > 0.9",
            "Triggers/L-Analog": "`Trigger L`", "Triggers/R-Analog": "`Trigger R`",
            "D-Pad/Up": "`Pad N`", "D-Pad/Down": "`Pad S`",
            "D-Pad/Left": "`Pad W`", "D-Pad/Right": "`Pad E`"
        }
        const mapping = {}
        for (const key in keys) mapping["GCPad1/" + key] = keys[key]
        saving = true
        Launcher.internalRequest("save_emulator_mapping", {key:"dolphin:gamecube", mapping:mapping}, function(ok, result) {
            root.saving = false
            root.notice = ok
                ? "Profil Xbox enregistré pour Dolphin / GameCube, joueur 1. Il sera écrit dans Dolphin au prochain lancement du jeu. Ferme puis relance le jeu depuis le lanceur."
                : "Échec de l’enregistrement : " + result
        })
    }
    function loadMapping() {
        Launcher.internalRequest("get_control_mapping", {key: mappingKey()},
                                 function(ok, result) {
                                     if (ok && result) targetState = result
                                 })
        if (system === "gamecube" && keyboard) {
            Launcher.internalRequest("get_emulator_mapping", {key: "dolphin:gamecube"},
                                     function(ok, result) {
                                         if (!ok || !result) return
                                         const reverse = {
                                             "GCPad1/Buttons/A": "face-bottom",
                                             "GCPad1/Buttons/B": "face-right",
                                             "GCPad1/Buttons/X": "face-left",
                                             "GCPad1/Buttons/Y": "face-top",
                                             "GCPad1/Triggers/L": "leftshoulder",
                                             "GCPad1/Triggers/R": "rightshoulder",
                                             "GCPad1/Buttons/Start": "start",
                                             "GCPad1/DPad/Up": "dpad-up",
                                             "GCPad1/DPad/Down": "dpad-down",
                                             "GCPad1/DPad/Left": "dpad-left",
                                             "GCPad1/DPad/Right": "dpad-right"
                                         }
                                         const next = {}
                                         for (const path in result) {
                                             const target = reverse[path]
                                             const value = String(result[path])
                                             if (target && value.indexOf("Keyboard/") === 0)
                                                 next[target] = value.substring("Keyboard/".length).toLowerCase()
                                         }
                                         assignments = next
                                     })
        }
    }
    function restoreRecommended() {
        reset()
        const defaults = {"__deadzone": 0.12, "__sensitivity": 1.0, "__invertx": 0, "__inverty": 0}
        targetState = defaults
        saveMapping()
    }
    function close() { Launcher.controlCapture = false; reset(); closeRequested() }

    onKeyboardChanged: reset()
    onSystemChanged: { notice = ""; rebuildTarget(); loadMapping() }
    onVisibleChanged: {
        Launcher.controlCapture = visible
        if (visible) loadMapping()
        else reset()
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
                Label { text: "COMMANDES ET MANETTES"; font.pixelSize: 22; font.bold: true; color: "black" }
                Label { text: "1. Choisis ton périphérique   →   2. Choisis la console   →   3. Configure le jeu"; color: "#555" }
            }
            Button { text: "← Retour"; onClicked: root.close() }
        }
        Flow {
            Layout.fillWidth: true
            spacing: 8
            Label { text: "Périphérique"; }
            ComboBox {
                id: device; currentIndex: /xbox|xinput/i.test(Launcher.padName) ? 2 : 0; model: ["Clavier", "Souris", "Manette Xbox", "Manette PS5"]
                onCurrentIndexChanged: { root.notice = ""; root.reset() }
                enabled: true; Layout.preferredWidth: 220
            }
            Label { text: "Console cible"; }
            ComboBox { id: systemBox; model: ["GameCube", "PlayStation 2", "Xbox", "PS5"]; currentIndex: 0; Layout.preferredWidth: 160 }
            CheckBox { id: advanced; text: "Réglages du test (avancés)"; checked: false }
            CheckBox { id: animations; text: "Animations"; checked: true }
            Button { text: "Enregistrer le test"; onClicked: root.saveMapping() }
            Button { text: "Réinitialiser le test"; onClicked: root.restoreRecommended() }
        }

        Rectangle {
            Layout.fillWidth: true
            implicitHeight: statusText.implicitHeight + 24
            color: "#e8eef5"; radius: 6
            Label {
                id: statusText; anchors.fill: parent; anchors.margins: 12
                wrapMode: Text.WordWrap; color: "#17324d"
                text: root.notice || (root.canApply
                    ? "Test en direct ci-dessous. Pour jouer avec ta Xbox : clique sur « Utiliser dans Dolphin », puis relance ton jeu GameCube depuis le lanceur. A/B/X/Y, sticks et gâchettes utilisent le profil conseillé."
                    : "Cet écran teste les entrées. Les autres consoles et périphériques se configurent dans leur émulateur ; les dessins seuls ne changent pas les commandes en jeu.")
            }
        }
        Flow {
            Layout.fillWidth: true; spacing: 10
            Button { text: "Configurer dans l’émulateur…"; visible: !root.canApply; onClicked: { root.close(); Launcher.openSettings({ page: "emulators" }) } }
            Label { text: "Xbox Windows"; visible: root.canApply }
            ComboBox { id: xboxSlot; visible: root.canApply; model: ["Manette XInput 1", "Manette XInput 2", "Manette XInput 3", "Manette XInput 4"] }
            Button { text: root.saving ? "Enregistrement…" : "Utiliser dans Dolphin"; visible: root.canApply; enabled: !root.saving; onClicked: root.applyXbox() }
            Label { text: "Si aucune réaction dans Dolphin, essaie le numéro XInput suivant."; visible: root.canApply; color: "#555" }
        }

        RowLayout {
            Layout.fillWidth: true; Layout.fillHeight: true; spacing: 18
            Rectangle {
                Layout.fillWidth: true; Layout.fillHeight: true; color: "white"; border.color: "black"; border.width: 2
                Label { x: 18; y: 14; text: root.keyboard || root.mouse ? device.currentText.toUpperCase() : (device.currentText.toUpperCase() + (Launcher.padName ? " — " + Launcher.padName : " — NON CONNECTÉE")); font.bold: true; color: "black" }
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
                Item {
                    anchors.centerIn: parent; width: 300; height: 260; visible: root.mouse
                    Rectangle { x: 105; y: 20; width: 90; height: 180; radius: 42; color: root.active("mouse-left") ? "#1769aa" : "white"; border.color: "black"; border.width: 2 }
                    Rectangle { x: 150; y: 20; width: 1; height: 55; color: "black" }
                    Rectangle { x: 149; y: 70; width: 3; height: 38; color: root.active("mouse-middle") ? "#1769aa" : "black" }
                    Text { x: 115; y: 215; text: "GAUCHE   MOLETTE   DROITE"; font.pixelSize: 10; color: "black" }
                    Text { x: 115; y: 235; text: "Souris X/Y → stick droit"; font.pixelSize: 11; color: "#555" }
                }

                XboxControlPad { anchors.centerIn: parent; width: Math.min(parent.width - 30, 704); height: Math.max(0, parent.height - 85); visible: device.currentIndex === 2; active: root.sourceState; animationsEnabled: animations.checked }
                PS5ControlPad {
                    anchors.centerIn: parent
                    width: Math.max(0, parent.width - 30); height: Math.max(0, parent.height - 95)
                    visible: device.currentIndex === 3
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
                    text: "Aperçu des entrées reçues — la configuration en jeu est indiquée en haut"
                    horizontalAlignment: Text.AlignHCenter; wrapMode: Text.WordWrap; color: "#555"; font.pixelSize: 11
                }
            }
        }

        GridLayout {
            visible: advanced.checked
            Layout.fillWidth: true; columns: width > 740 ? 4 : 2; columnSpacing: 24; rowSpacing: 6
            Repeater {
                model: [
                    {label: "STICK GAUCHE", x: "leftx", y: "lefty"},
                    {label: "STICK DROIT", x: "rightx", y: "righty"},
                    {label: "GÂCHETTE LT", trigger: "lefttrigger"},
                    {label: "GÂCHETTE RT", trigger: "righttrigger"},
                    {label: "ZONE MORTE", setting: "deadzone"},
                    {label: "SENSIBILITÉ", setting: "sensitivity"}
                ]
                ColumnLayout {
                    required property var modelData
                    Layout.fillWidth: true; spacing: 2
                    Label { text: modelData.label; color: "#555"; font.pixelSize: 11 }
                    Label {
                        visible: (!root.keyboard && !root.mouse) || !!modelData.setting
                        text: modelData.setting === "deadzone"
                            ? Math.round(Number(root.targetState.__deadzone || 0.12) * 100) + " %"
                            : modelData.setting === "sensitivity"
                            ? Number(root.targetState.__sensitivity || 1.0).toFixed(2)
                            : modelData.trigger
                            ? Math.round(Math.max(0, Number(root.sourceState[modelData.trigger] || 0)) * 100) + " %"
                            : "X " + Number(root.sourceState[modelData.x] || 0).toFixed(2) + "   Y " + Number(root.sourceState[modelData.y] || 0).toFixed(2)
                        color: "#111"; font.pixelSize: 15; font.family: "Consolas"
                    }
                }
            }
        }
        RowLayout {
            visible: advanced.checked
            Layout.fillWidth: true
            Label { text: "Profil"; color: "#555" }
            ComboBox {
                id: player
                model: ["Joueur 1", "Joueur 2", "Joueur 3", "Joueur 4"]
                Layout.preferredWidth: 130
                onCurrentIndexChanged: { root.reset(); root.loadMapping() }
            }
            Label { text: "Zone morte"; color: "#555" }
            Slider {
                id: deadzone
                from: 0; to: 0.5
                value: Number(root.targetState.__deadzone || 0.12)
                Layout.fillWidth: true
                onMoved: {
                    const copy = Object.assign({}, root.targetState)
                    copy.__deadzone = value
                    root.targetState = copy
                }
            }
            Label { text: Math.round(deadzone.value * 100) + " %"; color: "black" }
            Label { text: "Sensibilité"; color: "#555" }
            Slider {
                id: sensitivity
                from: 0.25; to: 2
                value: Number(root.targetState.__sensitivity || 1)
                Layout.fillWidth: true
                onValueChanged: {
                    const copy = Object.assign({}, root.targetState)
                    copy.__sensitivity = value
                    root.targetState = copy
                }
            }
            CheckBox {
                text: "Inv. X"; checked: !!root.targetState.__invertx
                onToggled: { const copy = Object.assign({}, root.targetState); copy.__invertx = checked ? 1 : 0; root.targetState = copy }
            }
            CheckBox {
                text: "Inv. Y"; checked: !!root.targetState.__inverty
                onToggled: { const copy = Object.assign({}, root.targetState); copy.__inverty = checked ? 1 : 0; root.targetState = copy }
            }
        }
        RowLayout {
            visible: advanced.checked
            Layout.fillWidth: true
            Label { text: "Test : commande cible"; color: "#555" }
            ComboBox {
                id: targetPicker
                model: [
                    "face-bottom", "face-right", "face-left", "face-top",
                    "leftshoulder", "rightshoulder", "start",
                    "dpad-up", "dpad-down", "dpad-left", "dpad-right"
                ]
                currentIndex: 0
                displayText: root.targetLabels[currentIndex]
                delegate: ItemDelegate { required property int index; width: targetPicker.width; text: root.targetLabels[index]; highlighted: targetPicker.highlightedIndex === index }
                onCurrentTextChanged: root.selectedTarget = currentText
                Layout.preferredWidth: 180
            }
            Button {
                text: root.assignmentMode ? "Appuie maintenant…" : "Choisir une touche/bouton"
                enabled: !root.assignmentMode
                onClicked: root.assignmentMode = true
            }
            Label {
                Layout.fillWidth: true
                text: root.assignments[root.selectedTarget]
                    ? root.selectedTarget + " ← " + root.assignments[root.selectedTarget]
                    : "Aucune affectation"
                color: "#555"
            }
        }
        RowLayout {
            Layout.fillWidth: true
            Label { Layout.fillWidth: true; text: lastControl ? "Reçu : " + lastControl + "  " + lastValue.toFixed(2) : "Appuie sur une touche, un bouton ou bouge un joystick."; color: "black" }
            Label { text: "Échap ou Retour pour fermer"; color: "#555" }
        }
    }
}

