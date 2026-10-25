// Console Launcher — Qt Quick application.

#include <cstdio>

#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QGuiApplication>
#include <QMutex>
#include <QQmlApplicationEngine>
#include <QQmlError>
#include <QStandardPaths>
#include <QTimer>
#include <QtQml/qqml.h>

#include "backend.h"
#include "gamepad.h"
#include "keyfilter.h"

namespace {

QFile *g_log = nullptr;
QMutex g_logLock;

// Every Qt and QML message goes to the log file (and to stderr), so an
// interface failure on someone's machine can be diagnosed.
void logHandler(QtMsgType type, const QMessageLogContext &, const QString &msg) {
    static const char *names[] = {"debug", "warning", "critical", "fatal", "info"};
    const QString line = QStringLiteral("%1 [%2] %3\n")
                             .arg(QDateTime::currentDateTime().toString(Qt::ISODateWithMs),
                                  QString::fromLatin1(names[type]), msg);
    const QByteArray bytes = line.toUtf8();
    {
        QMutexLocker lock(&g_logLock);
        if (g_log) {
            g_log->write(bytes);
            g_log->flush();
        }
    }
    std::fputs(bytes.constData(), stderr);
}

/// `console-launcher.log` next to the executable, or in the temporary folder
/// when the program folder is read-only.
QString openLog() {
    const QStringList dirs = {QCoreApplication::applicationDirPath(),
                              QStandardPaths::writableLocation(QStandardPaths::TempLocation)};
    for (const QString &dir : dirs) {
        auto *f = new QFile(dir + "/console-launcher.log");
        if (f->open(QIODevice::WriteOnly | QIODevice::Truncate | QIODevice::Text)) {
            g_log = f;
            return QDir::toNativeSeparators(f->fileName());
        }
        delete f;
    }
    return QString();
}

/// Shown when the built-in interface cannot be loaded, instead of an
/// invisible program.
void showLoadError(QQmlApplicationEngine &engine, const QStringList &errors, const QString &logPath) {
    QString text = errors.join("\n");
    text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    QString log = logPath;
    log.replace('\\', "\\\\");
    const QByteArray qml = QStringLiteral(R"(
import QtQuick
import QtQuick.Window
Window {
    visible: true; width: 900; height: 520; color: "#151a24"
    title: "Console Launcher"
    Column {
        x: 28; y: 24; width: parent.width - 56; spacing: 12
        Text { text: "L'interface n'a pas pu être chargée · The interface could not be loaded"; color: "#ffc440"; font.pixelSize: 20; font.bold: true }
        Text { width: parent.width; wrapMode: Text.Wrap; color: "#e8ecf4"; font.pixelSize: 13; text: "%1" }
        Text { width: parent.width; wrapMode: Text.Wrap; color: "#8a93a6"; font.pixelSize: 13; text: "Journal / Log: %2" }
    }
})").arg(text, log).toUtf8();
    engine.loadData(qml);
}

} // namespace

int main(int argc, char *argv[]) {
    QGuiApplication app(argc, argv);
    app.setOrganizationName("ConsoleLauncher");
    app.setApplicationName("console-launcher-qt");
    const bool smokeTest = app.arguments().contains("--smoke-test");

    const QString logPath = openLog();
    qInstallMessageHandler(logHandler);
    qInfo("Console Launcher %s, Qt %s, %s", CL_VERSION, qVersion(),
          qPrintable(QSysInfo::prettyProductName()));
    // The engine writes its own log next to this one.
    if (!logPath.isEmpty())
        qputenv("CONSOLE_LAUNCHER_ENGINE_LOG",
                QFileInfo(logPath).dir().filePath("console-launcher-engine.log").toUtf8());

    qInfo("opening the engine");
    Backend backend;
    qInfo("engine ready");

    KeyFilter keys;
    app.installEventFilter(&keys);
    QObject::connect(&keys, &KeyFilter::nav, &backend, &Backend::emitNav);
    QObject::connect(&keys, &KeyFilter::fullscreen, &backend, &Backend::toggleFullscreen);
    QObject::connect(&keys, &KeyFilter::inputEvent, &backend, &Backend::emitControlInput);

    QQmlApplicationEngine engine;
    QStringList qmlErrors;
    QObject::connect(&engine, &QQmlApplicationEngine::warnings, &app,
                     [&qmlErrors](const QList<QQmlError> &list) {
                         for (const QQmlError &e : list)
                             qmlErrors << e.toString();
                     });
    // A registered singleton is visible only to the trusted, compiled QML
    // module. Context properties are intentionally avoided: they are invisible
    // to static tooling and would become a broad ambient API.
    qmlRegisterUncreatableType<PerfMonitor>("ConsoleLauncher", 1, 0, "PerformanceMonitor", "Owned by the launcher");
    qmlRegisterSingletonInstance("ConsoleLauncher", 1, 0, "Launcher", &backend);
    backend.setQmlEngine(&engine);
    qInfo("loading built-in interface");
    backend.loadInterface();
    const bool loaded = !engine.rootObjects().isEmpty();
    if (loaded) {
        qInfo("interface loaded");
    } else {
        qCritical("built-in interface failed to load");
        for (const QString &e : qmlErrors)
            qCritical("%s", qPrintable(e));
        if (!smokeTest)
            showLoadError(engine, qmlErrors.isEmpty() ? QStringList{backend.lastError()} : qmlErrors,
                          logPath);
    }

    // Controllers start after the window, on their own thread: a slow or
    // stuck device never keeps the interface from appearing.
    Gamepad pad;
    QObject::connect(&pad, &Gamepad::nav, &backend, &Backend::emitNav);
    QObject::connect(&pad, &Gamepad::padName, &backend, &Backend::setPadName);
    QObject::connect(&pad, &Gamepad::inputEvent, &backend, &Backend::emitControlInput);
    QObject::connect(&backend, &Backend::sessionActive, &pad, &Gamepad::setPaused);
    pad.start();

    backend.reload();

    if (smokeTest) {
        // CI check: the interface and its settings hierarchy must load, and
        // the library request must answer.
        QObject::connect(&backend, &Backend::libraryChanged, &app, [&] {
            qInfo("library answered: %lld systems", static_cast<long long>(backend.systems().size()));
        });
        QTimer::singleShot(250, &backend, [&backend] { backend.openSettings(); });
        QTimer::singleShot(4000, &app, [&app, loaded] {
            qInfo("smoke test %s", loaded ? "passed" : "FAILED");
            app.exit(loaded ? 0 : 1);
        });
    }
    return app.exec();
}
