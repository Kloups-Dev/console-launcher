#include "backend.h"

#include <QCoreApplication>
#include <QDir>
#include <QJSEngine>
#include <QPointer>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QSet>
#include <QThreadPool>
#include <QUrl>

Backend::Backend(QObject *parent) : QObject(parent) {
    connect(this, &Backend::sessionActive, &m_performance, &PerfMonitor::setActive);
    // Settings are read once, synchronously, before the interface is loaded.
    const Engine::Answer a = m_engine.call("config");
    if (a.ok)
        applyConfig(a.result.toMap());
    else
        setError(a.error);
}

void Backend::applyConfig(const QVariantMap &config) {
    m_performance.configure(config.value("perf").toMap());
    const QString lang = config.value("language").toString();
    if (!lang.isEmpty() && lang != m_language) {
        m_language = lang;
        Q_EMIT languageChanged();
    }
}

QString Backend::version() const { return QStringLiteral(CL_VERSION); }

QQuickWindow *Backend::window() const {
    if (!m_qml || m_qml->rootObjects().isEmpty())
        return nullptr;
    return qobject_cast<QQuickWindow *>(m_qml->rootObjects().first());
}

void Backend::loadInterface() {
    if (!m_qml)
        return;
    m_settingsOpen = false;
    qInfo("loading desktop component");
    m_qml->loadFromModule("ConsoleLauncher", "DesktopApp");
    qInfo("desktop component returned");
    if (!m_qml->rootObjects().isEmpty() && m_performance.supported()) {
        qInfo("loading performance component");
        m_qml->loadFromModule("ConsoleLauncher", "PerformanceOverlay");
        qInfo("performance component returned");
    }

    if (m_qml->rootObjects().isEmpty())
        setError(tr("The built-in interface could not be loaded"));
}

void Backend::setError(const QString &e) {
    if (e == m_error)
        return;
    m_error = e;
    Q_EMIT errorChanged();
}

void Backend::async(const QString &method, const QVariant &params,
                    std::function<void(const Engine::Answer &)> done) {
    QPointer<Backend> self(this);
    const Engine *engine = &m_engine;
    QThreadPool::globalInstance()->start([self, engine, method, params, done] {
        Engine::Answer a = engine->call(method, params);
        if (!self)
            return;
        QMetaObject::invokeMethod(
            self.data(), [self, a, done] { if (self) done(a); }, Qt::QueuedConnection);
    });
}

void Backend::internalRequest(const QString &method, const QJSValue &params,
                              const QJSValue &callback) {
    static const QSet<QString> allowedMethods{
        QStringLiteral("add_game"),
        QStringLiteral("add_path"),
        QStringLiteral("add_custom_emulator"),
        QStringLiteral("allow_capture"),
        QStringLiteral("hardware"),
        QStringLiteral("install_emulator"),
        QStringLiteral("open_emulator"),
        QStringLiteral("remove_custom_emulator"),
        QStringLiteral("remove_game"),
        QStringLiteral("remove_path"),
        QStringLiteral("refresh_game_metadata"),
        QStringLiteral("reset_emulator_tuning"),
        QStringLiteral("set_emulator_path"),
        QStringLiteral("set_system_emulator"),
        QStringLiteral("settings_overview"),
        QStringLiteral("update_custom_emulator"),
        QStringLiteral("update_preferences"),
    };
    if (!allowedMethods.contains(method)) {
        const QString error = tr("Interface request blocked: %1").arg(method);
        qWarning("%s", qPrintable(error));
        if (callback.isCallable())
            callback.call({QJSValue(false), QJSValue(error)});
        return;
    }
    const QVariant p = params.isUndefined() || params.isNull() ? QVariant() : params.toVariant();
    QJSValue cb = callback;
    async(method, p, [this, method, cb](const Engine::Answer &a) mutable {
        // Settings that change what the theme shows.
        if (a.ok && method == "settings_overview")
            applyConfig(a.result.toMap().value("config").toMap());
        else if (a.ok && (method == "update_preferences" || method == "reload")) {
            const Engine::Answer c = m_engine.call("config");
            if (c.ok)
                applyConfig(c.result.toMap());
        }
        if (cb.isCallable()) {
            QJSEngine *js = qjsEngine(this);
            if (!js && m_qml)
                js = m_qml;
            if (js) {
                const QJSValue r = cb.call({QJSValue(a.ok), a.ok ? js->toScriptValue(a.result) : QJSValue(a.error)});
                if (r.isError())
                    qWarning("%s: %s (%s)", qPrintable(method), qPrintable(r.toString()),
                             qPrintable(r.property("stack").toString()));
            }
        }
    });
}

