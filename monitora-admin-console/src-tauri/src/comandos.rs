use crate::estado::Estado;
use crate::models::{Alerta, Servico, StatusConexao};
use chrono::Utc;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

fn agora_iso() -> String {
    Utc::now().to_rfc3339()
}

/// Autenticação do administrador. Trocar por chamada real à API
/// (POST /admin/sessao) quando o backend estiver disponível.
#[tauri::command]
pub fn autenticar(estado: State<Arc<Estado>>, email: String, senha: String) -> Result<bool, String> {
    if email.trim().is_empty() || senha.trim().is_empty() {
        return Err("Informe e-mail e senha.".into());
    }
    *estado.autenticado.lock().unwrap() = true;
    Ok(true)
}

#[tauri::command]
pub fn listar_servicos(estado: State<Arc<Estado>>) -> Vec<Servico> {
    estado.servicos.lock().unwrap().clone()
}

#[tauri::command]
pub fn listar_alertas(estado: State<Arc<Estado>>) -> Vec<Alerta> {
    estado.alertas.lock().unwrap().clone()
}

#[tauri::command]
pub fn resolver_alerta(estado: State<Arc<Estado>>, id: String) -> bool {
    let mut alertas = estado.alertas.lock().unwrap();
    if let Some(alerta) = alertas.iter_mut().find(|a| a.id == id) {
        alerta.resolvido = true;
        true
    } else {
        false
    }
}

#[tauri::command]
pub fn status_conexao(estado: State<Arc<Estado>>) -> StatusConexao {
    estado.conectado.lock().unwrap().clone()
}

#[tauri::command]
pub fn conectar_websocket(estado: State<Arc<Estado>>, app: AppHandle, url: String) -> Result<bool, String> {
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
pub fn desconectar_websocket(estado: State<Arc<Estado>>, app: AppHandle) -> bool {
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
