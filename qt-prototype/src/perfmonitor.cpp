#include "perfmonitor.h"
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <QScreen>
#include <QStandardPaths>
#include <QThread>
#ifdef Q_OS_WIN
#define NOMINMAX
#include <windows.h>
#include <psapi.h>
#endif

namespace { constexpr int HotkeyId = 0x434c; }

PerfMonitor::PerfMonitor(QObject *parent)
    : QObject(parent), m_settings("ConsoleLauncher", "PerformanceOverlay") {
    m_enabled = m_settings.value("enabled", true).toBool();
    m_key = qBound(1, m_settings.value("functionKey", 8).toInt(), 11);
    if (m_dir.isValid())
        qputenv("CONSOLE_LAUNCHER_LIVE_SESSION", QFile::encodeName(m_dir.filePath("session.json")));
    m_clock.start();
    m_timer.setInterval(500);
    connect(&m_timer, &QTimer::timeout, this, &PerfMonitor::poll);
    connect(&m_capture, &QProcess::readyReadStandardOutput, this, &PerfMonitor::readFrames);
    connect(&m_capture, &QProcess::readyReadStandardError, this, [this] {
        // Drain diagnostics without recording input or creating a growing log.
        m_capture.readAllStandardError();
    });
    connect(&m_capture, &QProcess::errorOccurred, this, [this](QProcess::ProcessError) {
        m_status = tr("Mesure indisponible : vérifier PresentMon et son autorisation dans les réglages.");
        Q_EMIT changed();
    });
    connect(&m_capture, qOverload<int, QProcess::ExitStatus>(&QProcess::finished), this,
            [this](int code, QProcess::ExitStatus) {
                if (m_active && m_enabled) {
                    m_status = code == 0 ? tr("Collecteur arrêté.")
                        : tr("Collecteur arrêté (%1) : vérifier l'autorisation de mesure.").arg(code);
                    Q_EMIT changed();
                }
            });

}

PerfMonitor::~PerfMonitor() {
    setActive(false);
    QCoreApplication::instance()->removeNativeEventFilter(this);
}

bool PerfMonitor::supported() const {
#ifdef Q_OS_WIN
    return true;
#else
    return false;
#endif
}

void PerfMonitor::configure(const QVariantMap &perf) {
    m_presentmon = perf.value("presentmon_path").toString();
}

void PerfMonitor::setEnabled(bool enabled) {
    if (enabled == m_enabled) return;
    m_enabled = enabled;
    m_settings.setValue("enabled", enabled);
    if (m_active && enabled && supported()) {
        QCoreApplication::instance()->installNativeEventFilter(this);
        m_attempted = false;
        if (m_pid) startCapture();
        m_timer.start();
    } else if (!enabled) {
        QCoreApplication::instance()->removeNativeEventFilter(this);
        m_timer.stop(); unregisterShortcut();
        m_requested = false; m_display = false;
        stopCapture();
    }
    Q_EMIT changed();
}
void PerfMonitor::setFunctionKey(int key) {
    if (key < 1 || key > 11 || key == m_key) return;
    unregisterShortcut();
    m_key = key;
    m_settings.setValue("functionKey", key);
    if (m_active && m_enabled) registerShortcut();
    Q_EMIT changed();
}

void PerfMonitor::setActive(bool active) {
    QCoreApplication::instance()->removeNativeEventFilter(this);
    m_active = active;
    m_timer.stop();
    unregisterShortcut();
    stopCapture();
#ifdef Q_OS_WIN
    if (m_process) CloseHandle(static_cast<HANDLE>(m_process));
#endif
    m_process = nullptr;
    m_pid = 0;
    m_previousCpu = 0; m_previousWall = 0;
    m_attempted = false;
    m_requested = false; m_display = false;
    m_metrics.clear(); m_frames.clear(); m_buffer.clear();
    QFile::remove(m_dir.filePath("session.json"));
    if (active && m_enabled && supported()) {
        QCoreApplication::instance()->installNativeEventFilter(this);
        m_status = tr("En attente du processus du jeu…");
        m_timer.start();
    }
    Q_EMIT changed();
}

void PerfMonitor::registerShortcut() {
#ifdef Q_OS_WIN
    m_registered = RegisterHotKey(nullptr, HotkeyId, MOD_NOREPEAT, VK_F1 + m_key - 1);
    if (!m_registered)
        m_status = tr("F%1 est déjà utilisée. Choisir une autre touche dans Performance.").arg(m_key);
#endif
}

void PerfMonitor::unregisterShortcut() {
#ifdef Q_OS_WIN
    if (m_registered) UnregisterHotKey(nullptr, HotkeyId);
#endif
    m_registered = false;
}

bool PerfMonitor::nativeEventFilter(const QByteArray &, void *message, qintptr *) {
#ifdef Q_OS_WIN
    const auto *msg = static_cast<MSG *>(message);
    if (msg->message == WM_HOTKEY && msg->wParam == HotkeyId && m_active && m_pid) {
        DWORD foreground = 0;
        GetWindowThreadProcessId(GetForegroundWindow(), &foreground);
        if (foreground == m_pid) { m_requested = !m_requested; poll(); }
        return true;
    }
#else
    Q_UNUSED(message)
#endif
    return false;
}

