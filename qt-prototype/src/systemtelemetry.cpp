#include "systemtelemetry.h"
#include <QLibrary>
#include <QDir>
#include <QVector>
#include <algorithm>
#include <cmath>
#include <vector>
#ifdef Q_OS_WIN
#define NOMINMAX
#include <windows.h>
#include <pdh.h>
#include <pdhmsg.h>
#endif

struct SystemTelemetry::State {
#ifdef Q_OS_WIN
    PDH_HQUERY query = nullptr;
    PDH_HCOUNTER gpu = nullptr, memory = nullptr;
    quint64 idle = 0, total = 0;
    QLibrary nvml;
    bool initialized = false;
    using Init = int (*)();
    using Count = int (*)(unsigned *);
    using Device = int (*)(unsigned, void **);
    using Name = int (*)(void *, char *, unsigned);
    struct Util { unsigned gpu, memory; };
    struct Memory { unsigned long long total, free, used; };
    using Utilization = int (*)(void *, Util *);
    using GetMemory = int (*)(void *, Memory *);
    using Sensor = int (*)(void *, unsigned, unsigned *);
    using Power = int (*)(void *, unsigned *);
    Count count = nullptr;
    Device device = nullptr;
    Name name = nullptr;
    Utilization utilization = nullptr;
    GetMemory getMemory = nullptr;
    Sensor temperature = nullptr, clock = nullptr;
    Power power = nullptr;
#endif
};

SystemTelemetry::SystemTelemetry() : d(std::make_unique<State>()) {
#ifdef Q_OS_WIN
    // Explicit system path: do not load driver DLLs from the current folder.
    wchar_t system[MAX_PATH]{};
    if (GetSystemDirectoryW(system, MAX_PATH)) {
        d->nvml.setFileName(QString::fromWCharArray(system) + "/nvml.dll");
        if (d->nvml.load()) {
            const auto init = reinterpret_cast<State::Init>(d->nvml.resolve("nvmlInit_v2"));
            d->initialized = init && init() == 0;
            if (d->initialized) {
                d->count = reinterpret_cast<State::Count>(d->nvml.resolve("nvmlDeviceGetCount_v2"));
                d->device = reinterpret_cast<State::Device>(d->nvml.resolve("nvmlDeviceGetHandleByIndex_v2"));
                d->name = reinterpret_cast<State::Name>(d->nvml.resolve("nvmlDeviceGetName"));
                d->utilization = reinterpret_cast<State::Utilization>(d->nvml.resolve("nvmlDeviceGetUtilizationRates"));
                d->getMemory = reinterpret_cast<State::GetMemory>(d->nvml.resolve("nvmlDeviceGetMemoryInfo"));
                d->temperature = reinterpret_cast<State::Sensor>(d->nvml.resolve("nvmlDeviceGetTemperature"));
                d->clock = reinterpret_cast<State::Sensor>(d->nvml.resolve("nvmlDeviceGetClockInfo"));
                d->power = reinterpret_cast<State::Power>(d->nvml.resolve("nvmlDeviceGetPowerUsage"));
            }
        }
    }
#endif
}

SystemTelemetry::~SystemTelemetry() {
    reset();
#ifdef Q_OS_WIN
    if (d->initialized) {
        const auto shutdown = reinterpret_cast<State::Init>(d->nvml.resolve("nvmlShutdown"));
        if (shutdown) shutdown();
    }
#endif
}

void SystemTelemetry::reset() {
#ifdef Q_OS_WIN
    if (d->query) PdhCloseQuery(d->query);
    d->query = nullptr; d->gpu = nullptr; d->memory = nullptr;
    d->idle = 0; d->total = 0;
#endif
}

