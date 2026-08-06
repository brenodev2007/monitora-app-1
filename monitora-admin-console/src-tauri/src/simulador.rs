use crate::db;
use crate::estado::Estado;
use crate::models::{Alerta, Heartbeat, Servico, StatusConexao};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

fn agora_iso() -> String {
    Utc::now().to_rfc3339()
}

fn marcar_status(estado: &Arc<Estado>, conectado: bool) {
    let mut conexao = estado.conectado.lock().unwrap();
    conexao.conectado = conectado;
    conexao.ultima_tentativa_em = agora_iso();
}

fn emit_status(app: &AppHandle, estado: &Arc<Estado>) {
    let payload = estado.conectado.lock().unwrap().clone();
    log::debug!("[WS] Emitindo conexao:status → conectado={}", payload.conectado);
    let _ = app.emit("conexao:status", payload);
}

fn extrair_servicos(valor: &Value) -> Option<Vec<Servico>> {
    if let Some(arr) = valor.as_array() {
        return serde_json::from_value::<Vec<Servico>>(Value::Array(arr.clone())).ok();
    }

    if let Some(obj) = valor.as_object() {
        for chave in ["servicos", "services"] {
            if let Some(payload) = obj.get(chave) {
                if let Ok(servicos) = serde_json::from_value::<Vec<Servico>>(payload.clone()) {
                    return Some(servicos);
                }
            }
        }

        if let Some(payload) = obj.get("payload") {
            return extrair_servicos(payload);
        }
    }

    None
}

fn extrair_heartbeat(valor: &Value) -> Option<Heartbeat> {
    if let Some(obj) = valor.as_object() {
        if let Some(payload) = obj.get("payload") {
            return serde_json::from_value::<Heartbeat>(payload.clone()).ok();
        }
    }

    serde_json::from_value::<Heartbeat>(valor.clone()).ok()
}

fn extrair_alerta(valor: &Value) -> Option<Alerta> {
    if let Some(obj) = valor.as_object() {
        if let Some(payload) = obj.get("payload") {
            return serde_json::from_value::<Alerta>(payload.clone()).ok();
        }
    }

    serde_json::from_value::<Alerta>(valor.clone()).ok()
}

fn extrair_status(valor: &Value) -> Option<StatusConexao> {
    if let Some(obj) = valor.as_object() {
        if let Some(payload) = obj.get("payload") {
            return serde_json::from_value::<StatusConexao>(payload.clone()).ok();
        }
    }

    serde_json::from_value::<StatusConexao>(valor.clone()).ok()
}

