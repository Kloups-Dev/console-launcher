#include "keyfilter.h"

#include <QGuiApplication>
#include <QKeyEvent>
#include <QWindow>

bool KeyFilter::eventFilter(QObject *watched, QEvent *event) {
    if (event->type() != QEvent::KeyPress || !qobject_cast<QWindow *>(watched))
        return false;
    // Text fields in themes keep their keys (except Escape, handled by QML).
    QObject *focus = QGuiApplication::focusObject();
    if (focus && focus->inherits("QQuickTextInput"))
        return false;

    const auto *k = static_cast<QKeyEvent *>(event);
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
