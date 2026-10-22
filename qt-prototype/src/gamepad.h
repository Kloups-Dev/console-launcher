#pragma once

// Controller input through SDL2's GameController API (its community
// database covers Xbox, PlayStation, Switch and most generic pads).
// Buttons are turned into the same abstract actions as the Tauri prototype:
// up, down, left, right, confirm, back, menu, favorite, pageLeft, pageRight.

#include <QElapsedTimer>
#include <QObject>
#include <QTimer>

class Gamepad : public QObject {
    Q_OBJECT
public:
    explicit Gamepad(QObject *parent = nullptr);
    ~Gamepad() override;

    /// While a game runs, SDL is shut down completely: no polling, no threads.
    void setPaused(bool paused);

Q_SIGNALS:
    void nav(const QString &action);
    void padName(const QString &name);

private:
    void start();
    void stop();
    void poll();
    void press(const QString &dir);

    QTimer m_timer;
    bool m_started = false;
    QString m_held;
    QElapsedTimer m_heldSince;
    qint64 m_lastRepeat = 0;
    float m_x = 0, m_y = 0;
};
