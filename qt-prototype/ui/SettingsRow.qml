// One line of the settings panel: a choice ("row"), a text block ("block")
// or a text field ("input"). Used by SettingsPanel.qml.

import QtQuick

Item {
    id: line
    required property var item
    required property bool focused
    property color text: "#171717"
    property color accent: "#2864c7"
    signal activated()
    signal submitted(string value)

    width: ListView.view ? ListView.view.width : 600
    height: item.type === "block" ? blockCol.implicitHeight + 20
          : item.type === "input" ? 78
          : Math.max(54, rowText.implicitHeight + 28)

    function focusInput() { field.forceActiveFocus(); }

    // ---- Choice ----
    Rectangle {
        visible: line.item.type === "row"
        anchors.fill: parent; anchors.topMargin: 3; anchors.bottomMargin: 3
        radius: 10
        color: line.focused ? "#e1eafa" : choiceMouse.containsMouse ? "#f0f5fc" : "#ffffff"
        border.width: 1
        border.color: line.focused ? (line.item.danger ? "#c84534" : line.accent) : "#dce2ea"
        opacity: line.item.dim ? 0.55 : 1
        Column {
            id: rowText
            anchors.left: parent.left; anchors.leftMargin: 14
            anchors.right: rowValue.left; anchors.rightMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            spacing: 5
            Text {
                id: rowLabel
                width: parent.width
                text: line.item.label || ""
                textFormat: Text.StyledText; wrapMode: Text.Wrap
                color: line.text; font.pixelSize: 15; font.weight: Font.DemiBold
            }
            Text {
                width: parent.width
                visible: !!line.item.description
                text: line.item.description || ""
                textFormat: Text.PlainText; wrapMode: Text.WordWrap
                color: "#536174"; font.pixelSize: 13; lineHeight: 1.2
            }
        }
        Text {
            id: rowValue
            anchors.right: parent.right; anchors.rightMargin: 14
            anchors.verticalCenter: parent.verticalCenter
            text: (line.item.left ? "<font color='#667085'>‹ ›</font>  " : "") + (line.item.value || "")
            textFormat: Text.StyledText
            color: line.text; font.pixelSize: 15; font.weight: Font.DemiBold
            width: Math.min(implicitWidth, parent.width * 0.38)
            elide: Text.ElideRight
        }
        MouseArea { id: choiceMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: line.activated() }
    }

    // ---- Text block ----
    Rectangle {
        visible: line.item.type === "block"
        anchors.fill: parent; anchors.topMargin: 4; anchors.bottomMargin: 4
        radius: 10; color: "#edf1f7"
        Column {
            id: blockCol
            x: 14; y: 10; width: parent.width - 28; spacing: 6
            Text {
                visible: !!line.item.title
                width: parent.width; wrapMode: Text.WordWrap
                text: line.item.title || ""; color: "#24364b"; font.pixelSize: 15; font.bold: true
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
        radius: 10; color: "#ffffff"
        border.width: 1; border.color: line.focused || field.activeFocus ? line.accent : "transparent"
        Text { x: 14; y: 8; text: line.item.label || ""; color: line.text; opacity: 0.7; font.pixelSize: 13 }
        Rectangle {
            x: 14; y: 30; width: parent.width - 28; height: 36; radius: 8
            color: "#ffffff"; border.color: "#bcbcb7"
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
