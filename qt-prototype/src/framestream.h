#pragma once

#include <QByteArray>
#include <QHash>
#include <QStringList>
#include <QVariantMap>
#include <algorithm>
#include <cmath>
#include <deque>

// Bounded rolling presentation statistics. Never infers emulation speed.
class FrameStream {
public:
    static QStringList csv(const QByteArray &line) {
        QStringList out;
        QString field;
        bool quoted = false;
        const QString text = QString::fromUtf8(line).trimmed();
        for (qsizetype i = 0; i < text.size(); ++i) {
            const QChar c = text[i];
            if (c == '"') {
                if (quoted && i + 1 < text.size() && text[i + 1] == '"') {
                    field += c;
                    ++i;
                } else quoted = !quoted;
            } else if (c == ',' && !quoted) {
                out << field.trimmed(); field.clear();
            } else field += c;
        }
        out << field.trimmed();
        return out;
    }
    void clear() { m_columns.clear(); m_streams.clear(); }
    void ingest(const QByteArray &line, qint64 nowMs, quint32 expectedPid) {
        const auto fields = csv(line);
        if (fields.contains(QStringLiteral("msbetweenpresents"), Qt::CaseInsensitive)) {
            m_columns = fields;
            for (auto &column : m_columns) column = column.toLower();
            return;
        }
        if (m_columns.isEmpty() || fields.size() != m_columns.size()) return;
        const int pid = m_columns.indexOf("processid");
        if (pid < 0 || fields[pid].toUInt() != expectedPid) return;
        const int ft = m_columns.indexOf("msbetweenpresents");
        bool ok = false;
        const double ms = fields[ft].toDouble(&ok);
        if (!ok || !std::isfinite(ms) || ms <= 0 || ms > 10000) return;
        const int swap = m_columns.indexOf("swapchainaddress");
        const QString key = swap >= 0 ? fields[swap] : QStringLiteral("main");
        for (auto it = m_streams.begin(); it != m_streams.end(); ) {
            if (!it.value().empty() && nowMs - it.value().back().arrival > 2000) it = m_streams.erase(it);
            else ++it;
        }
        if (!m_streams.contains(key) && m_streams.size() >= 8) return;
        auto &frames = m_streams[key];
        const auto optional = [&](const char *name) {
            const int at = m_columns.indexOf(QString::fromLatin1(name).toLower());
            if (at < 0) return -1.0;
            bool valid = false;
            const double v = fields[at].toDouble(&valid);
            return valid && std::isfinite(v) && v > 0 ? v : -1.0;
        };
        const int dropped = m_columns.indexOf("dropped");
        frames.push_back({nowMs, ms, optional("MsGPUActive"), optional("MsUntilRenderComplete"),
                          optional("MsUntilDisplayed"), dropped < 0 ? -1 : (fields[dropped] == "1" ? 1 : 0)});
        while (frames.size() > 1024 || (!frames.empty() && nowMs - frames.front().arrival > 2000))
            frames.pop_front();
    }
    QVariantMap snapshot(qint64 nowMs) const {
        QVector<double> values;
        std::deque<Frame> selected;
        qint64 newest = -1;
        for (auto it = m_streams.cbegin(); it != m_streams.cend(); ++it) {
            QVector<double> candidate;
            if (it.value().empty() || nowMs - it.value().back().arrival > 1500) continue;
            for (const auto &frame : it.value())
                if (nowMs - frame.arrival <= 2000) candidate << frame.ms;
            if (candidate.size() > values.size()) {
                values = candidate; newest = it.value().back().arrival;
                selected = it.value();
            }
        }
        if (values.size() < 20) return {{"measured", false}, {"frames", values.size()}};
        double sum = 0;
        for (double v : values) sum += v;
        QVariantList timeline;
        double gpu = 0, render = 0, display = 0;
        int gpuN = 0, renderN = 0, displayN = 0, droppedN = 0, dropped = 0;
        for (const auto &f : selected) {
            if (nowMs - f.arrival > 2000) continue;
            timeline << f.ms;
            if (f.gpu > 0) { gpu += f.gpu; ++gpuN; }
            if (f.render > 0) { render += f.render; ++renderN; }
            if (f.display > 0) { display += f.display; ++displayN; }
            if (f.dropped >= 0) { ++droppedN; dropped += f.dropped; }
        }
        std::sort(values.begin(), values.end());
        const double median = values[values.size() / 2];
        const double p99 = values[qRound((values.size() - 1) * 0.99)];
        int spikes = 0;
        for (double v : values) if (v > median * 1.5) ++spikes;
        QVariantMap result{{"measured", true}, {"frames", values.size()},
                {"fps", values.size() * 1000.0 / sum}, {"frameMs", sum / values.size()},
                {"p99Ms", p99}, {"low1", 1000.0 / p99},
                {"p95Ms", values[qRound((values.size()-1)*0.95)]}, {"medianMs", median},
                {"minMs", values.front()}, {"maxMs", values.back()}, {"timeline", timeline},
                {"spikes", spikes}, {"ageMs", nowMs - newest}};
        if (gpuN) result["gpuActiveMs"] = gpu/gpuN;
        if (renderN) result["renderDelayMs"] = render/renderN;
        if (displayN) result["displayDelayMs"] = display/displayN;
        if (droppedN) result["droppedPercent"] = 100.0*dropped/droppedN;
        return result;
    }
private:
    struct Frame { qint64 arrival; double ms, gpu, render, display; int dropped; };
    QStringList m_columns;
    QHash<QString, std::deque<Frame>> m_streams;
};
