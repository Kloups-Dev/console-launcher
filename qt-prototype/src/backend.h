#pragma once

// Bridge between QML themes and the engine.
//
// The engine is the Rust command-line tool `clctl` (same engine as the
// Tauri prototype), run as a short-lived process for each request. This keeps
// the interface process small and lets both interfaces share one engine.

#include <functional>

#include <QObject>
#include <QProcess>
#include <QStringList>
#include <QVariant>

class QQmlApplicationEngine;
class QQuickWindow;

class Backend : public QObject {
    Q_OBJECT
    Q_PROPERTY(QVariantList systems READ systems NOTIFY libraryChanged)
    Q_PROPERTY(QVariantList games READ games NOTIFY libraryChanged)
    Q_PROPERTY(bool running READ running NOTIFY runningChanged)
    Q_PROPERTY(QString padName READ padName NOTIFY padChanged)
    Q_PROPERTY(QStringList themes READ themes CONSTANT)
    Q_PROPERTY(QString theme READ theme NOTIFY themeChanged)
    Q_PROPERTY(QString lastError READ lastError NOTIFY errorChanged)

public:
    explicit Backend(QObject *parent = nullptr);

    void setEngine(QQmlApplicationEngine *engine) { m_engine = engine; }
    void loadTheme();

    QVariantList systems() const { return m_systems; }
    QVariantList games() const { return m_games; }
    bool running() const { return m_running; }
    QString padName() const { return m_padName; }
    QStringList themes() const;
    QString theme() const { return m_theme; }
    QString lastError() const { return m_error; }

    /// Called by the gamepad and keyboard filters; forwarded to the theme.
    void emitNav(const QString &action);
    void setPadName(const QString &name);

    Q_INVOKABLE void reload();
    Q_INVOKABLE void requestInfo(const QString &gameId);
    Q_INVOKABLE void play(const QString &gameId);
    Q_INVOKABLE void toggleFavorite(const QString &gameId);
    Q_INVOKABLE void addRomDir(const QString &path);
    Q_INVOKABLE void setTheme(const QString &name);
    Q_INVOKABLE void toggleFullscreen();
    Q_INVOKABLE void quit();
    Q_INVOKABLE QString fileUrl(const QString &path) const;
    /// File inside the active theme folder, as a URL (images, fonts…).
    Q_INVOKABLE QString asset(const QString &relPath) const;

Q_SIGNALS:
    void libraryChanged();
    void runningChanged();
    void padChanged();
    void themeChanged();
    void errorChanged();
    void nav(const QString &action);
    void gameInfoReady(const QString &gameId, const QVariantMap &info);
    void favoriteChanged(const QString &gameId, bool on);
    void sessionEnded(const QVariantMap &report);
    /// Emitted when a session starts or ends, so the gamepad can be paused.
    void sessionActive(bool active);

private:
    QString clctlPath() const;
    QString themesDir() const;
    QQuickWindow *window() const;
    /// Runs clctl asynchronously; `done` receives stdout or an error message.
    void runClctl(const QStringList &args,
                  std::function<void(bool ok, const QByteArray &out)> done);
    void setError(const QString &e);

    QQmlApplicationEngine *m_engine = nullptr;
    QVariantList m_systems;
    QVariantList m_games;
    bool m_running = false;
    QString m_padName;
    QString m_theme;
    QString m_error;
};
