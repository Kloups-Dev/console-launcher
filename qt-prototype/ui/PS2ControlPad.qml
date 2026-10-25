pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

// Geometry matches the cleaned SVG approved by the user.
// Visual correspondence only: this does not configure PCSX2.
Item {
    id: root
    property var active: ({})
    property color accent: "#1769aa"
    property bool animationsEnabled: true
    implicitWidth: 1240
    implicitHeight: 870
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
        width: 1240; height: 870
        anchors.centerIn: parent
        scale: Math.min(root.width / width, root.height / height)
        Item {
            x: -180; y: -130; width: 1600; height: 1200
            Shape {
                anchors.fill: parent
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M342 338 L354 279 Q359 256 384 248 Q444 225 504 248 Q529 256 534 279 L546 338Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1054 338 L1066 279 Q1071 256 1096 248 Q1156 225 1216 248 Q1241 256 1246 279 L1258 338Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M256 497 C256 393 340 309 444 309 Q499 309 546 339 H759 Q800 352 841 339 H1054 Q1101 309 1156 309 C1260 309 1344 393 1344 497 L1395 818 C1411 897 1372 953 1308 969 C1238 986 1184 946 1154 895 L1075 771 Q1038 821 978 821 Q881 822 850 728 H750 Q719 822 622 821 Q562 821 525 771 L446 895 C416 946 362 986 292 969 C228 953 189 897 205 818Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M546 339 H759 Q800 352 841 339 H1054 A188 188 0 1 1 1099 676 A125 125 0 1 1 975 548 A188 188 0 0 1 1054 339 H546 A188 188 0 0 1 625 548 A125 125 0 1 1 501 676 A188 188 0 1 1 546 339Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M566 355 H1034 M749 693 H851" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M403 356 H485 Q496 356 496 367 V434 Q496 444 506 444 H574 Q585 444 585 455 V538 Q585 549 574 549 H506 Q496 549 496 559 V626 Q496 637 485 637 H403 Q392 637 392 626 V559 Q392 549 382 549 H314 Q303 549 303 538 V455 Q303 444 314 444 H382 Q392 444 392 434 V367 Q392 356 403 356Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("dpad-up") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M420 396 H468 Q480 396 480 409 V439 Q480 451 468 461 L450 475 Q444 479 438 475 L420 461 Q408 451 408 439 V409 Q408 396 420 396Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("dpad-down") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M420 598 H468 Q480 598 480 585 V555 Q480 543 468 533 L450 519 Q444 515 438 519 L420 533 Q408 543 408 555 V585 Q408 598 420 598Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("dpad-left") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M343 473 Q343 460 356 460 H386 Q398 460 408 472 L422 490 Q426 497 422 504 L408 522 Q398 534 386 534 H356 Q343 534 343 521Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("dpad-right") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M545 473 Q545 460 532 460 H502 Q490 460 480 472 L466 490 Q462 497 466 504 L480 522 Q490 534 502 534 H532 Q545 534 545 521Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1114 356 H1197 Q1208 356 1208 367 V434 Q1208 444 1218 444 H1285 Q1296 444 1296 455 V538 Q1296 549 1285 549 H1218 Q1208 549 1208 559 V626 Q1208 637 1197 637 H1114 Q1103 637 1103 626 V559 Q1103 549 1093 549 H1025 Q1014 549 1014 538 V455 Q1014 444 1025 444 H1093 Q1103 444 1103 434 V367 Q1103 356 1114 356Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("face-top") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1198 408 A42 42 0 1 0 1114 408 A42 42 0 1 0 1198 408Z" }
                }
                ShapePath {
                    strokeColor: root.pressed("face-top") ? "white" : "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1156 384 L1133 426 H1179Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("face-right") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1287 497 A42 42 0 1 0 1203 497 A42 42 0 1 0 1287 497Z" }
                }
                ShapePath {
                    strokeColor: root.pressed("face-right") ? "white" : "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1268 497 A23 23 0 1 0 1222 497 A23 23 0 1 0 1268 497Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("face-bottom") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1198 585 A42 42 0 1 0 1114 585 A42 42 0 1 0 1198 585Z" }
                }
                ShapePath {
                    strokeColor: root.pressed("face-bottom") ? "white" : "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1138 567 L1174 603 M1174 567 L1138 603" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("face-left") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1109 497 A42 42 0 1 0 1025 497 A42 42 0 1 0 1109 497Z" }
                }
                ShapePath {
                    strokeColor: root.pressed("face-left") ? "white" : "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1047 477 H1087 V517 H1047Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("back") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M668 475 H713 Q724 475 724 486 V505 Q724 516 713 516 H668 Q657 516 657 505 V486 Q657 475 668 475Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("start") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M887 475 L941 489 Q955 493 945 501 L887 516 Q876 519 876 508 V485 Q876 472 887 475Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M710 670 A88 88 0 1 0 534 670 A88 88 0 1 0 710 670Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("leftstick") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M702 670 A80 80 0 1 0 542 670 A80 80 0 1 0 702 670Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1066 670 A88 88 0 1 0 890 670 A88 88 0 1 0 1066 670Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("rightstick") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1058 670 A80 80 0 1 0 898 670 A80 80 0 1 0 1058 670Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("leftshoulder") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M344 150 H394 Q408 150 408 164 V194 Q408 208 394 208 H344 Q330 208 330 194 V164 Q330 150 344 150Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressureColor("lefttrigger")
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M464 165 H514 Q528 165 528 179 V209 Q528 223 514 223 H464 Q450 223 450 209 V179 Q450 165 464 165Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressed("rightshoulder") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1204 150 H1254 Q1268 150 1268 164 V194 Q1268 208 1254 208 H1204 Q1190 208 1190 194 V164 Q1190 150 1204 150Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3.2; fillColor: root.pressureColor("righttrigger")
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1084 165 H1134 Q1148 165 1148 179 V209 Q1148 223 1134 223 H1084 Q1070 223 1070 209 V179 Q1070 165 1084 165Z" }
                }
            }
            Text { x: 334; y: 154; width: 70; height: 50; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; text: "L1"; font.family: "Arial"; font.pixelSize: 23; color: root.pressed("leftshoulder") ? "white" : "#171717" }
            Text { x: 454; y: 169; width: 70; height: 50; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; text: "L2"; font.family: "Arial"; font.pixelSize: 23; color: root.pressed("lefttrigger") ? "white" : "#171717" }
            Text { x: 1194; y: 154; width: 70; height: 50; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; text: "R1"; font.family: "Arial"; font.pixelSize: 23; color: root.pressed("rightshoulder") ? "white" : "#171717" }
            Text { x: 1074; y: 169; width: 70; height: 50; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; text: "R2"; font.family: "Arial"; font.pixelSize: 23; color: root.pressed("righttrigger") ? "white" : "#171717" }
            component DirectionDot: Rectangle {
                required property string axisX
                required property string axisY
                required property real centerX
                required property real centerY
                required property real travel
                width: 16; height: 16; radius: 8
                x: centerX - width / 2 + root.value(axisX) * travel
                y: centerY - height / 2 + root.value(axisY) * travel
                color: root.accent; border.color: "#171717"; border.width: 1.5
                Behavior on x { enabled: root.animationsEnabled; NumberAnimation { duration: 20 } }
                Behavior on y { enabled: root.animationsEnabled; NumberAnimation { duration: 20 } }
            }
            DirectionDot { axisX: "leftx"; axisY: "lefty"; centerX: 622; centerY: 670; travel: 49 }
            DirectionDot { axisX: "rightx"; axisY: "righty"; centerX: 978; centerY: 670; travel: 49 }
        }
    }
}