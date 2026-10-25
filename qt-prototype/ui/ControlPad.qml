import QtQuick

Item {
    id: root
    property var active: ({})
    property string labelBottom: "A"
    property string labelRight: "B"
    property string labelLeft: "X"
    property string labelTop: "Y"
    property string labelLeftShoulder: "LB"
    property string labelRightShoulder: "RB"
    property string labelCenterLeft: "BACK"
    property string labelCenterRight: "START"
    property color accent: "#1769aa"

    function value(code) { return Number(active && active[code] !== undefined ? active[code] : 0) }
    function pressed(code) { return Math.abs(value(code)) > 0.18 }

    component RoundButton: Rectangle {
        required property string code
        required property string buttonText
        width: 42; height: 42; radius: 21
        color: root.pressed(code) ? root.accent : "white"
        border.color: "black"; border.width: 2
        Text { anchors.centerIn: parent; text: parent.buttonText; color: root.pressed(parent.code) ? "white" : "black"; font.bold: true }
    }

    component Shoulder: Rectangle {
        required property string code
        required property string buttonText
        width: 104; height: 28; radius: 8
        color: root.pressed(code) ? root.accent : "white"
        border.color: "black"; border.width: 2
        Text { anchors.centerIn: parent; text: parent.buttonText; color: root.pressed(parent.code) ? "white" : "black"; font.bold: true; font.pixelSize: 12 }
    }

    Shoulder { x: 45; y: 15; code: "leftshoulder"; buttonText: root.labelLeftShoulder }
    Shoulder { x: parent.width - width - 45; y: 15; code: "rightshoulder"; buttonText: root.labelRightShoulder }

    Rectangle {
        x: 28; y: 40; width: parent.width - 56; height: 225; radius: 92
        color: "#fafafa"; border.color: "black"; border.width: 3
    }
    Rectangle { x: 35; y: 175; width: 125; height: 155; radius: 62; rotation: -12; color: "#fafafa"; border.color: "black"; border.width: 3 }
    Rectangle { x: parent.width - 160; y: 175; width: 125; height: 155; radius: 62; rotation: 12; color: "#fafafa"; border.color: "black"; border.width: 3 }

    // D-pad
    Rectangle { x: 80; y: 102; width: 34; height: 92; radius: 5; color: root.pressed("dpad-up") || root.pressed("dpad-down") ? root.accent : "white"; border.color: "black"; border.width: 2 }
    Rectangle { x: 51; y: 131; width: 92; height: 34; radius: 5; color: root.pressed("dpad-left") || root.pressed("dpad-right") ? root.accent : "white"; border.color: "black"; border.width: 2 }

    // Analogue sticks; the inner disc moves with the live axis value.
    Rectangle {
        x: 153; y: 174; width: 66; height: 66; radius: 33
        color: "white"; border.color: "black"; border.width: 2
        Rectangle {
            width: 34; height: 34; radius: 17
            x: 16 + root.value("leftx") * 12; y: 16 + root.value("lefty") * 12
            color: root.pressed("leftx") || root.pressed("lefty") || root.pressed("leftstick") ? root.accent : "#dedede"
            border.color: "black"
        }
    }
    Rectangle {
        x: parent.width - 219; y: 174; width: 66; height: 66; radius: 33
        color: "white"; border.color: "black"; border.width: 2
        Rectangle {
            width: 34; height: 34; radius: 17
            x: 16 + root.value("rightx") * 12; y: 16 + root.value("righty") * 12
            color: root.pressed("rightx") || root.pressed("righty") || root.pressed("rightstick") ? root.accent : "#dedede"
            border.color: "black"
        }
    }

    RoundButton { x: parent.width - 108; y: 139; code: "face-bottom"; buttonText: root.labelBottom }
    RoundButton { x: parent.width - 66; y: 98; code: "face-right"; buttonText: root.labelRight }
    RoundButton { x: parent.width - 150; y: 98; code: "face-left"; buttonText: root.labelLeft }
    RoundButton { x: parent.width - 108; y: 57; code: "face-top"; buttonText: root.labelTop }

    RoundButton { x: parent.width / 2 - 62; y: 112; width: 48; height: 25; radius: 8; code: "back"; buttonText: root.labelCenterLeft }
    RoundButton { x: parent.width / 2 + 14; y: 112; width: 48; height: 25; radius: 8; code: "start"; buttonText: root.labelCenterRight }

    Rectangle {
        x: 45; y: 51; width: 104; height: 7; radius: 3
        color: root.value("lefttrigger") > 0.08 ? root.accent : "#d8d8d8"; border.color: "black"
        Rectangle { width: parent.width * Math.max(0, root.value("lefttrigger")); height: parent.height; color: root.accent; radius: 3 }
    }
    Rectangle {
        x: parent.width - width - 45; y: 51; width: 104; height: 7; radius: 3
        color: root.value("righttrigger") > 0.08 ? root.accent : "#d8d8d8"; border.color: "black"
        Rectangle { width: parent.width * Math.max(0, root.value("righttrigger")); height: parent.height; color: root.accent; radius: 3 }
    }
}
