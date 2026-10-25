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
#include <QDateTime>
#ifdef Q_OS_WIN
#define NOMINMAX
#include <windows.h>
#include <psapi.h>
#elif defined(Q_OS_LINUX)
#include <unistd.h>
#endif

namespace { constexpr int HotkeyId = 0x434c, FallbackHotkeyId = 0x434d; }

PerfMonitor::PerfMonitor(QObject *parent)
    : QObject(parent), m_settings("ConsoleLauncher", "PerformanceOverlay") {
    m_enabled = m_settings.value("enabled", true).toBool();
    m_key = qBound(1, m_settings.value("functionKey", 8).toInt(), 11);
    if (m_dir.isValid())
        qputenv("CONSOLE_LAUNCHER_LIVE_SESSION", QFile::encodeName(m_dir.filePath("session.json")));
#ifdef Q_OS_WIN
    // The Rust DLL reads the Win32 environment, not necessarily Qt's CRT copy.
    if (m_dir.isValid())
        SetEnvironmentVariableW(L"CONSOLE_LAUNCHER_LIVE_SESSION",
            reinterpret_cast<LPCWSTR>(m_dir.filePath("session.json").utf16()));
#endif
    m_clock.start();
    m_timer.setInterval(500);
    connect(&m_timer, &QTimer::timeout, this, &PerfMonitor::poll);
    connect(&m_capture, &QProcess::readyReadStandardOutput, this, &PerfMonitor::readFrames);
    connect(&m_capture, &QProcess::readyReadStandardError, this, [this] {
        m_collectorDiagnostic = (m_collectorDiagnostic + QString::fromUtf8(m_capture.readAllStandardError())).right(1600);
        const QString diagnostic = m_collectorDiagnostic.trimmed();
        const bool standardUserNotice = diagnostic.startsWith("warning: PresentMon requires elevated privilege")
            && !diagnostic.contains("error:", Qt::CaseInsensitive)
            && !diagnostic.contains("failed", Qt::CaseInsensitive);
        // This notice only limits queries for other accounts/short-lived
        // processes. It does not mean that the PID-based trace was denied.
        if (!diagnostic.isEmpty() && !standardUserNotice)
            m_metrics["collectorError"] = diagnostic.right(600);
        Q_EMIT changed();
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
#if defined(Q_OS_WIN)
    return true;
#else
    return false;
#endif
}

void PerfMonitor::configure(const QVariantMap &perf) {
    m_presentmon = perf.value("presentmon_path").toString();
}

void PerfMonitor::showPreview(bool compact) {
    m_frames.clear();
    m_frames.ingest("Application,ProcessID,SwapChainAddress,MsBetweenPresents", 0, 42);
    for (int i=0; i<120; ++i)
        m_frames.ingest(QByteArray("test,42,0x1,") + (i==70 ? "75" : "16.666667"), i*16, 42);
    m_metrics = m_frames.snapshot(1950);
    m_metrics["context"] = QVariantMap{{"game", "APERÇU DE TEST — CHIFFRES SIMULÉS"}, {"system", "gamecube"},
        {"profile", "1440p"}, {"api", "d3d11"}, {"priority", "balanced"}};
    m_metrics["emulator"] = "Dolphin (test)";
    m_metrics["pid"] = 42;
    m_metrics["timestamp"] = QDateTime::currentDateTime().toString(Qt::ISODate);
    m_metrics["launcherVersion"] = QCoreApplication::applicationVersion();
    m_metrics["screen"] = "2560 × 1440 · 144 Hz";
    m_metrics["cpu"] = 120.0; m_metrics["cpuPcPercent"] = 7.5;
    m_metrics["systemCpu"] = 18.0; m_metrics["ramMb"] = 980.0;
    m_metrics["gpuProcessBusy"] = 37.0;
    m_metrics["shortcutAvailable"] = true; m_metrics["fallbackAvailable"] = true;
    m_status = "APERÇU VISUEL UNIQUEMENT — aucune mesure d’un vrai jeu.";
    m_mode = compact ? 1 : 2; m_width = compact ? 340 : 1400; m_height = 850;
    m_x = 0; m_y = 0; m_display = true;
    Q_EMIT changed();
}

void PerfMonitor::setEnabled(bool enabled) {
    if (enabled == m_enabled) return;
    m_enabled = enabled;
    m_settings.setValue("enabled", enabled);
    if (m_active && enabled && supported()) {
        QCoreApplication::instance()->installNativeEventFilter(this);
        m_attempted = false;
        registerShortcut();
        m_mode = 1;
        if (m_pid) startCapture();
        m_timer.start();
    } else if (!enabled) {
        QCoreApplication::instance()->removeNativeEventFilter(this);
        m_timer.stop(); unregisterShortcut();
        m_mode = 0; m_display = false;
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
    m_clock.restart();
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
    m_mode = active ? 1 : 0; m_display = false;
    m_context.clear(); m_system.reset();
    m_metrics.clear(); m_frames.clear(); m_buffer.clear();
    QFile::remove(m_dir.filePath("session.json"));
    if (active && m_enabled && supported()) {
        QCoreApplication::instance()->installNativeEventFilter(this);
        m_status = tr("En attente du processus du jeu…");
        registerShortcut();
        m_timer.start();
    }
    Q_EMIT changed();
}

void PerfMonitor::registerShortcut() {
#ifdef Q_OS_WIN
    m_registered = RegisterHotKey(nullptr, HotkeyId, MOD_NOREPEAT, VK_F1 + m_key - 1);
    m_fallbackRegistered = RegisterHotKey(nullptr, FallbackHotkeyId, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, VK_F8);
    if (!m_registered)
        m_status = tr("F%1 est déjà utilisée. Choisir une autre touche dans Performance.").arg(m_key);
#endif
}

void PerfMonitor::unregisterShortcut() {
#ifdef Q_OS_WIN
    if (m_registered) UnregisterHotKey(nullptr, HotkeyId);
    if (m_fallbackRegistered) UnregisterHotKey(nullptr, FallbackHotkeyId);
#endif
    m_registered = false;
    m_fallbackRegistered = false;
}

bool PerfMonitor::nativeEventFilter(const QByteArray &, void *message, qintptr *) {
#ifdef Q_OS_WIN
    const auto *msg = static_cast<MSG *>(message);
    if (msg->message == WM_HOTKEY && (msg->wParam == HotkeyId || msg->wParam == FallbackHotkeyId) && m_active && m_pid) {
        DWORD foreground = 0;
        GetWindowThreadProcessId(GetForegroundWindow(), &foreground);
        if (foreground == m_pid) { m_mode = (m_mode + 1) % 3; poll(); }
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
        const auto doc = QJsonDocument::fromJson(file.read(65536));
        const auto object = doc.object();
        const auto id = object.value("pid").toInteger();
        if (id <= 0 || id > 0xffffffffLL) return;
        m_pid = quint32(id);
        m_context = object.value("context").toObject().toVariantMap();
        m_metrics["emulator"] = QFileInfo(object.value("program").toString()).completeBaseName();
#ifdef Q_OS_WIN
        m_process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, FALSE, m_pid);
#endif
        startCapture();
    }
    const QString emulator = m_metrics.value("emulator").toString();
    const QVariant collectorError = m_metrics.value("collectorError");
    m_metrics = m_frames.snapshot(m_clock.elapsed());
    m_metrics["emulator"] = emulator;
    m_metrics["shortcutAvailable"] = m_registered;
    m_metrics["fallbackAvailable"] = m_fallbackRegistered;
    m_metrics["collectorError"] = collectorError;
    m_metrics["context"] = m_context;
    m_metrics["pid"] = m_pid;
    m_metrics["timestamp"] = QDateTime::currentDateTime().toString(Qt::ISODate);
    m_metrics["launcherVersion"] = QCoreApplication::applicationVersion();
    m_metrics["sessionSeconds"] = m_clock.elapsed()/1000;
    const auto system = m_system.sample(m_pid);
    for (auto it = system.cbegin(); it != system.cend(); ++it) m_metrics[it.key()] = it.value();
    sampleProcess();
#ifdef Q_OS_WIN
    const HWND window = GetForegroundWindow();
    DWORD foreground = 0;
    GetWindowThreadProcessId(window, &foreground);
    if (foreground == m_pid && !m_registered && !m_fallbackRegistered) registerShortcut();
    if (foreground != m_pid) unregisterShortcut();
    m_display = m_mode > 0 && foreground == m_pid;
    if (m_display) {
        MONITORINFOEXW monitor{};
        monitor.cbSize = sizeof(monitor);
        GetMonitorInfoW(MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST), &monitor);
        QScreen *screen = QGuiApplication::primaryScreen();
        for (auto *candidate : QGuiApplication::screens())
            if (candidate->name() == QString::fromWCharArray(monitor.szDevice)) screen = candidate;
        if (screen) {
            const QRect bounds = screen->geometry();
            m_width = m_mode == 2 ? bounds.width() - 36 : 340;
            m_height = bounds.height() - 36;
            m_x = m_mode == 2 ? bounds.left() + 18 : bounds.right() - m_width - 17;
            m_y = bounds.top() + 18;
            m_metrics["screen"] = QStringLiteral("%1 × %2 · %3 Hz").arg(screen->size().width()).arg(screen->size().height()).arg(screen->refreshRate(), 0, 'f', 1);
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
            // FILETIME uses 100 ns units; expose process CPU as a percentage
            // of one logical CPU (a multi-threaded process may exceed 100%).
            m_metrics["cpu"] = (cpu - m_previousCpu) / 10000.0
                / double(now - m_previousWall) * 100.0;
        m_previousCpu = cpu; m_previousWall = now;
        if (m_metrics.contains("cpu")) m_metrics["cpuPcPercent"] = m_metrics["cpu"].toDouble()/qMax(1, QThread::idealThreadCount());
    }
    PROCESS_MEMORY_COUNTERS memory{};
    memory.cb = sizeof(memory);
    if (GetProcessMemoryInfo(process, &memory, sizeof(memory)))
        m_metrics["ramMb"] = memory.WorkingSetSize / 1048576.0;
#elif defined(Q_OS_LINUX)
    QFile statFile(QStringLiteral("/proc/%1/stat").arg(m_pid));
    if (!statFile.open(QIODevice::ReadOnly))
        return;
    const QByteArray raw = statFile.readAll().trimmed();
    const qsizetype commEnd = raw.lastIndexOf(')');
    if (commEnd < 0 || commEnd + 2 >= raw.size())
        return;
    const QList<QByteArray> fields = raw.mid(commEnd + 2).split(' ');
    if (fields.size() < 17)
        return;
    bool ok = false;
    const quint64 utime = fields[11].toULongLong(&ok);
    const quint64 stime = fields[12].toULongLong(&ok);
    if (!ok)
        return;
    const quint64 cpu = utime + stime;
    const qint64 now = m_clock.elapsed();
    const long ticksPerSecond = sysconf(_SC_CLK_TCK);
    if (ticksPerSecond > 0 && m_previousWall && now > m_previousWall && cpu >= m_previousCpu)
        m_metrics["cpu"] = (cpu - m_previousCpu) * 1000000.0
            / ticksPerSecond / double(now - m_previousWall);
    m_previousCpu = cpu;
    m_previousWall = now;

    QFile status(QStringLiteral("/proc/%1/status").arg(m_pid));
    if (status.open(QIODevice::ReadOnly)) {
        const QList<QByteArray> lines = status.readAll().split('\n');
        for (const QByteArray &line : lines) {
            if (!line.startsWith("VmRSS:"))
                continue;
            const QList<QByteArray> parts = line.simplified().split(' ');
            if (parts.size() >= 2)
                m_metrics["ramMb"] = parts[1].toDouble() / 1024.0;
            break;
        }
    }
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
    m_collectorDiagnostic.clear();
    m_metrics.remove("collectorError");
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