QVariantMap SystemTelemetry::sample(quint32 pid) {
    QVariantMap out;
#ifdef Q_OS_WIN
    MEMORYSTATUSEX mem{}; mem.dwLength = sizeof(mem);
    if (GlobalMemoryStatusEx(&mem)) {
        out["systemRamUsedMb"] = (mem.ullTotalPhys - mem.ullAvailPhys) / 1048576.0;
        out["systemRamTotalMb"] = mem.ullTotalPhys / 1048576.0;
        out["systemRamPercent"] = double(mem.dwMemoryLoad);
    }
    FILETIME idle{}, kernel{}, user{};
    if (GetSystemTimes(&idle, &kernel, &user)) {
        const auto ticks = [](FILETIME f) { return (quint64(f.dwHighDateTime) << 32) | f.dwLowDateTime; };
        const quint64 nowIdle = ticks(idle), nowTotal = ticks(kernel) + ticks(user);
        if (d->total && nowTotal > d->total && nowIdle >= d->idle)
            out["systemCpu"] = std::clamp(100.0 * (1.0 - double(nowIdle-d->idle) / double(nowTotal-d->total)), 0.0, 100.0);
        d->idle = nowIdle; d->total = nowTotal;
    }
    if (!d->query && PdhOpenQueryW(nullptr, 0, &d->query) == ERROR_SUCCESS) {
        PdhAddEnglishCounterW(d->query, L"\\GPU Engine(*)\\Utilization Percentage", 0, &d->gpu);
        PdhAddEnglishCounterW(d->query, L"\\GPU Process Memory(*)\\Dedicated Usage", 0, &d->memory);
    }
    if (d->query && PdhCollectQueryData(d->query) == ERROR_SUCCESS) {
        const auto values = [pid](PDH_HCOUNTER counter, bool maximum) -> QVariant {
            if (!counter) return {};
            DWORD bytes = 0, count = 0;
            if (PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &bytes, &count, nullptr) != PDH_MORE_DATA || bytes > 8*1024*1024)
                return {};
            std::vector<unsigned char> storage(bytes);
            auto *items = reinterpret_cast<PDH_FMT_COUNTERVALUE_ITEM_W *>(storage.data());
            if (PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &bytes, &count, items) != ERROR_SUCCESS) return {};
            double value = 0; bool found = false;
            const QString prefix = QStringLiteral("pid_%1_").arg(pid);
            for (DWORD i = 0; i < count; ++i) {
                const auto &v = items[i].FmtValue;
                if (!QString::fromWCharArray(items[i].szName).startsWith(prefix) ||
                    (v.CStatus != PDH_CSTATUS_VALID_DATA && v.CStatus != PDH_CSTATUS_NEW_DATA) || !std::isfinite(v.doubleValue)) continue;
                found = true;
                value = maximum ? std::max(value, v.doubleValue) : value + v.doubleValue;
            }
            return found ? QVariant(maximum ? std::clamp(value, 0.0, 100.0) : value/1048576.0) : QVariant();
        };
        const auto gpu = values(d->gpu, true), memory = values(d->memory, false);
        if (gpu.isValid()) out["gpuProcessBusy"] = gpu;
        if (memory.isValid()) out["gpuProcessMemoryMb"] = memory;
    }
    QVariantList cards;
    unsigned count = 0;
    if (d->initialized && d->count && d->device && d->count(&count) == 0) {
        for (unsigned i = 0; i < std::min(count, 8u); ++i) {
            void *device = nullptr;
            if (d->device(i, &device) != 0) continue;
            QVariantMap card;
            char name[128]{};
            if (d->name && d->name(device, name, sizeof(name)) == 0) card["name"] = QString::fromUtf8(name);
            State::Util util{};
            if (d->utilization && d->utilization(device, &util) == 0) card["busy"] = util.gpu;
            State::Memory memory{};
            if (d->getMemory && d->getMemory(device, &memory) == 0) {
                card["usedMb"] = memory.used/1048576.0; card["totalMb"] = memory.total/1048576.0;
            }
            unsigned value = 0;
            if (d->temperature && d->temperature(device, 0, &value) == 0) card["temperature"] = value;
            if (d->clock && d->clock(device, 0, &value) == 0) card["clockMhz"] = value;
            if (d->power && d->power(device, &value) == 0) card["powerW"] = value/1000.0;
            card["source"] = "Pilote NVIDIA / NVML · carte entière";
            cards << card;
        }
    }
    out["gpuCards"] = cards;
#else
    Q_UNUSED(pid)
#endif
    return out;
}
