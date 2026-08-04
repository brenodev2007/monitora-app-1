use crate::db;
use crate::estado::Estado;
use crate::models::{Alerta, ConexaoSalva, Servico, StatusConexao};
use crate::persistencia;
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
pub fn listar_conexoes(estado: State<'_, Arc<Estado>>, app: AppHandle) -> Vec<ConexaoSalva> {
    let mut conexoes = estado.conexoes_salvas.lock().unwrap();
    if conexoes.is_empty() {
        *conexoes = persistencia::carregar_conexoes(&app);
    }
    conexoes.clone()
}

#[tauri::command]
pub fn salvar_conexao(estado: State<'_, Arc<Estado>>, app: AppHandle, nome: String, url: String) -> Vec<ConexaoSalva> {
    let nome = nome.trim().to_string();
    let url = url.trim().to_string();
    if nome.is_empty() || url.is_empty() {
        return listar_conexoes(estado.clone(), app.clone());
    }

    let mut conexoes = estado.conexoes_salvas.lock().unwrap();
    let id = format!("conn-{}", chrono::Utc::now().timestamp_millis());
    conexoes.push(ConexaoSalva { id, nome, url });
    let snapshot = conexoes.clone();
    drop(conexoes);

    let _ = persistencia::salvar_conexoes(&app, &snapshot);
    snapshot
}

#[tauri::command]
pub fn atualizar_conexao(estado: State<'_, Arc<Estado>>, app: AppHandle, id: String, nome: String, url: String) -> Vec<ConexaoSalva> {
    let nome = nome.trim().to_string();
    let url = url.trim().to_string();
    if nome.is_empty() || url.is_empty() {
        return listar_conexoes(estado.clone(), app.clone());
    }

    let mut conexoes = estado.conexoes_salvas.lock().unwrap();
    if let Some(item) = conexoes.iter_mut().find(|item| item.id == id) {
        item.nome = nome;
        item.url = url;
    }
    let snapshot = conexoes.clone();
    drop(conexoes);

    let _ = persistencia::salvar_conexoes(&app, &snapshot);
    snapshot
}

#[tauri::command]
pub fn deletar_conexao(estado: State<'_, Arc<Estado>>, app: AppHandle, id: String) -> Vec<ConexaoSalva> {
    let mut conexoes = estado.conexoes_salvas.lock().unwrap();
    conexoes.retain(|item| item.id != id);
    let snapshot = conexoes.clone();
    drop(conexoes);

    let _ = persistencia::salvar_conexoes(&app, &snapshot);
    snapshot
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
