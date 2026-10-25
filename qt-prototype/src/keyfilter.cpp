#include "keyfilter.h"

#include <QGuiApplication>
#include <QKeyEvent>
#include <QWindow>

namespace {
QString controlName(int key) {
    switch (key) {
    case Qt::Key_Up: return "up";
    case Qt::Key_Down: return "down";
    case Qt::Key_Left: return "left";
    case Qt::Key_Right: return "right";
    case Qt::Key_Return:
    case Qt::Key_Enter: return "enter";
    case Qt::Key_Escape: return "escape";
    case Qt::Key_Backspace: return "backspace";
    case Qt::Key_Space: return "space";
    default:
        if (key >= Qt::Key_A && key <= Qt::Key_Z)
            return QString(QChar('a' + key - Qt::Key_A));
        return {};
    }
}
} // namespace

bool KeyFilter::eventFilter(QObject *watched, QEvent *event) {
    const bool pressed = event->type() == QEvent::KeyPress;
    const bool released = event->type() == QEvent::KeyRelease;
    if ((!pressed && !released) || !qobject_cast<QWindow *>(watched))
        return false;
    // Text fields in themes keep their keys (except Escape, handled by QML).
    QObject *focus = QGuiApplication::focusObject();
    if (focus && focus->inherits("QQuickTextInput"))
        return false;

    const auto *k = static_cast<QKeyEvent *>(event);
    if (!k->isAutoRepeat()) {
        const QString control = controlName(k->key());
        if (!control.isEmpty())
            Q_EMIT inputEvent({{"source", "keyboard"},
                               {"control", control},
                               {"value", pressed ? 1.0 : 0.0}});
    }
    if (released)
        return false;
    QString a;
    switch (k->key()) {
    case Qt::Key_Up: a = "up"; break;
    case Qt::Key_Down: a = "down"; break;
    case Qt::Key_Left: a = "left"; break;
    case Qt::Key_Right: a = "right"; break;
    case Qt::Key_Return:
    case Qt::Key_Enter:
    case Qt::Key_Space: a = "confirm"; break;
    case Qt::Key_Escape:
    case Qt::Key_Backspace: a = "back"; break;
    case Qt::Key_F: a = "favorite"; break;
    case Qt::Key_Tab:
    case Qt::Key_M: a = "menu"; break;
    case Qt::Key_PageUp:
    case Qt::Key_Q: a = "pageLeft"; break;
    case Qt::Key_PageDown:
    case Qt::Key_E: a = "pageRight"; break;
    case Qt::Key_F11: Q_EMIT fullscreen(); return true;
    default: return false;
    }
    Q_EMIT nav(a);
    return true;
}
