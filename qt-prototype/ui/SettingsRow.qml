// One line of the settings panel: a choice ("row"), a text block ("block")
// or a text field ("input"). Used by SettingsPanel.qml.

import QtQuick

Item {
    id: line
    required property var item
    required property bool focused
    property color text: "#e8ecf4"
    property color accent: "#ffc440"
    signal activated()
    signal submitted(string value)

    width: ListView.view ? ListView.view.width : 600
    height: item.type === "block" ? blockCol.implicitHeight + 20
          : item.type === "input" ? 78
          : Math.max(48, rowLabel.implicitHeight + 24)

    function focusInput() { field.forceActiveFocus(); }

    // ---- Choice ----
    Rectangle {
        visible: line.item.type === "row"
        anchors.fill: parent; anchors.topMargin: 3; anchors.bottomMargin: 3
        radius: 10
        color: line.focused ? "#232b3d" : "#1b2130"
        border.width: 1
        border.color: line.focused ? (line.item.danger ? "#ff6a5a" : line.accent) : "transparent"
        opacity: line.item.dim ? 0.55 : 1
        Text {
            id: rowLabel
            anchors.left: parent.left; anchors.leftMargin: 14
            anchors.right: rowValue.left; anchors.rightMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            text: line.item.label || ""
            textFormat: Text.StyledText; wrapMode: Text.Wrap
            color: line.text; font.pixelSize: 15
        }
        Text {
            id: rowValue
            anchors.right: parent.right; anchors.rightMargin: 14
            anchors.verticalCenter: parent.verticalCenter
            text: (line.item.left ? "<font color='#6b7488'>‹ ›</font>  " : "") + (line.item.value || "")
            textFormat: Text.StyledText
            color: line.text; font.pixelSize: 15; font.weight: Font.DemiBold
        }
        MouseArea { anchors.fill: parent; onClicked: line.activated() }
    }

    // ---- Text block ----
    Rectangle {
        visible: line.item.type === "block"
        anchors.fill: parent; anchors.topMargin: 4; anchors.bottomMargin: 4
        radius: 10; color: "#111520"
        Column {
            id: blockCol
            x: 14; y: 10; width: parent.width - 28; spacing: 6
            Text {
                visible: !!line.item.title
                text: line.item.title || ""; color: line.text; opacity: 0.65; font.pixelSize: 13
            }
            Text {
                visible: !!line.item.html
                width: parent.width
                text: line.item.html || ""
                textFormat: Text.StyledText; wrapMode: Text.Wrap
                color: line.text; font.pixelSize: 14; lineHeight: 1.15
            }
        }
    }

    // ---- Text field ----
    Rectangle {
        visible: line.item.type === "input"
        anchors.fill: parent; anchors.topMargin: 3; anchors.bottomMargin: 3
        radius: 10; color: "#1b2130"
        border.width: 1; border.color: line.focused || field.activeFocus ? line.accent : "transparent"
        Text { x: 14; y: 8; text: line.item.label || ""; color: line.text; opacity: 0.7; font.pixelSize: 13 }
        Rectangle {
            x: 14; y: 30; width: parent.width - 28; height: 36; radius: 8
            color: "#0b0d12"; border.color: "#2a3244"
            TextInput {
                id: field
                anchors.fill: parent; anchors.margins: 9
                text: line.item.value || ""
                color: line.text; font.pixelSize: 14; clip: true
                selectByMouse: true
                Keys.onReturnPressed: { line.submitted(text); focus = false; }
                Keys.onEnterPressed: { line.submitted(text); focus = false; }
                Keys.onEscapePressed: focus = false
                onTextChanged: if (line.item) line.item.value = text
            }
        }
        MouseArea { anchors.fill: parent; z: -1; onClicked: field.forceActiveFocus() }
    }
}
