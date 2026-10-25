#include "keyfilter.h"

#include <QGuiApplication>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QWheelEvent>
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
    case Qt::Key_Tab: return "tab";
    case Qt::Key_Shift: return "shift";
    case Qt::Key_Control: return "ctrl";
    case Qt::Key_Alt: return "alt";
    case Qt::Key_CapsLock: return "capslock";
    case Qt::Key_Insert: return "insert";
    case Qt::Key_Delete: return "delete";
    case Qt::Key_Home: return "home";
    case Qt::Key_End: return "end";
    case Qt::Key_PageUp: return "pageup";
    case Qt::Key_PageDown: return "pagedown";
    case Qt::Key_F1: return "f1";
    case Qt::Key_F2: return "f2";
    case Qt::Key_F3: return "f3";
    case Qt::Key_F4: return "f4";
    case Qt::Key_F5: return "f5";
    case Qt::Key_F6: return "f6";
    case Qt::Key_F7: return "f7";
    case Qt::Key_F8: return "f8";
    case Qt::Key_F9: return "f9";
    case Qt::Key_F10: return "f10";
    case Qt::Key_F11: return "f11";
    case Qt::Key_F12: return "f12";
    default:
        if (key >= Qt::Key_0 && key <= Qt::Key_9)
            return QString(QChar('0' + key - Qt::Key_0));
        if (key >= Qt::Key_A && key <= Qt::Key_Z)
            return QString(QChar('a' + key - Qt::Key_A));
        return {};
    }
}
} // namespace

bool KeyFilter::eventFilter(QObject *watched, QEvent *event) {
    const bool pressed = event->type() == QEvent::KeyPress;
    const bool released = event->type() == QEvent::KeyRelease;
    const bool mousePress = event->type() == QEvent::MouseButtonPress;
    const bool mouseRelease = event->type() == QEvent::MouseButtonRelease;
    const bool mouseMove = event->type() == QEvent::MouseMove;
    const bool wheel = event->type() == QEvent::Wheel;
    if ((!pressed && !released && !mousePress && !mouseRelease && !mouseMove && !wheel)
        || !qobject_cast<QWindow *>(watched))
        return false;
    // Text fields in themes keep their keys (except Escape, handled by QML).
    QObject *focus = QGuiApplication::focusObject();
    if (focus && focus->inherits("QQuickTextInput"))
        return false;

    if (mousePress || mouseRelease) {
        const auto *m = static_cast<QMouseEvent *>(event);
        QString control;
        switch (m->button()) {
        case Qt::LeftButton: control = "mouse-left"; break;
        case Qt::RightButton: control = "mouse-right"; break;
        case Qt::MiddleButton: control = "mouse-middle"; break;
        case Qt::XButton1: control = "mouse-x1"; break;
        case Qt::XButton2: control = "mouse-x2"; break;
        default: break;
        }
        if (!control.isEmpty())
            Q_EMIT inputEvent({{"source", "mouse"},
                               {"control", control},
                               {"value", mousePress ? 1.0 : 0.0}});
        return false;
    }
    if (mouseMove) {
        static QPoint last;
        const auto *m = static_cast<QMouseEvent *>(event);
        const QPoint now = m->position().toPoint();
        if (!last.isNull()) {
            const QPoint delta = now - last;
            if (!delta.isNull()) {
                Q_EMIT inputEvent({{"source", "mouse"},
                                   {"control", "mousex"},
                                   {"value", qBound(-1.0, delta.x() / 20.0, 1.0)}});
                Q_EMIT inputEvent({{"source", "mouse"},
                                   {"control", "mousey"},
                                   {"value", qBound(-1.0, delta.y() / 20.0, 1.0)}});
            }
        }
        last = now;
        return false;
    }
    if (wheel) {
        const auto *w = static_cast<QWheelEvent *>(event);
        const qreal delta = w->angleDelta().y() / 120.0;
        if (delta != 0)
            Q_EMIT inputEvent({{"source", "mouse"},
                               {"control", delta > 0 ? "mouse-wheel-up" : "mouse-wheel-down"},
                               {"value", delta > 0 ? 1.0 : -1.0}});
        return false;
    }

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
