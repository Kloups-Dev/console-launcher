#pragma once

// Thin C++ wrapper over the Rust engine library (console_launcher_engine.h).
// Thread-safe: the engine serialises calls itself, except "play", which runs
// alongside the others.

#include <QJsonValue>
#include <QString>
#include <QVariant>

struct ClEngine;

class Engine {
public:
    struct Answer {
        bool ok = false;
        QVariant result;   // when ok
        QString error;     // when not ok
    };

    /// `dataDir` empty: the standard per-user folder.
    explicit Engine(const QString &dataDir = QString());
    ~Engine();
    Engine(const Engine &) = delete;
    Engine &operator=(const Engine &) = delete;

    bool isOpen() const { return m_handle != nullptr; }
    Answer call(const QString &method, const QVariant &params = QVariant()) const;

private:
    ClEngine *m_handle = nullptr;
};