void PerfMonitor::poll() {
    if (!m_active || !m_enabled) return;
    if (!m_pid) {
        QFile file(m_dir.filePath("session.json"));
        if (!file.open(QIODevice::ReadOnly)) return;
        const auto doc = QJsonDocument::fromJson(file.read(8192));
        const auto object = doc.object();
        const auto id = object.value("pid").toInteger();
        if (id <= 0 || id > 0xffffffffLL) return;
        m_pid = quint32(id);
        m_metrics["emulator"] = QFileInfo(object.value("program").toString()).completeBaseName();
#ifdef Q_OS_WIN
        m_process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, FALSE, m_pid);
#endif
        startCapture();
    }
    const QString emulator = m_metrics.value("emulator").toString();
    m_metrics = m_frames.snapshot(m_clock.elapsed());
    m_metrics["emulator"] = emulator;
    m_metrics["shortcutAvailable"] = m_registered;
    sampleProcess();
#ifdef Q_OS_WIN
    const HWND window = GetForegroundWindow();
    DWORD foreground = 0;
    GetWindowThreadProcessId(window, &foreground);
    if (foreground == m_pid && !m_registered) registerShortcut();
    if (foreground != m_pid) unregisterShortcut();
    m_display = m_requested && foreground == m_pid;
    if (m_display) {
        MONITORINFOEXW monitor{};
        monitor.cbSize = sizeof(monitor);
        GetMonitorInfoW(MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST), &monitor);
        QScreen *screen = QGuiApplication::primaryScreen();
        for (auto *candidate : QGuiApplication::screens())
            if (candidate->name() == QString::fromWCharArray(monitor.szDevice)) screen = candidate;
        if (screen) {
            m_x = screen->availableGeometry().left() + 18;
            m_y = screen->availableGeometry().top() + 18;
        }
    }
#endif
    Q_EMIT changed();
}

void PerfMonitor::sampleProcess() {
#ifdef Q_OS_WIN
    if (!m_process) return;
    const HANDLE process = static_cast<HANDLE>(m_process);
    FILETIME creation{}, exit{}, kernel{}, user{};
    if (GetProcessTimes(process, &creation, &exit, &kernel, &user)) {
        const auto ticks = [](FILETIME t) { return (quint64(t.dwHighDateTime) << 32) | t.dwLowDateTime; };
        const quint64 cpu = ticks(kernel) + ticks(user);
        const qint64 now = m_clock.elapsed();
        if (m_previousWall && now > m_previousWall && cpu >= m_previousCpu)
            m_metrics["cpu"] = (cpu - m_previousCpu) / double(now - m_previousWall) / 100.0
                / qMax(1, QThread::idealThreadCount());
        m_previousCpu = cpu; m_previousWall = now;
    }
    PROCESS_MEMORY_COUNTERS memory{};
    memory.cb = sizeof(memory);
    if (GetProcessMemoryInfo(process, &memory, sizeof(memory)))
        m_metrics["ramMb"] = memory.WorkingSetSize / 1048576.0;
#endif
}

void PerfMonitor::startCapture() {
    if (m_attempted) return;
    m_attempted = true;
    QString exe = m_presentmon;
    if (!QFileInfo::exists(exe)) exe = QStandardPaths::findExecutable("PresentMon.exe");
    if (exe.isEmpty()) {
        const QString tools = qEnvironmentVariable("CONSOLE_LAUNCHER_TOOLS",
                              QCoreApplication::applicationDirPath() + "/tools");
        const QDir folder(tools + "/presentmon");
        const auto matches = folder.entryList({"PresentMon*.exe"}, QDir::Files, QDir::Name);
        if (!matches.isEmpty()) exe = folder.filePath(matches.last());
    }
    if (exe.isEmpty()) { m_status = tr("PresentMon absent : FPS non mesurés."); return; }
    m_capture.setProgram(exe);
    m_capture.setArguments({"--process_id", QString::number(m_pid), "--output_stdout", "--v1_metrics",
                           "--no_track_input", "--no_console_stats", "--terminate_on_proc_exit",
                           "--session_name", "ConsoleLauncherLive-" + QString::number(QCoreApplication::applicationPid())});
#ifdef Q_OS_WIN
    m_capture.setCreateProcessArgumentsModifier([](QProcess::CreateProcessArguments *args) {
        args->flags |= CREATE_NO_WINDOW;
    });
#endif
    m_status = tr("Collecte en cours — fenêtre glissante de 2 s.");
    m_capture.start();
}

void PerfMonitor::readFrames() {
    m_buffer += m_capture.readAllStandardOutput();
    if (m_buffer.size() > 262144) m_buffer = m_buffer.right(262144).mid(m_buffer.right(262144).indexOf('\n') + 1);
    qsizetype newline = 0;
    while ((newline = m_buffer.indexOf('\n')) >= 0) {
        const auto line = m_buffer.left(newline);
        m_buffer.remove(0, newline + 1);
        m_frames.ingest(line, m_clock.elapsed(), m_pid);
    }
}

void PerfMonitor::stopCapture() {
    if (m_capture.state() == QProcess::NotRunning) return;
    QProcess stop;
    stop.setProgram(m_capture.program());
    stop.setArguments({"--session_name", "ConsoleLauncherLive-" + QString::number(QCoreApplication::applicationPid()),
                       "--terminate_existing_session", "--no_console_stats"});
#ifdef Q_OS_WIN
    stop.setCreateProcessArgumentsModifier([](QProcess::CreateProcessArguments *args) { args->flags |= CREATE_NO_WINDOW; });
#endif
    stop.start();
    if (!stop.waitForFinished(1500)) { stop.kill(); stop.waitForFinished(500); }
    if (!m_capture.waitForFinished(500)) { m_capture.kill(); m_capture.waitForFinished(500); }
}