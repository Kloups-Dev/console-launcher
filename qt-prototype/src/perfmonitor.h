#pragma once
#include <QAbstractNativeEventFilter>
#include <QElapsedTimer>
#include <QObject>
#include <QProcess>
#include <QSettings>
#include <QTemporaryDir>
#include <QTimer>
#include <QVariantMap>
#include "framestream.h"
#include "systemtelemetry.h"

class PerfMonitor : public QObject, public QAbstractNativeEventFilter {
    Q_OBJECT
    Q_PROPERTY(QVariantMap metrics READ metrics NOTIFY changed)
    Q_PROPERTY(QString status READ status NOTIFY changed)
    Q_PROPERTY(bool display READ display NOTIFY changed)
    Q_PROPERTY(int viewMode READ viewMode NOTIFY changed)
    Q_PROPERTY(int panelWidth READ panelWidth NOTIFY changed)
    Q_PROPERTY(int panelHeight READ panelHeight NOTIFY changed)
    Q_PROPERTY(int panelX READ panelX NOTIFY changed)
    Q_PROPERTY(int panelY READ panelY NOTIFY changed)
    Q_PROPERTY(bool enabled READ enabled WRITE setEnabled NOTIFY changed)
    Q_PROPERTY(int functionKey READ functionKey WRITE setFunctionKey NOTIFY changed)
    Q_PROPERTY(bool supported READ supported CONSTANT)
public:
    explicit PerfMonitor(QObject *parent = nullptr);
    ~PerfMonitor() override;
    void configure(const QVariantMap &perf);
    void setActive(bool active);
    QVariantMap metrics() const { return m_metrics; }
    QString status() const { return m_status; }
    bool display() const { return m_display; }
    int viewMode() const { return m_mode; }
    int panelWidth() const { return m_width; }
    int panelHeight() const { return m_height; }
    int panelX() const { return m_x; }
    int panelY() const { return m_y; }
    bool enabled() const { return m_enabled; }
    int functionKey() const { return m_key; }
    bool supported() const;
    void setEnabled(bool enabled);
    void setFunctionKey(int key);
    // Explicit command-line visual test only. Never used in a game session.
    void showPreview(bool compact);
    bool nativeEventFilter(const QByteArray &, void *message, qintptr *) override;
Q_SIGNALS:
    void changed();
private:
    void poll();
    void startCapture();
    void stopCapture();
    void readFrames();
    void registerShortcut();
    void unregisterShortcut();
    void sampleProcess();
    QTemporaryDir m_dir;
    QSettings m_settings;
    QTimer m_timer;
    QElapsedTimer m_clock;
    QProcess m_capture;
    QByteArray m_buffer;
    FrameStream m_frames;
    SystemTelemetry m_system;
    QVariantMap m_context;
    QVariantMap m_metrics;
    QString m_status, m_presentmon;
    QString m_collectorDiagnostic;
    bool m_active = false, m_enabled = true, m_display = false;
    bool m_fallbackRegistered = false;
    bool m_registered = false, m_attempted = false;
    int m_key = 8, m_x = 20, m_y = 20;
    int m_mode = 0, m_width = 340, m_height = 900;
    quint32 m_pid = 0;
    void *m_process = nullptr;
    quint64 m_previousCpu = 0;
    qint64 m_previousWall = 0;
};
