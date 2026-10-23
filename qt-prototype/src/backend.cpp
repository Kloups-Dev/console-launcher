#include "backend.h"

#include <QCoreApplication>
#include <QDir>
#include <QFileInfo>
#include <QGuiApplication>
#include <QJSEngine>
#include <QPointer>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QThreadPool>
#include <QUrl>

Backend::Backend(QObject *parent) : QObject(parent) {
    // Settings are read once, synchronously, before the theme is loaded.
    const Engine::Answer a = m_engine.call("config");
    if (a.ok)
        applyConfig(a.result.toMap());
    else
        setError(a.error);
    if (!themes().contains(m_theme))
        m_theme = QStringLiteral("horizon");
}

void Backend::applyConfig(const QVariantMap &config) {
    const QString lang = config.value("language").toString();
    if (!lang.isEmpty() && lang != m_language) {
        m_language = lang;
        Q_EMIT languageChanged();
    }
    const QString theme = config.value("theme").toString();
    if (!theme.isEmpty())
        m_theme = theme;
}

QString Backend::version() const { return QStringLiteral(CL_VERSION); }

QString Backend::themesDir() const {
    const QByteArray env = qgetenv("CONSOLE_LAUNCHER_QT_THEMES");
    if (!env.isEmpty())
        return QString::fromLocal8Bit(env);
    return QCoreApplication::applicationDirPath() + "/themes";
}

QStringList Backend::themes() const {
    QStringList out;
    const auto dirs = QDir(themesDir()).entryList(QDir::Dirs | QDir::NoDotAndDotDot, QDir::Name);
    for (const QString &d : dirs)
        if (QFileInfo::exists(themesDir() + "/" + d + "/Main.qml"))
            out << d;
    return out;
}

QQuickWindow *Backend::window() const {
    if (!m_qml || m_qml->rootObjects().isEmpty())
        return nullptr;
    return qobject_cast<QQuickWindow *>(m_qml->rootObjects().first());
}

void Backend::loadTheme() {
    if (!m_qml)
        return;
    // Drop the previous theme's window before loading the new one.
    const auto roots = m_qml->rootObjects();
    for (QObject *o : roots)
        o->deleteLater();
    m_qml->clearComponentCache();
    m_settingsOpen = false;
    const QString file = themesDir() + "/" + m_theme + "/Main.qml";
    m_qml->load(QUrl::fromLocalFile(file));
    if (m_qml->rootObjects().isEmpty())
        setError(tr("Theme '%1' could not be loaded").arg(m_theme));
    Q_EMIT themeChanged();
}

void Backend::setTheme(const QString &name) {
    if (!themes().contains(name) || name == m_theme)
        return;
    m_theme = name;
    async("update_preferences", QVariantMap{{"theme", name}}, [](const Engine::Answer &) {});
    QMetaObject::invokeMethod(this, [this] { loadTheme(); }, Qt::QueuedConnection);
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

void Backend::request(const QString &method, const QJSValue &params, const QJSValue &callback) {
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
    async("library", QVariant(), [this](const Engine::Answer &a) {
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

QString Backend::fileUrl(const QString &path) const {
    return path.isEmpty() ? QString() : QUrl::fromLocalFile(path).toString();
}

QString Backend::asset(const QString &relPath) const {
    return QUrl::fromLocalFile(themesDir() + "/" + m_theme + "/" + relPath).toString();
}

void Backend::emitNav(const QString &action) {
    if (m_running)
        return;
    if (m_settingsOpen)
        Q_EMIT settingsNav(action);
    else
        Q_EMIT nav(action);
}

void Backend::setPadName(const QString &name) {
    if (name == m_padName)
        return;
    m_padName = name;
    Q_EMIT padChanged();
}
