#pragma once

// Keyboard → navigation actions, so themes only handle `Launcher.nav`.

#include <QObject>
#include <QVariantMap>

class KeyFilter : public QObject {
    Q_OBJECT
public:
    using QObject::QObject;

Q_SIGNALS:
    void nav(const QString &action);
    void fullscreen();
    void inputEvent(const QVariantMap &event);

protected:
    bool eventFilter(QObject *watched, QEvent *event) override;
};
