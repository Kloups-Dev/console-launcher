#include "backend.h"

#include <QCoreApplication>
#include <QDir>
#include <QFileInfo>
#include <QGuiApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QSettings>
#include <QUrl>

Backend::Backend(QObject *parent) : QObject(parent) {
    QSettings s;
    m_theme = s.value("theme", "horizon").toString();
    if (!themes().contains(m_theme))
        m_theme = "horizon";
}

QString Backend::clctlPath() const {
    const QByteArray env = qgetenv("CLCTL");
    if (!env.isEmpty())
        return QString::fromLocal8Bit(env);
#ifdef Q_OS_WIN
    return QCoreApplication::applicationDirPath() + "/clctl.exe";
#else
    return QCoreApplication::applicationDirPath() + "/clctl";
#endif
}

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
    if (!m_engine || m_engine->rootObjects().isEmpty())
        return nullptr;
    return qobject_cast<QQuickWindow *>(m_engine->rootObjects().first());
}

void Backend::loadTheme() {
    if (!m_engine)
        return;
    // Drop the previous theme's window before loading the new one.
    const auto roots = m_engine->rootObjects();
    for (QObject *o : roots)
        o->deleteLater();
    m_engine->clearComponentCache();
    const QString file = themesDir() + "/" + m_theme + "/Main.qml";
    m_engine->load(QUrl::fromLocalFile(file));
    if (m_engine->rootObjects().isEmpty())
        setError(tr("Theme '%1' could not be loaded").arg(m_theme));
    Q_EMIT themeChanged();
}

void Backend::setTheme(const QString &name) {
    if (!themes().contains(name) || name == m_theme)
        return;
    m_theme = name;
    QSettings().setValue("theme", name);
    QMetaObject::invokeMethod(this, [this] { loadTheme(); }, Qt::QueuedConnection);
}

void Backend::setError(const QString &e) {
    m_error = e;
    Q_EMIT errorChanged();
}

void Backend::runClctl(const QStringList &args,
                       std::function<void(bool, const QByteArray &)> done) {
    auto *p = new QProcess(this);
    p->setProgram(clctlPath());
    p->setArguments(args);
    p->setProcessChannelMode(QProcess::SeparateChannels);
    connect(p, &QProcess::finished, this, [p, done](int code, QProcess::ExitStatus st) {
        const bool ok = st == QProcess::NormalExit && code == 0;
        done(ok, ok ? p->readAllStandardOutput() : p->readAllStandardError().trimmed());
        p->deleteLater();
    });
    connect(p, &QProcess::errorOccurred, this, [this, p, done](QProcess::ProcessError e) {
        if (e == QProcess::FailedToStart) {
            done(false, QStringLiteral("clctl not found: %1").arg(p->program()).toUtf8());
            p->deleteLater();
        }
    });
    p->start();
}

void Backend::reload() {
    runClctl({"--json", "scan"}, [this](bool ok, const QByteArray &out) {
        if (!ok) {
            setError(QString::fromUtf8(out));
            return;
        }
        // Output: [systems, games]
        const QJsonArray pair = QJsonDocument::fromJson(out).array();
        m_systems = pair.at(0).toArray().toVariantList();
        m_games = pair.at(1).toArray().toVariantList();
        setError(QString());
        Q_EMIT libraryChanged();
    });
}

void Backend::requestInfo(const QString &gameId) {
    runClctl({"info", gameId}, [this, gameId](bool ok, const QByteArray &out) {
        QVariantMap info;
        if (ok)
            info = QJsonDocument::fromJson(out).object().toVariantMap();
        else
            info = {{"ready", false}, {"error", QString::fromUtf8(out)}};
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
    runClctl({"--json", "play", gameId}, [this](bool ok, const QByteArray &out) {
        QVariantMap report;
        if (ok)
            report = QJsonDocument::fromJson(out).object().toVariantMap();
        else
            report = {{"error", QString::fromUtf8(out)}};
        m_running = false;
        Q_EMIT runningChanged();
        Q_EMIT sessionActive(false);
        if (auto *w = window()) {
            w->show();
            w->raise();
            w->requestActivate();
        }
        Q_EMIT sessionEnded(report);
    });
}

void Backend::toggleFavorite(const QString &gameId) {
    runClctl({"--json", "favorite", gameId}, [this, gameId](bool ok, const QByteArray &out) {
        if (!ok)
            return;
        const bool on = out.trimmed() == "true";
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
    runClctl({"add-roms", path}, [this](bool ok, const QByteArray &out) {
        if (!ok)
            setError(QString::fromUtf8(out));
        reload();
    });
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
    if (!m_running)
        Q_EMIT nav(action);
}

void Backend::setPadName(const QString &name) {
    if (name == m_padName)
        return;
    m_padName = name;
    Q_EMIT padChanged();
}
