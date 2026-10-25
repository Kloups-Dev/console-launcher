pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

// Geometry matches the cleaned SVG approved by the user.
// Visual correspondence only: this does not configure Dolphin.
Item {
    id: root
    property var active: ({})
    property color accent: "#1769aa"
    property bool animationsEnabled: true
    implicitWidth: 1000
    implicitHeight: 680
    function value(code) {
        const n = Number(active && active[code] !== undefined ? active[code] : 0)
        return isFinite(n) ? Math.max(-1, Math.min(1, n)) : 0
    }
    function pressed(code) { return Math.abs(value(code)) > 0.18 }
    function pressureColor(code) {
        const p = Math.max(0, value(code))
        return Qt.rgba(1 + (accent.r - 1) * p, 1 + (accent.g - 1) * p, 1 + (accent.b - 1) * p, 1)
    }
    Item {
        width: 1000; height: 680
        anchors.centerIn: parent
        scale: Math.min(root.width / width, root.height / height)
        Item {
            x: -70; width: 1122; height: 680
            Shape {
                anchors.fill: parent
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressureColor("lefttrigger")
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M215 108 C218 70 241 29 279 21 C320 11 355 30 387 51 L304 85Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressureColor("righttrigger")
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M757 49 C788 22 818 15 846 21 C889 29 916 61 925 108 L893 96 Q902 74 866 61 L795 43Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M129.37 270 A162 162 0 0 1 290 87 C394 45 472 20 565 20 C657 18 747 42 850 87 A162 162 0 0 1 1010.63 270 C1031 348 1041 437 1041 515 C1042 574 1024 628 986 651 C955 672 924 648 905 617 C879 576 881 530 866 480 Q857 451 842 433 Q838 438 841 444 A112 112 0 1 1 623 444 C631 407 658 383 694 355 C613 320 519 324 439 350 C470 377 503 406 517 444 A112 112 0 1 1 299 444 C276 477 264 511 260 550 C256 590 258 624 226 648 C205 664 180 673 162 661 C120 637 101 577 98 515 C94 430 100 350 129 270Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M290 87 C394 45 472 20 565 20 C657 18 747 42 850 87 A162 162 0 0 0 701.18 313 C707 327 708 341 694 355 C613 320 519 324 439 350 Q431 337 438.82 313 A162 162 0 0 0 290 87Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M129.37 270 A162 162 0 0 0 246 404.91 C276 408 293 422 298 446" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1010.63 270 A162 162 0 0 1 889 406.24 C865 408 850 417 842 433" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M402 250 A112 112 0 1 0 178 250 A112 112 0 1 0 402 250Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M290 172 L345.15 194.85 L368 250 L345.15 305.15 L290 328 L234.85 305.15 L212 250 L234.85 194.85Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M346 250 A56 56 0 1 0 234 250 A56 56 0 1 0 346 250Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M941 249 A102 102 0 1 0 737 249 A102 102 0 1 0 941 249Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("face-bottom") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M904 257 A61 61 0 1 0 782 257 A61 61 0 1 0 904 257Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("face-right") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M795 340 A34 34 0 1 0 727 340 A34 34 0 1 0 795 340Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("face-top") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M782 157 C799 139 822 134 850 136 C868 136 880 144 879 158 C878 172 870 177 853 176 C831 174 815 181 801 195 C791 205 781 200 777 190 C773 178 776 166 782 157Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("face-left") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M935 200 C951 195 967 205 974 224 C984 248 982 269 974 289 C968 304 955 307 944 302 C932 298 929 289 934 276 C940 257 939 244 931 230 C924 218 923 205 935 200Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("start") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M591 278 A21 21 0 1 0 549 278 A21 21 0 1 0 591 278Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M489 470 A81 81 0 1 0 327 470 A81 81 0 1 0 489 470Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M396 409 H420 Q425 409 425 414 V451 H462 Q467 451 467 456 V480 Q467 485 462 485 H425 V522 Q425 527 420 527 H396 Q391 527 391 522 V485 H354 Q349 485 349 480 V456 Q349 451 354 451 H391 V414 Q391 409 396 409Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M702.15 402.94 L761.85 402.94 L804.06 445.15 L804.06 504.85 L761.85 547.06 L702.15 547.06 L659.94 504.85 L659.94 445.15Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M769 475 A37 37 0 1 0 695 475 A37 37 0 1 0 769 475Z" }
                }
            }
            Text { x: 257; y: 37; width: 70; height: 50; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; text: "L"; font.family: "Arial"; font.pixelSize: 32; color: "#171717" }
            Text { x: 806; y: 21; width: 70; height: 50; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; text: "R"; font.family: "Arial"; font.pixelSize: 32; color: "#171717" }
            Text { x: 808; y: 232; width: 70; height: 50; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; text: "A"; font.family: "Arial"; font.pixelSize: 32; color: root.pressed("face-bottom") ? "white" : "#171717" }
            Text { x: 726; y: 315; width: 70; height: 50; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; text: "B"; font.family: "Arial"; font.pixelSize: 32; color: root.pressed("face-right") ? "white" : "#171717" }
            Text { x: 790; y: 146; width: 70; height: 50; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; text: "Y"; font.family: "Arial"; font.pixelSize: 27; color: root.pressed("face-top") ? "white" : "#171717" }
            Text { x: 920; y: 225; width: 70; height: 50; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; text: "X"; font.family: "Arial"; font.pixelSize: 27; color: root.pressed("face-left") ? "white" : "#171717" }
            Text { x: 697; y: 450; width: 70; height: 50; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; text: "C"; font.family: "Arial"; font.pixelSize: 32; color: "#171717" }
            component DirectionDot: Rectangle {
                required property string axisX
                required property string axisY
                required property real centerX
                required property real centerY
                required property real travel
                property bool showAtRest: true
                visible: showAtRest || Math.abs(root.value(axisX)) > 0.01 || Math.abs(root.value(axisY)) > 0.01
                width: 16; height: 16; radius: 8
                x: centerX - width / 2 + root.value(axisX) * travel
                y: centerY - height / 2 + root.value(axisY) * travel
                color: root.accent; border.color: "#171717"; border.width: 1.5
                Behavior on x { enabled: root.animationsEnabled; NumberAnimation { duration: 20 } }
                Behavior on y { enabled: root.animationsEnabled; NumberAnimation { duration: 20 } }
            }
            DirectionDot { axisX: "leftx"; axisY: "lefty"; centerX: 290; centerY: 250; travel: 32 }
            DirectionDot { axisX: "rightx"; axisY: "righty"; centerX: 732; centerY: 475; travel: 19; showAtRest: false }
            Repeater {
                model: [
                    {c:"dpad-up",x:395,y:413,w:26,h:38},
                    {c:"dpad-down",x:395,y:485,w:26,h:38},
                    {c:"dpad-left",x:353,y:455,w:38,h:26},
                    {c:"dpad-right",x:425,y:455,w:38,h:26}
                ]
                Rectangle {
                    required property var modelData
                    x: modelData.x; y: modelData.y; width: modelData.w; height: modelData.h; radius: 4
                    color: root.accent; opacity: root.pressed(modelData.c) ? 1 : 0
                    Behavior on opacity { enabled: root.animationsEnabled; NumberAnimation { duration: 75 } }
                }
            }
            Rectangle {
                x: 954; y: 30; width: 62; height: 34; radius: 10
                color: root.pressed("rightshoulder") ? root.accent : "white"
                border.color: "#171717"; border.width: 3.2
                Behavior on color { enabled: root.animationsEnabled; ColorAnimation { duration: 75 } }
                Text { anchors.centerIn: parent; text: "Z"; font.pixelSize: 22; color: root.pressed("rightshoulder") ? "white" : "#171717" }
            }
        }
    }
}