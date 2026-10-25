pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

// Front-only Xbox drawing, traced from the user-provided reference.
Item {
    id: root
    property var active: ({})
    property color accent: "#1769aa"
    property bool animationsEnabled: true
    implicitWidth: 704
    implicitHeight: 540

    function value(code) {
        const n = Number(active && active[code] !== undefined ? active[code] : 0)
        return isFinite(n) ? Math.max(-1, Math.min(1, n)) : 0
    }
    function pressed(code) { return Math.abs(value(code)) > 0.18 }

    Item {
        id: drawing
        width: 704; height: 540
        scale: Math.min(root.width / width, root.height / height)
        anchors.centerIn: parent

        Shape {
            anchors.fill: parent
            ShapePath {
                strokeColor: "#111"; strokeWidth: 2.5
                fillColor: root.pressed("leftshoulder") ? root.accent : "white"
                Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 90 } }
                PathSvg { path: "M149 122 Q159 92 222 80 Q245 72 272 91 L261 112 Q237 87 217 95Z" }
            }
            ShapePath {
                strokeColor: "#111"; strokeWidth: 2.5
                fillColor: root.pressed("rightshoulder") ? root.accent : "white"
                Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 90 } }
                PathSvg { path: "M432 91 Q453 73 477 80 Q539 91 554 122 L485 97 Q459 85 443 113Z" }
            }
        }
        Shape {
            z: 0
            anchors.fill: parent
            ShapePath {
                strokeColor: "#111"; strokeWidth: 2.5; fillColor: "white"
                PathSvg { path: "M149 122 Q183 101 218 94 Q240 88 255 107 L291 146 Q299 155 313 155 H391 Q404 155 414 143 L448 106 Q464 88 485 96 Q534 104 555 123 Q575 135 585 164 L627 282 Q648 346 647 398 Q648 456 614 482 Q598 496 583 493 Q571 489 561 477 L507 409 Q482 376 457 376 H249 Q224 375 201 401 L140 476 Q127 493 113 493 Q94 492 78 474 Q56 450 58 390 Q61 339 81 280 L119 174 Q130 137 149 122Z" }
            }
            ShapePath {
                strokeColor: "#111"; strokeWidth: 1.3; fillColor: "transparent"
                PathSvg { path: "M113 493 Q120 468 140 445 L191 384 Q216 357 245 357 H459 Q490 357 515 386 L567 451 Q585 474 591 493 M261 112 L272 91 H432 L443 113" }
            }
        }

        Shape {
            z: 0
            anchors.fill: parent
            ShapePath {
                strokeColor: "#111"; strokeWidth: 2.5; fillColor: "white"
                PathSvg { path: "M261 112 L272 91 H432 L443 113 L414 144 Q404 155 391 155 H313 Q299 155 291 146Z" }
            }
        }
        component ButtonDisc: Rectangle {
            z: 3
            required property string code
            required property string label
            property real diameter: 40
            width: diameter; height: diameter; radius: diameter / 2
            color: root.pressed(code) ? root.accent : "white"
            scale: root.pressed(code) ? 0.94 : 1
            Behavior on color { enabled: root.animationsEnabled; ColorAnimation { duration: 90 } }
            Behavior on scale { enabled: root.animationsEnabled; NumberAnimation { duration: 75; easing.type: Easing.OutCubic } }
            border.color: "#111"; border.width: 2
            Text {
                anchors.centerIn: parent; text: parent.label
                color: root.pressed(parent.code) ? "white" : "#111"
                font.family: "Arial"; font.pixelSize: parent.diameter >= 40 ? 30 : 13
            }
        }

        // Rear triggers represented as circles above the front bumpers.
        component Trigger: Rectangle {
            required property string code
            required property string label
            width: 46; height: 46; radius: 23
            color: "white"; border.color: "#111"; border.width: 2
            Rectangle {
                anchors.fill: parent; anchors.margins: 3; radius: width / 2
                color: root.accent; opacity: Math.max(0, root.value(parent.code)) * 0.2
                Behavior on opacity { enabled: root.animationsEnabled; NumberAnimation { duration: 40 } }
            }
            Canvas {
                id: pressureRing
                anchors.fill: parent; antialiasing: true
                property real pressure: Math.max(0, root.value(parent.code))
                onPressureChanged: requestPaint()
                Connections { target: root; function onAccentChanged() { pressureRing.requestPaint() } }
                onPaint: {
                    const ctx = getContext("2d")
                    ctx.reset()
                    ctx.strokeStyle = root.accent
                    ctx.lineWidth = 4
                    ctx.lineCap = "round"
                    if (pressure > 0.001) {
                        ctx.beginPath()
                        ctx.arc(width / 2, height / 2, width / 2 - 4, -Math.PI / 2, -Math.PI / 2 + pressure * Math.PI * 2)
                        ctx.stroke()
                    }
                }
            }
            Text {
                y: parent.height + 3; width: parent.width; horizontalAlignment: Text.AlignHCenter
                text: Math.round(Math.max(0, root.value(parent.code)) * 100) + " %"
                font.pixelSize: 12; color: "#555"
            }            Text { anchors.centerIn: parent; text: parent.label; color: "#111"; font.pixelSize: 16; font.bold: true }
        }
        component BumperIndicator: Rectangle {
            required property string code
            required property string label
            z: 3
            width: 46; height: 46; radius: 10
            color: root.pressed(code) ? root.accent : "white"
            border.color: "#111"; border.width: 2
            scale: root.pressed(code) ? 0.94 : 1
            Behavior on color { enabled: root.animationsEnabled; ColorAnimation { duration: 90 } }
            Behavior on scale { enabled: root.animationsEnabled; NumberAnimation { duration: 75; easing.type: Easing.OutCubic } }
            Text {
                anchors.centerIn: parent; text: parent.label
                color: root.pressed(parent.code) ? "white" : "#111"
                font.pixelSize: 16; font.bold: true
            }
        }
        BumperIndicator { x: 184; y: 0; code: "leftshoulder"; label: "LB" }
        BumperIndicator { x: 475; y: 0; code: "rightshoulder"; label: "RB" }
        Trigger { x: 122; y: 22; code: "lefttrigger"; label: "LT" }
        Trigger { x: 537; y: 22; code: "righttrigger"; label: "RT" }
        component Stick: Rectangle {
            z: 2
            required property string axisX
            required property string axisY
            required property string clickCode
            width: 104; height: 104; radius: 52
            color: "white"; border.color: "#111"; border.width: 1.3
            Rectangle {
                anchors.centerIn: parent; width: 74; height: 74; radius: 37
                color: root.pressed(parent.clickCode) ? "#d8eafa" : "white"
                border.color: "#111"; border.width: 2
            }
            Rectangle { anchors.centerIn: parent; width: 14; height: 14; radius: 7; color: "#edf3f8"; border.color: "#c5d6e4" }
            Rectangle { anchors.centerIn: parent; width: 65; height: 1; color: "#d0d0d0" }
            Rectangle { anchors.centerIn: parent; width: 1; height: 65; color: "#d0d0d0" }
            Rectangle {
                width: 12; height: 12; radius: 6
                x: 46 + root.value(parent.axisX) * 30
                y: 46 + root.value(parent.axisY) * 30
                color: root.accent; border.color: "#111"; border.width: 1
                Behavior on x { enabled: root.animationsEnabled; NumberAnimation { duration: 20 } }
                Behavior on y { enabled: root.animationsEnabled; NumberAnimation { duration: 20 } }
            }
        }
        Stick { x: 146; y: 133; axisX: "leftx"; axisY: "lefty"; clickCode: "leftstick" }
        Stick { x: 377; y: 231; axisX: "rightx"; axisY: "righty"; clickCode: "rightstick" }

        Shape {
            z: 0
            anchors.fill: parent
            ShapePath {
                strokeColor: "#111"; strokeWidth: 2.5; fillColor: "white"
                PathSvg { path: "M272 246 H283 Q290 246 290 253 V269 Q290 275 296 275 H311 Q318 275 318 282 V294 Q318 301 311 301 H296 Q290 301 290 307 V321 Q290 329 282 329 H272 Q263 329 263 321 V307 Q263 301 257 301 H242 Q235 301 235 294 V283 Q235 275 242 275 H257 Q263 275 263 269 V254 Q263 246 272 246Z" }
            }
        }
        Repeater {
            model: [
                {code:"dpad-up", x:266, y:250, w:21, h:26},
                {code:"dpad-down", x:266, y:299, w:21, h:26},
                {code:"dpad-left", x:239, y:278, w:26, h:20},
                {code:"dpad-right", x:289, y:278, w:25, h:20}
            ]
            Rectangle {
                required property var modelData
                x: modelData.x; y: modelData.y; width: modelData.w; height: modelData.h; radius: 3
                color: root.accent; opacity: root.pressed(modelData.code) ? 1 : 0
                Behavior on opacity { enabled: root.animationsEnabled; NumberAnimation { duration: 75 } }
            }
        }
        ButtonDisc { x: 490; y: 127; code: "y"; label: "Y" }
        ButtonDisc { x: 449; y: 165; code: "x"; label: "X" }
        ButtonDisc { x: 529; y: 169; code: "b"; label: "B" }
        ButtonDisc { x: 489; y: 206; code: "a"; label: "A" }
        ButtonDisc { x: 332; y: 104; code: "guide"; label: "" }
        ButtonDisc { x: 297; y: 177; code: "back"; label: "▣"; diameter: 24 }
        ButtonDisc { x: 383; y: 177; code: "start"; label: "≡"; diameter: 24 }
    }
}