async fn processar_mensagem(app: &AppHandle, estado: &Arc<Estado>, texto: &str) -> Result<(), String> {
    let valor: Value = serde_json::from_str(texto).map_err(|err| {
        let msg = format!("Mensagem inválida (JSON parse error): {err}");
        log::error!("[WS] {}", msg);
        msg
    })?;

    if let Some(evento) = valor.get("event").and_then(|v| v.as_str()) {
        log::debug!("[WS] Evento recebido: '{}'", evento);
        match evento {
            "servicos:sincronizados" => {
                if let Some(servicos) = extrair_servicos(&valor) {
                    log::info!("[WS] servicos:sincronizados → {} serviço(s)", servicos.len());
                    *estado.servicos.lock().unwrap() = servicos.clone();
                    let _ = app.emit("servicos:sincronizados", servicos);
                } else {
                    log::warn!("[WS] servicos:sincronizados → payload não pôde ser deserializado.");
                }
            }
            "heartbeat:recebido" => {
                if let Some(hb) = extrair_heartbeat(&valor) {
                    let servico_id = hb.servico_id.clone();
                    log::debug!("[WS] heartbeat:recebido → servico_id={} status={:?}", servico_id, hb.status);
                    {
                        let mut servicos = estado.servicos.lock().unwrap();
                        for servico in servicos.iter_mut() {
                            if servico.id == servico_id {
                                servico.status = hb.status;
                                servico.ultimo_heartbeat_em = Some(hb.recebido_em.clone());
                                servico.historico.push(hb.clone());
                                if servico.historico.len() > 40 {
                                    servico.historico.remove(0);
                                }
                                break;
                            }
                        }
                    }
                    if let Err(err) = db::salvar_heartbeat(estado, hb.clone()).await {
                        log::error!("[WS] Erro ao salvar heartbeat no banco: {}", err);
                    }
                    let _ = app.emit("heartbeat:recebido", (servico_id, hb));
                } else {
                    log::warn!("[WS] heartbeat:recebido → payload não pôde ser deserializado.");
                }
            }
            "alerta:novo" => {
                if let Some(alerta) = extrair_alerta(&valor) {
                    log::info!(
                        "[WS] alerta:novo → id={} servico='{}' tipo='{}'",
                        alerta.id, alerta.servico_nome, alerta.tipo
                    );
                    {
                        let mut alertas = estado.alertas.lock().unwrap();
                        alertas.insert(0, alerta.clone());
                        if alertas.len() > 100 {
                            alertas.truncate(100);
                        }
                    }
                    if let Err(err) = db::salvar_alerta(estado, alerta.clone()).await {
                        log::error!("[WS] Erro ao salvar alerta no banco: {}", err);
                    }
                    let _ = app.emit("alerta:novo", alerta.clone());

                    // Notificação de sistema — mostra o serviço e o tipo de alerta
                    let titulo = format!("⚠️ MONITORA+ — Alerta: {}", alerta.servico_nome);
                    let corpo = format!("{} (canal: {:?})", alerta.tipo, alerta.canal);
                    match app.notification().builder().title(&titulo).body(&corpo).show() {
                        Ok(_) => log::debug!("[WS] Notificação de sistema disparada."),
                        Err(err) => log::warn!("[WS] Falha ao exibir notificação: {}", err),
                    }
                } else {
                    log::warn!("[WS] alerta:novo → payload não pôde ser deserializado.");
                }
            }
            "conexao:status" => {
                if let Some(status) = extrair_status(&valor) {
                    log::info!("[WS] conexao:status via evento → conectado={}", status.conectado);
                    let mut conexao = estado.conectado.lock().unwrap();
                    *conexao = status;
                    drop(conexao);
                    emit_status(app, estado);
                } else {
                    log::warn!("[WS] conexao:status → payload não pôde ser deserializado.");
                }
            }
            outro => {
                log::warn!("[WS] Evento desconhecido recebido: '{}'", outro);
            }
        }

        return Ok(());
    }

    // Sem campo "event" — tenta inferir o tipo da mensagem
    log::debug!("[WS] Mensagem sem campo 'event' — tentando inferir tipo...");

    if let Some(servicos) = extrair_servicos(&valor) {
        log::info!("[WS] (sem event) → servicos:sincronizados inferido → {} serviço(s)", servicos.len());
        *estado.servicos.lock().unwrap() = servicos.clone();
        let _ = app.emit("servicos:sincronizados", servicos);
        return Ok(());
    }

    if let Some(hb) = extrair_heartbeat(&valor) {
        let servico_id = hb.servico_id.clone();
        log::debug!("[WS] (sem event) → heartbeat inferido para servico_id={}", servico_id);
        {
            let mut servicos = estado.servicos.lock().unwrap();
            for servico in servicos.iter_mut() {
                if servico.id == servico_id {
                    servico.status = hb.status;
                    servico.ultimo_heartbeat_em = Some(hb.recebido_em.clone());
                    servico.historico.push(hb.clone());
                    if servico.historico.len() > 40 {
                        servico.historico.remove(0);
                    }
                    break;
                }
            }
        }
        if let Err(err) = db::salvar_heartbeat(estado, hb.clone()).await {
            log::error!("[WS] Erro ao salvar heartbeat inferido no banco: {}", err);
        }
        let _ = app.emit("heartbeat:recebido", (servico_id, hb));
        return Ok(());
    }

    if let Some(alerta) = extrair_alerta(&valor) {
        log::info!(
            "[WS] (sem event) → alerta inferido → id={} servico='{}'",
            alerta.id, alerta.servico_nome
        );
        {
            let mut alertas = estado.alertas.lock().unwrap();
            alertas.insert(0, alerta.clone());
            if alertas.len() > 100 {
                alertas.truncate(100);
            }
        }
        if let Err(err) = db::salvar_alerta(estado, alerta.clone()).await {
            log::error!("[WS] Erro ao salvar alerta inferido no banco: {}", err);
        }
        let _ = app.emit("alerta:novo", alerta);
        return Ok(());
    }

    if let Some(status) = extrair_status(&valor) {
        log::info!("[WS] (sem event) → status inferido → conectado={}", status.conectado);
        let mut conexao = estado.conectado.lock().unwrap();
        *conexao = status;
        drop(conexao);
        emit_status(app, estado);
        return Ok(());
    }

    log::warn!("[WS] Mensagem recebida não pôde ser identificada: {}", &texto[..texto.len().min(200)]);
    Ok(())
}

pub async fn conectar_e_ouvir(app: AppHandle, estado: Arc<Estado>) -> Result<(), String> {
    let url = estado.ws_url.lock().unwrap().clone();
    if url.trim().is_empty() {
        log::warn!("[WS] conectar_e_ouvir → URL vazia, abortando.");
        return Err("Informe a URL do WebSocket.".into());
    }

    log::info!("[WS] Conectando ao WebSocket: {}", url);

    let ws_url = Url::parse(&url).map_err(|err| {
        let msg = format!("URL inválida '{}': {}", url, err);
        log::error!("[WS] {}", msg);
        msg
    })?;
    let ws_url = ws_url.to_string();

    marcar_status(&estado, false);
    emit_status(&app, &estado);

    let (ws_stream, response) = connect_async(&ws_url).await.map_err(|err| {
        let msg = format!("Falha ao conectar ao WebSocket '{}': {}", ws_url, err);
        log::error!("[WS] {}", msg);
        msg
    })?;

    log::info!(
        "[WS] Conexão estabelecida! Status HTTP: {}",
        response.status()
    );

    let (mut write, mut read) = ws_stream.split();

    marcar_status(&estado, true);
    emit_status(&app, &estado);
    log::info!("[WS] Ouvindo mensagens de '{}'...", ws_url);

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(texto)) => {
                log::debug!("[WS] Mensagem TEXT recebida ({} bytes)", texto.len());
                if let Err(err) = processar_mensagem(&app, &estado, &texto).await {
                    log::error!("[WS] Erro ao processar mensagem: {}", err);
                    let _ = app.emit("erro:conexao", err);
                }
            }
            Ok(Message::Ping(payload)) => {
                log::debug!("[WS] Ping recebido → enviando Pong");
                let _ = write.send(Message::Pong(payload)).await;
            }
            Ok(Message::Close(frame)) => {
                log::info!("[WS] Servidor fechou a conexão: {:?}", frame);
                break;
            }
            Ok(Message::Binary(data)) => {
                log::debug!("[WS] Mensagem binária recebida ({} bytes) — ignorada.", data.len());
            }
            Err(err) => {
                let msg = err.to_string();
                log::error!("[WS] Erro na stream WebSocket: {}", msg);
                let _ = app.emit("erro:conexao", msg);
                break;
            }
            _ => {}
        }
    }

    marcar_status(&estado, false);
    emit_status(&app, &estado);
    log::info!("[WS] Loop de escuta encerrado para '{}'.", ws_url);
    Ok(())
}
