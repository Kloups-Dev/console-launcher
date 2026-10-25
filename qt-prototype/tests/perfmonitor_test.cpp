#include "perfmonitor.h"
#include <QGuiApplication>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QTimer>
#include <QWindow>
#include <cstdio>
#ifdef Q_OS_WIN
#define NOMINMAX
#include <windows.h>
#endif

int main(int argc, char **argv) {
    QGuiApplication app(argc, argv);
    app.setQuitOnLastWindowClosed(false);
    PerfMonitor monitor;
    const bool previousEnabled = monitor.enabled();
    monitor.setEnabled(true);
    monitor.setActive(true);
    // Reproduce native-window creation while the session filter is installed.
    QWindow game;
    game.resize(480, 240);
    game.setTitle("Console Launcher — test automatique du diagnostic");
    game.show();
    game.requestActivate();
    QFile manifest(qEnvironmentVariable("CONSOLE_LAUNCHER_LIVE_SESSION"));
    if (!manifest.open(QIODevice::WriteOnly)) return 2;
    manifest.write(QJsonDocument(QJsonObject{
        {"pid", qint64(QCoreApplication::applicationPid())},
        {"program", QCoreApplication::applicationFilePath()}
    }).toJson());
    manifest.close();
    int failed = 0;
    QTimer::singleShot(2200, &app, [&] {
        if (!monitor.metrics().contains("cpu")) { std::fprintf(stderr, "FAIL: process CPU missing\n"); ++failed; }
#ifdef Q_OS_WIN
        DWORD foreground = 0;
        GetWindowThreadProcessId(GetForegroundWindow(), &foreground);
        if (foreground == GetCurrentProcessId()) {
            MSG message{}; message.message = WM_HOTKEY; message.wParam = 0x434c;
            monitor.nativeEventFilter("windows_generic_MSG", &message, nullptr);
            if (!monitor.display() || monitor.viewMode() != 2) { std::fprintf(stderr, "FAIL: shortcut did not show detailed panel\n"); ++failed; }
            monitor.nativeEventFilter("windows_generic_MSG", &message, nullptr);
            if (monitor.display()) { std::fprintf(stderr, "FAIL: shortcut did not hide panel\n"); ++failed; }
            monitor.nativeEventFilter("windows_generic_MSG", &message, nullptr);
            if (!monitor.display() || monitor.viewMode() != 1) { std::fprintf(stderr, "FAIL: shortcut did not restore compact panel\n"); ++failed; }
        } else std::printf("Foreground shortcut check skipped: another window is active.\n");
#endif
        monitor.setActive(false);
        if (monitor.display() || !monitor.metrics().isEmpty() || manifest.exists()) {
            std::fprintf(stderr, "FAIL: session state was not cleared\n"); ++failed;
        }
        monitor.setEnabled(previousEnabled);
        std::printf("Performance monitor lifecycle: %s\n", failed ? "FAILED" : "passed");
        app.exit(failed ? 1 : 0);
    });
    QTimer::singleShot(10000, &app, [&] { app.exit(3); });
    return app.exec();
}
