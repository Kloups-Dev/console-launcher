#pragma once
#include <QVariantMap>
#include <memory>

// Read-only OS/driver counters. Unsupported sensors stay absent.
class SystemTelemetry {
public:
    SystemTelemetry();
    ~SystemTelemetry();
    QVariantMap sample(quint32 pid);
    void reset();
private:
    struct State;
    std::unique_ptr<State> d;
};
