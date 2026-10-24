#pragma once

// Keyboard → navigation actions, so themes only handle `Launcher.nav`.

#include <QObject>

class KeyFilter : public QObject {
    Q_OBJECT
public:
    using QObject::QObject;

Q_SIGNALS:
    void nav(const QString &action);
    void fullscreen();

protected:
    bool eventFilter(QObject *watched, QEvent *event) override;
};
