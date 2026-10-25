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
        const auto header = fields.indexOf("MsBetweenPresents");
        if (header >= 0) { m_columns = fields; return; }
        if (m_columns.isEmpty() || fields.size() != m_columns.size()) return;
        const int pid = m_columns.indexOf("ProcessID");
        if (pid < 0 || fields[pid].toUInt() != expectedPid) return;
        const int ft = m_columns.indexOf("MsBetweenPresents");
        bool ok = false;
        const double ms = fields[ft].toDouble(&ok);
        if (!ok || !std::isfinite(ms) || ms <= 0 || ms > 10000) return;
        const int swap = m_columns.indexOf("SwapChainAddress");
        const QString key = swap >= 0 ? fields[swap] : QStringLiteral("main");
        for (auto it = m_streams.begin(); it != m_streams.end(); ) {
            if (!it.value().empty() && nowMs - it.value().back().arrival > 2000) it = m_streams.erase(it);
            else ++it;
        }
        if (!m_streams.contains(key) && m_streams.size() >= 8) return;
        auto &frames = m_streams[key];
        frames.push_back({nowMs, ms});
        while (frames.size() > 1024 || (!frames.empty() && nowMs - frames.front().arrival > 2000))
            frames.pop_front();
    }
    QVariantMap snapshot(qint64 nowMs) const {
        QVector<double> values;
        qint64 newest = -1;
        for (auto it = m_streams.cbegin(); it != m_streams.cend(); ++it) {
            QVector<double> candidate;
            if (it.value().empty() || nowMs - it.value().back().arrival > 1500) continue;
            for (const auto &frame : it.value())
                if (nowMs - frame.arrival <= 2000) candidate << frame.ms;
            if (candidate.size() > values.size()) {
                values = candidate; newest = it.value().back().arrival;
            }
        }
        if (values.size() < 20) return {{"measured", false}, {"frames", values.size()}};
        double sum = 0;
        for (double v : values) sum += v;
        std::sort(values.begin(), values.end());
        const double median = values[values.size() / 2];
        const double p99 = values[qRound((values.size() - 1) * 0.99)];
        int spikes = 0;
        for (double v : values) if (v > median * 1.5) ++spikes;
        return {{"measured", true}, {"frames", values.size()},
                {"fps", values.size() * 1000.0 / sum}, {"frameMs", sum / values.size()},
                {"p99Ms", p99}, {"low1", 1000.0 / p99},
                {"spikes", spikes}, {"ageMs", nowMs - newest}};
    }
private:
    struct Frame { qint64 arrival; double ms; };
    QStringList m_columns;
    QHash<QString, std::deque<Frame>> m_streams;
};
