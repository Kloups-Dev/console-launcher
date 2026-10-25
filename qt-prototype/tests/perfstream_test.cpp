#include "framestream.h"
#include <QCoreApplication>
#include <cstdio>

int main(int argc, char **argv) {
    QCoreApplication app(argc, argv);
    int failed = 0;
    const auto check = [&failed](bool ok, const char *label) {
        if (!ok) { std::fprintf(stderr, "FAIL: %s\n", label); ++failed; }
    };
    check(FrameStream::csv("\"Dolphin, test\",42,\"a\"\"b\"") == QStringList{"Dolphin, test", "42", "a\"b"}, "quoted CSV");
    FrameStream stream;
    stream.ingest("Application,ProcessID,SwapChainAddress,MsBetweenPresents", 0, 42);
    for (int i = 0; i < 120; ++i) {
        stream.ingest("Dolphin.exe,42,0x1,16.666667", i * 16, 42);
        stream.ingest("unrelated.exe,99,0x1,1", i * 16, 42);
        if (i < 25) stream.ingest("Dolphin.exe,42,0x2,100", i * 16, 42);
    }
    const auto measured = stream.snapshot(1950);
    check(measured.value("measured").toBool(), "enough data");
    check(std::abs(measured.value("fps").toDouble() - 60) < 0.01, "dominant swapchain only, expected PID only");
    check(measured.value("frames").toInt() == 120, "does not sum separate rendering surfaces");
    check(measured.value("timeline").toList().size() == 120, "bounded graph keeps chronological samples");
    check(!measured.contains("gpuActiveMs"), "missing GPU work is not reported as zero");
    check(!stream.snapshot(4000).value("measured").toBool(), "old values expire");
    stream.clear();
    check(!stream.snapshot(0).value("measured").toBool(), "new session starts empty");
    stream.ingest("Application,ProcessID,SwapChainAddress,MsBetweenPresents,MsGPUActive,MsUntilDisplayed,Dropped", 0, 42);
    for (int i=0; i<30; ++i) stream.ingest("game,42,0x1,33.333333,4,8,1", i*33, 42);
    const auto slowNative = stream.snapshot(1000);
    check(std::abs(slowNative.value("fps").toDouble()-30) < 0.01, "native 30 FPS stays 30, never inferred as emulation speed");
    check(slowNative.value("gpuActiveMs").toDouble() == 4, "GPU work parsed separately from frame time");
    check(slowNative.value("droppedPercent").toDouble() == 100, "dropped presentations exposed");
    stream.clear();
    stream.ingest("Application,ProcessID,SwapChainAddress,MsBetweenPresents", 0, 42);
    for (const auto &bad : {"nan", "inf", "-1", "0", "oops", "10001"})
        stream.ingest(QByteArray("game,42,0x1,") + bad, 0, 42);
    check(stream.snapshot(0).value("frames").toInt() == 0, "invalid frames rejected");
    for (int i = 0; i < 5000; ++i) stream.ingest("game,42,0x1,10", 0, 42);
    check(stream.snapshot(0).value("frames").toInt() == 1024, "bounded memory");
    std::printf("Presentation stream: %s\n", failed ? "FAILED" : "passed");
    return failed ? 1 : 0;
}
