#pragma once

// Private bridge between the built-in QML interface and the engine.
//
// The Rust engine is linked into the application (see engine.h). Calls run
// on a worker thread so the interface never waits; answers come back on the
// interface thread, as signals or as JavaScript callbacks.

#include <functional>
#include <cstdint>

#include <QJSValue>
#include <QObject>
#include <QVariant>

#include "engine.h"

class QQmlApplicationEngine;
class QQuickWindow;

class Backend : public QObject {
    Q_OBJECT
    Q_PROPERTY(QVariantList systems READ systems NOTIFY libraryChanged)
    Q_PROPERTY(QVariantList games READ games NOTIFY libraryChanged)
    Q_PROPERTY(bool running READ running NOTIFY runningChanged)
    Q_PROPERTY(QString padName READ padName NOTIFY padChanged)
    Q_PROPERTY(QString language READ language NOTIFY languageChanged)
    Q_PROPERTY(QString lastError READ lastError NOTIFY errorChanged)
    Q_PROPERTY(bool settingsOpen READ settingsOpen WRITE setSettingsOpen NOTIFY settingsOpenChanged)
    Q_PROPERTY(QString version READ version CONSTANT)

public:
    explicit Backend(QObject *parent = nullptr);

    void setQmlEngine(QQmlApplicationEngine *engine) { m_qml = engine; }
    /// Loads the interface compiled into the executable. External QML is
    /// deliberately never executed: future user themes are data-only packs.
    void loadInterface();

    QVariantList systems() const { return m_systems; }
    QVariantList games() const { return m_games; }
    bool running() const { return m_running; }
    QString padName() const { return m_padName; }
    QString language() const { return m_language; }
    QString lastError() const { return m_error; }
    bool settingsOpen() const { return m_settingsOpen; }
    void setSettingsOpen(bool open);
    QString version() const;

    /// Called by the gamepad and keyboard filters; forwarded to the interface,
    /// or to the settings panel while it is open.
    void emitNav(const QString &action);
    void setPadName(const QString &name);

    Q_INVOKABLE void reload();
    Q_INVOKABLE void requestInfo(const QString &gameId);
    Q_INVOKABLE void play(const QString &gameId);
    Q_INVOKABLE void toggleFavorite(const QString &gameId);
    Q_INVOKABLE void addRomDir(const QString &path);
    Q_INVOKABLE void toggleFullscreen();
    Q_INVOKABLE void quit();
    /// Opens the built-in settings panel. `target` (optional):
    /// {system: id}, {emulator: id}, or {page: "general"|"controls"|…}.
    Q_INVOKABLE void openSettings(const QVariant &target = QVariant());
    /// Trusted built-in UI plumbing. This is intentionally not part of the
    /// future theme contract. The callback receives (ok, resultOrError).
    Q_INVOKABLE void internalRequest(const QString &method,
                                     const QJSValue &params = QJSValue(),
                                     const QJSValue &callback = QJSValue());
    /// Local path from a "file:///…" URL given by a file dialog.
    Q_INVOKABLE QString localPath(const QUrl &url) const;

Q_SIGNALS:
    void libraryChanged();
    void runningChanged();
    void padChanged();
    void languageChanged();
    void errorChanged();
    void settingsOpenChanged();
    void nav(const QString &action);
    void settingsNav(const QString &action);
    void settingsRequested(const QVariant &target);
    void gameInfoReady(const QString &gameId, const QVariantMap &info);
    void favoriteChanged(const QString &gameId, bool on);
    void sessionEnded(const QVariantMap &report);
    /// Emitted when a session starts or ends, so the gamepad can be paused.
    void sessionActive(bool active);

private:
    QQuickWindow *window() const;
    /// Runs an engine call on a worker thread; `done` runs on this thread.
    void async(const QString &method, const QVariant &params,
               std::function<void(const Engine::Answer &)> done);
    void setError(const QString &e);
    void applyConfig(const QVariantMap &config);

    Engine m_engine;
    QQmlApplicationEngine *m_qml = nullptr;
    QVariantList m_systems;
    QVariantList m_games;
    bool m_running = false;
    bool m_settingsOpen = false;
    QString m_padName;
    QString m_language = QStringLiteral("fr");
    QString m_error;
    std::uint64_t m_libraryRequestId = 0;
};
