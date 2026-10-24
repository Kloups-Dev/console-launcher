#pragma once

// Controller input through SDL2's GameController API (its community
// database covers Xbox, PlayStation, Switch and most generic pads).
// Buttons are turned into the same abstract actions as the Tauri prototype:
// up, down, left, right, confirm, back, menu, favorite, pageLeft, pageRight.

#include <QElapsedTimer>
#include <QObject>
#include <QThread>
#include <QTimer>

/// Runs on its own thread (see Gamepad).
class GamepadWorker : public QObject {
    Q_OBJECT
public:
    void start();
    void stop();
    void setPaused(bool paused);

Q_SIGNALS:
    void nav(const QString &action);
    void padName(const QString &name);

private:
    void poll();
    void press(const QString &dir);

    QTimer *m_timer = nullptr;
    bool m_started = false;
    QString m_held;
    QElapsedTimer m_heldSince;
    qint64 m_lastRepeat = 0;
    float m_x = 0, m_y = 0;
};

/// Controller input. SDL runs on a dedicated thread, so a slow or stuck
/// device driver never blocks the interface.
class Gamepad : public QObject {
    Q_OBJECT
public:
    explicit Gamepad(QObject *parent = nullptr);
    ~Gamepad() override;

    void start();
    /// While a game runs, SDL is shut down completely: no polling.
    void setPaused(bool paused);

Q_SIGNALS:
    void nav(const QString &action);
    void padName(const QString &name);

private:
    QThread m_thread;
    GamepadWorker *m_worker = nullptr;
};
