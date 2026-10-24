#include "engine.h"

#include <QJsonDocument>
#include <QJsonObject>

#include "console_launcher_engine.h"

Engine::Engine(const QString &dataDir) {
    const QByteArray dir = dataDir.toUtf8();
    m_handle = cl_engine_open(dataDir.isEmpty() ? nullptr : dir.constData());
}

Engine::~Engine() {
    if (m_handle)
        cl_engine_close(m_handle);
}

Engine::Answer Engine::call(const QString &method, const QVariant &params) const {
    Answer a;
    if (!m_handle) {
        a.error = QStringLiteral("engine not available");
        return a;
    }
    const QByteArray m = method.toUtf8();
    QByteArray p;
    if (params.isValid() && !params.isNull())
        p = QJsonDocument::fromVariant(params).toJson(QJsonDocument::Compact);
    char *raw = cl_engine_call(m_handle, m.constData(), p.isEmpty() ? nullptr : p.constData());
    const QJsonObject o = QJsonDocument::fromJson(QByteArray(raw)).object();
    cl_string_free(raw);
    a.ok = o.value("ok").toBool();
    if (a.ok)
        a.result = o.value("result").toVariant();
    else
        a.error = o.value("error").toString();
    return a;
}
