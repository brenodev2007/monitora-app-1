use crate::db;
use crate::estado::Estado;
use crate::models::{Alerta, Servico, StatusConexao};
use chrono::Utc;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

fn agora_iso() -> String {
    Utc::now().to_rfc3339()
}

/// Autenticação do administrador usando o banco MySQL.
#[tauri::command]
pub async fn autenticar(estado: State<'_, Arc<Estado>>, email: String, senha: String) -> Result<bool, String> {
    db::autenticar(&estado, email, senha).await
}

#[tauri::command]
pub async fn listar_servicos(estado: State<'_, Arc<Estado>>) -> Result<Vec<Servico>, String> {
    db::listar_servicos(&estado).await
}

#[tauri::command]
pub async fn listar_alertas(estado: State<'_, Arc<Estado>>) -> Result<Vec<Alerta>, String> {
    db::listar_alertas(&estado).await
}

#[tauri::command]
pub async fn resolver_alerta(estado: State<'_, Arc<Estado>>, id: String) -> Result<bool, String> {
    db::resolver_alerta(&estado, id).await
}

#[tauri::command]
pub fn status_conexao(estado: State<'_, Arc<Estado>>) -> StatusConexao {
    estado.conectado.lock().unwrap().clone()
}

#[tauri::command]
pub fn conectar_websocket(estado: State<'_, Arc<Estado>>, app: AppHandle, url: String) -> Result<bool, String> {
    let url = url.trim().to_string();
    if url.is_empty() {
        return Err("Informe a URL do WebSocket.".into());
    }

    *estado.ws_url.lock().unwrap() = url.clone();

    if let Some(handle) = estado.ws_task.lock().unwrap().take() {
        handle.abort();
    }

    let estado_clone = estado.inner().clone();
    let app_clone = app.clone();
    let handle = tauri::async_runtime::spawn(async move {
        let app_task = app_clone.clone();
        if let Err(err) = crate::simulador::conectar_e_ouvir(app_task, estado_clone).await {
            let _ = app_clone.emit(
                "conexao:status",
                StatusConexao {
                    conectado: false,
                    ultima_tentativa_em: agora_iso(),
                },
            );
            let _ = app_clone.emit("erro:conexao", err);
        }
    });

    *estado.ws_task.lock().unwrap() = Some(handle);
    Ok(true)
}

#[tauri::command]
pub fn desconectar_websocket(estado: State<'_, Arc<Estado>>, app: AppHandle) -> bool {
    if let Some(handle) = estado.ws_task.lock().unwrap().take() {
        handle.abort();
    }

    let mut conexao = estado.conectado.lock().unwrap();
    conexao.conectado = false;
    conexao.ultima_tentativa_em = agora_iso();
    drop(conexao);

    let _ = app.emit(
        "conexao:status",
        StatusConexao {
            conectado: false,
            ultima_tentativa_em: agora_iso(),
        },
    );
    true
}
