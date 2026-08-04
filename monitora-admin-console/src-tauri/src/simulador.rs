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
    let valor: Value = serde_json::from_str(texto).map_err(|err| format!("Mensagem inválida: {err}"))?;

    if let Some(evento) = valor.get("event").and_then(|v| v.as_str()) {
        match evento {
            "servicos:sincronizados" => {
                if let Some(servicos) = extrair_servicos(&valor) {
                    *estado.servicos.lock().unwrap() = servicos.clone();
                    let _ = app.emit("servicos:sincronizados", servicos);
                }
            }
            "heartbeat:recebido" => {
                if let Some(hb) = extrair_heartbeat(&valor) {
                    let servico_id = hb.servico_id.clone();
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
                    let _ = db::salvar_heartbeat(estado, hb.clone()).await;
                    let _ = app.emit("heartbeat:recebido", (servico_id, hb));
                }
            }
            "alerta:novo" => {
                if let Some(alerta) = extrair_alerta(&valor) {
                    {
                        let mut alertas = estado.alertas.lock().unwrap();
                        alertas.insert(0, alerta.clone());
                        if alertas.len() > 100 {
                            alertas.truncate(100);
                        }
                    }
                    let _ = db::salvar_alerta(estado, alerta.clone()).await;
                    let _ = app.emit("alerta:novo", alerta);
                    let _ = app
                        .notification()
                        .builder()
                        .title("MONITORA+ — novo alerta")
                        .body("Um novo alerta chegou pelo WebSocket")
                        .show();
                }
            }
            "conexao:status" => {
                if let Some(status) = extrair_status(&valor) {
                    let mut conexao = estado.conectado.lock().unwrap();
                    *conexao = status;
                    drop(conexao);
                    emit_status(app, estado);
                }
            }
            _ => {}
        }

        return Ok(());
    }

    if let Some(servicos) = extrair_servicos(&valor) {
        *estado.servicos.lock().unwrap() = servicos.clone();
        let _ = app.emit("servicos:sincronizados", servicos);
        return Ok(());
    }

    if let Some(hb) = extrair_heartbeat(&valor) {
        let servico_id = hb.servico_id.clone();
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
        let _ = db::salvar_heartbeat(estado, hb.clone()).await;
        let _ = app.emit("heartbeat:recebido", (servico_id, hb));
        return Ok(());
    }

    if let Some(alerta) = extrair_alerta(&valor) {
        {
            let mut alertas = estado.alertas.lock().unwrap();
            alertas.insert(0, alerta.clone());
            if alertas.len() > 100 {
                alertas.truncate(100);
            }
        }
        let _ = db::salvar_alerta(estado, alerta.clone()).await;
        let _ = app.emit("alerta:novo", alerta);
        return Ok(());
    }

    if let Some(status) = extrair_status(&valor) {
        let mut conexao = estado.conectado.lock().unwrap();
        *conexao = status;
        drop(conexao);
        emit_status(app, estado);
    }

    Ok(())
}

pub async fn conectar_e_ouvir(app: AppHandle, estado: Arc<Estado>) -> Result<(), String> {
    let url = estado.ws_url.lock().unwrap().clone();
    if url.trim().is_empty() {
        return Err("Informe a URL do WebSocket.".into());
    }

    let ws_url = Url::parse(&url).map_err(|err| format!("URL inválida: {err}"))?;
    let ws_url = ws_url.to_string();

    marcar_status(&estado, false);
    emit_status(&app, &estado);

    let (ws_stream, _) = connect_async(ws_url).await.map_err(|err| format!("Falha ao conectar: {err}"))?;
    let (mut write, mut read) = ws_stream.split();

    marcar_status(&estado, true);
    emit_status(&app, &estado);

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(texto)) => {
                if let Err(err) = processar_mensagem(&app, &estado, &texto).await {
                    let _ = app.emit("erro:conexao", err);
                }
            }
            Ok(Message::Ping(payload)) => {
                let _ = write.send(Message::Pong(payload)).await;
            }
            Ok(Message::Close(_)) => break,
            Ok(Message::Binary(_)) => {}
            Err(err) => {
                let _ = app.emit("erro:conexao", err.to_string());
                break;
            }
            _ => {}
        }
    }

    marcar_status(&estado, false);
    emit_status(&app, &estado);
    Ok(())
}
