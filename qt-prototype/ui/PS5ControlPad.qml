pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

// Geometry matches the cleaned SVG approved by the user.
// Visual correspondence only: this does not configure an emulator.
Item {
    id: root
    property var active: ({})
    property color accent: "#1769aa"
    property bool animationsEnabled: true
    implicitWidth: 1600
    implicitHeight: 1170
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
        width: 1600; height: 1170
        anchors.centerIn: parent
        scale: Math.min(root.width / width, root.height / height)
        Item {
            x: -100; y: 0; width: 1800; height: 1260
            Shape {
                anchors.fill: parent
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3; fillColor: root.pressed("leftshoulder") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M321 198 L326 170 Q413 106 493 124 Q518 127 525 133 L529 161Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 1.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M326 170 Q424 148 525 133" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3; fillColor: root.pressed("rightshoulder") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1271 161 L1275 133 Q1282 127 1307 124 Q1387 106 1474 170 L1479 198Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 1.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1275 133 Q1376 148 1474 170" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 3; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M297 202 Q301 196 321 195 L529 160 C668 133 1126 118 1271 160 L1479 195 Q1499 196 1503 202 C1586 366 1645 568 1665 746 C1679 866 1671 963 1652 1026 C1637 1079 1598 1116 1562 1124 Q1547 1129 1531 1118 L1467 1140 Q1457 1147 1447 1130 C1412 1067 1366 934 1327 867 C1292 803 1255 775 1211 775 Q1194 773 1171 779 Q1141 786 1103 782 H697 Q659 786 629 779 Q606 773 589 775 C545 775 508 803 473 867 C434 934 388 1067 353 1130 Q343 1147 333 1140 L269 1124 Q254 1129 238 1124 C202 1116 163 1079 148 1026 C129 963 121 866 135 746 C155 568 214 366 297 202Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M529 160 Q583 167 604 210 L621 359 C626 405 625 444 596 478 C452 645 378 853 287 1084 Q272 1124 248 1125" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1271 160 Q1217 167 1196 210 L1179 359 C1174 405 1175 444 1204 478 C1348 645 1422 853 1513 1084 Q1528 1124 1552 1125" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M333 1140 C372 1080 415 944 457 868 C497 798 535 773 582 774 M1467 1140 C1428 1080 1385 944 1343 868 C1303 798 1265 773 1218 774" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M642 148 C773 132 1027 132 1158 148 Q1211 153 1213 196 L1172 419 Q1165 463 1134 480 Q1122 487 1092 487 H708 Q678 487 666 480 Q635 463 628 419 L587 196 Q589 153 642 148Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("touchpad") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M642 148 C773 132 1027 132 1158 148 Q1206 152 1199 199 L1163 403 Q1157 445 1124 459 Q1110 465 1087 465 H713 Q690 465 676 459 Q643 445 637 403 L601 199 Q594 152 642 148Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("back") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M539.139 215.721 L539.139 215.721 Q555.767 212.187 559.302 228.815 L566.163 261.094 Q569.698 277.723 553.069 281.257 L553.069 281.257 Q536.44 284.792 532.906 268.163 L526.045 235.884 Q522.51 219.256 539.139 215.721 Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 1.7; fillColor: root.pressed("back") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M539 187 V198 M526 191 L531 201 M552 191 L547 201" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("start") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1257.861 215.721 L1257.861 215.721 Q1274.49 219.256 1270.955 235.884 L1264.094 268.163 Q1260.56 284.792 1243.931 281.257 L1243.931 281.257 Q1227.302 277.723 1230.837 261.094 L1237.698 228.815 Q1241.233 212.187 1257.861 215.721 Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 1.7; fillColor: root.pressed("start") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1248 186 H1268 M1248 194 H1268 M1248 202 H1268" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("dpad-up") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M407 291 Q428 287 449 291 Q471 294 471 314 L468 350 Q468 361 459 371 L442 389 Q428 403 414 389 L397 371 Q388 361 388 350 L385 314 Q385 294 407 291Z" }
                }
                ShapePath {
                    strokeColor: "transparent"; strokeWidth: 2; fillColor: root.pressed("dpad-up") ? "white" : "#111"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M419 325 Q415 325 418 322 L426 314 Q429 312 431 314 L439 322 Q442 325 438 325Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("dpad-down") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M407 538 Q428 542 449 538 Q471 535 471 515 L468 479 Q468 468 459 458 L442 440 Q428 426 414 440 L397 458 Q388 468 388 479 L385 515 Q385 535 407 538Z" }
                }
                ShapePath {
                    strokeColor: "transparent"; strokeWidth: 2; fillColor: root.pressed("dpad-down") ? "white" : "#111"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M419 503 H438 Q442 503 439 506 L431 514 Q429 516 426 514 L418 506 Q415 503 419 503Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("dpad-left") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M304 393 Q300 414 304 435 Q307 457 327 457 L363 454 Q374 454 384 445 L402 428 Q416 414 402 400 L384 383 Q374 374 363 374 L327 371 Q307 371 304 393Z" }
                }
                ShapePath {
                    strokeColor: "transparent"; strokeWidth: 2; fillColor: root.pressed("dpad-left") ? "white" : "#111"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M338 405 V424 Q338 428 335 425 L327 417 Q325 414 327 412 L335 404 Q338 401 338 405Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("dpad-right") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M552 393 Q556 414 552 435 Q549 457 529 457 L493 454 Q482 454 472 445 L454 428 Q440 414 454 400 L472 383 Q482 374 493 374 L529 371 Q549 371 552 393Z" }
                }
                ShapePath {
                    strokeColor: "transparent"; strokeWidth: 2; fillColor: root.pressed("dpad-right") ? "white" : "#111"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M517 405 Q517 401 520 404 L528 412 Q530 414 528 417 L520 425 Q517 428 517 424Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("face-top") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1420 303 A50 50 0 1 0 1320 303 A50 50 0 1 0 1420 303Z" }
                }
                ShapePath {
                    strokeColor: root.pressed("face-top") ? "white" : "#171717"; strokeWidth: 5; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1370 271 L1342 319 H1398Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("face-left") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1309 414 A50 50 0 1 0 1209 414 A50 50 0 1 0 1309 414Z" }
                }
                ShapePath {
                    strokeColor: root.pressed("face-left") ? "white" : "#171717"; strokeWidth: 5; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1235 390 H1283 V438 H1235Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("face-right") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1531 414 A50 50 0 1 0 1431 414 A50 50 0 1 0 1531 414Z" }
                }
                ShapePath {
                    strokeColor: root.pressed("face-right") ? "white" : "#171717"; strokeWidth: 5; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1510 414 A29 29 0 1 0 1452 414 A29 29 0 1 0 1510 414Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("face-bottom") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1420 525 A50 50 0 1 0 1320 525 A50 50 0 1 0 1420 525Z" }
                }
                ShapePath {
                    strokeColor: root.pressed("face-bottom") ? "white" : "#171717"; strokeWidth: 5; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1347 502 L1393 548 M1393 502 L1347 548" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M774 621 A115 115 0 1 0 544 621 A115 115 0 1 0 774 621Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 1.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M757 621 A98 98 0 1 0 561 621 A98 98 0 1 0 757 621Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 1.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M749 621 A90 90 0 1 0 569 621 A90 90 0 1 0 749 621Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("leftstick") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M717 621 A58 58 0 1 0 601 621 A58 58 0 1 0 717 621Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1256 621 A115 115 0 1 0 1026 621 A115 115 0 1 0 1256 621Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 1.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1239 621 A98 98 0 1 0 1043 621 A98 98 0 1 0 1239 621Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 1.7; fillColor: "transparent"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    PathSvg { path: "M1231 621 A90 90 0 1 0 1051 621 A90 90 0 1 0 1231 621Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("rightstick") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M1199 621 A58 58 0 1 0 1083 621 A58 58 0 1 0 1199 621Z" }
                }
                ShapePath {
                    strokeColor: "#171717"; strokeWidth: 2; fillColor: root.pressed("guide") ? root.accent : "white"
                    joinStyle: ShapePath.RoundJoin; capStyle: ShapePath.RoundCap
                    Behavior on fillColor { enabled: root.animationsEnabled; ColorAnimation { duration: 65 } }
                    PathSvg { path: "M936 614 A36 36 0 1 0 864 614 A36 36 0 1 0 936 614Z" }
                }
            }
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
            DirectionDot { axisX: "leftx"; axisY: "lefty"; centerX: 659; centerY: 621; travel: 33 }
            DirectionDot { axisX: "rightx"; axisY: "righty"; centerX: 1141; centerY: 621; travel: 33 }
            component TriggerIndicator: Rectangle {
                required property string code
                required property string label
                width: 110; height: 46; radius: 12
                color: root.pressureColor(code); border.color: "#171717"; border.width: 2
                Behavior on color { enabled: root.animationsEnabled; ColorAnimation { duration: 45 } }
                Text { anchors.centerIn: parent; text: parent.label + " " + Math.round(Math.max(0, root.value(parent.code)) * 100) + "%"; font.pixelSize: 21; color: root.value(parent.code) > 0.6 ? "white" : "#171717" }
            }
            TriggerIndicator { x: 360; y: 40; code: "lefttrigger"; label: "L2" }
            TriggerIndicator { x: 1330; y: 40; code: "righttrigger"; label: "R2" }
            Text { x: 384; y: 130; width: 80; height: 32; text: "L1"; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; font.pixelSize: 23; color: root.pressed("leftshoulder") ? "white" : "#171717" }
            Text { x: 1336; y: 130; width: 80; height: 32; text: "R1"; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; font.pixelSize: 23; color: root.pressed("rightshoulder") ? "white" : "#171717" }
        }
    }
}