QString Backend::localPath(const QUrl &url) const {
    return url.isLocalFile() ? QDir::toNativeSeparators(url.toLocalFile()) : url.toString();
}

void Backend::reload() {
    ++m_libraryRequestId;
    if (m_libraryReloadScheduled || m_libraryReloadInFlight) {
        m_libraryReloadAgain = true;
        return;
    }
    m_libraryReloadScheduled = true;
    QMetaObject::invokeMethod(
        this,
        [this] {
            m_libraryReloadScheduled = false;
            m_libraryReloadInFlight = true;
            const std::uint64_t requestId = m_libraryRequestId;
            async("library", QVariant(), [this, requestId](const Engine::Answer &a) {
                m_libraryReloadInFlight = false;
                if (m_libraryReloadAgain) {
                    m_libraryReloadAgain = false;
                    reload();
                    return;
                }
                if (requestId != m_libraryRequestId)
                    return;
                if (!a.ok) {
                    setError(a.error);
                    return;
                }
                const QVariantMap lib = a.result.toMap();
                m_systems = lib.value("systems").toList();
                m_games = lib.value("games").toList();
                setError(QString());
                Q_EMIT libraryChanged();
            });
        },
        Qt::QueuedConnection);
}

void Backend::requestInfo(const QString &gameId) {
    async("game_info", QVariantMap{{"id", gameId}}, [this, gameId](const Engine::Answer &a) {
        QVariantMap info = a.ok ? a.result.toMap()
                                : QVariantMap{{"ready", false}, {"error", a.error}};
        Q_EMIT gameInfoReady(gameId, info);
    });
}

void Backend::play(const QString &gameId) {
    if (m_running)
        return;
    m_running = true;
    Q_EMIT runningChanged();
    Q_EMIT sessionActive(true);
    // Hidden window: Qt Quick stops rendering, so the launcher uses
    // (almost) nothing while the game runs.
    if (auto *w = window())
        w->hide();
    async("play", QVariantMap{{"id", gameId}}, [this](const Engine::Answer &a) {
        QVariantMap report = a.ok ? a.result.toMap() : QVariantMap{{"error", a.error}};
        m_running = false;
        Q_EMIT runningChanged();
        Q_EMIT sessionActive(false);
        if (auto *w = window()) {
            w->show();
            w->raise();
            w->requestActivate();
        }
        if (!a.ok)
            setError(a.error);
        Q_EMIT sessionEnded(report);
    });
}

void Backend::toggleFavorite(const QString &gameId) {
    async("toggle_favorite", QVariantMap{{"id", gameId}}, [this, gameId](const Engine::Answer &a) {
        if (!a.ok)
            return;
        const bool on = a.result.toBool();
        for (QVariant &g : m_games) {
            QVariantMap m = g.toMap();
            if (m.value("id").toString() == gameId) {
                m["favorite"] = on;
                g = m;
            }
        }
        Q_EMIT favoriteChanged(gameId, on);
    });
}

void Backend::addRomDir(const QString &path) {
    async("add_path", QVariantMap{{"path", path}}, [this](const Engine::Answer &a) {
        if (!a.ok)
            setError(a.error);
        reload();
    });
}

void Backend::openSettings(const QVariant &target) {
    QVariant t = target;
    if (t.userType() == qMetaTypeId<QJSValue>())
        t = t.value<QJSValue>().toVariant();
    Q_EMIT settingsRequested(t);
}

void Backend::setSettingsOpen(bool open) {
    if (open == m_settingsOpen)
        return;
    m_settingsOpen = open;
    Q_EMIT settingsOpenChanged();
}

void Backend::toggleFullscreen() {
    if (auto *w = window())
        w->setVisibility(w->visibility() == QWindow::FullScreen ? QWindow::Windowed
                                                                 : QWindow::FullScreen);
}

void Backend::quit() { QCoreApplication::quit(); }

void Backend::emitNav(const QString &action) {
    if (m_running || m_controlCapture)
        return;
    if (m_settingsOpen)
        Q_EMIT settingsNav(action);
    else
        Q_EMIT nav(action);
}

void Backend::setControlCapture(bool capture) {
    if (capture == m_controlCapture)
        return;
    m_controlCapture = capture;
    Q_EMIT controlCaptureChanged();
}

void Backend::emitControlInput(const QVariantMap &event) {
    if (!m_running && m_controlCapture)
        Q_EMIT controlInput(event);
}

void Backend::setPadName(const QString &name) {
    if (name == m_padName)
        return;
    m_padName = name;
    Q_EMIT padChanged();
}
