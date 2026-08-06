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
    log::info!("[CMD] autenticar chamado (email={})", email.trim());
    let resultado = db::autenticar(&estado, email, senha).await;
    match &resultado {
        Ok(ok) => log::info!("[CMD] autenticar → resultado: {}", if *ok { "SUCESSO" } else { "FALHA (credenciais inválidas)" }),
        Err(err) => log::error!("[CMD] autenticar → erro: {}", err),
    }
    resultado
}

#[tauri::command]
pub async fn listar_servicos(estado: State<'_, Arc<Estado>>) -> Result<Vec<Servico>, String> {
    log::info!("[CMD] listar_servicos chamado");
    let resultado = db::listar_servicos(&estado).await;
    match &resultado {
        Ok(lista) => log::info!("[CMD] listar_servicos → {} serviço(s) retornado(s)", lista.len()),
        Err(err) => log::error!("[CMD] listar_servicos → erro: {}", err),
    }
    resultado
}

#[tauri::command]
pub async fn listar_alertas(estado: State<'_, Arc<Estado>>) -> Result<Vec<Alerta>, String> {
    log::info!("[CMD] listar_alertas chamado");
    let resultado = db::listar_alertas(&estado).await;
    match &resultado {
        Ok(lista) => log::info!("[CMD] listar_alertas → {} alerta(s) retornado(s)", lista.len()),
        Err(err) => log::error!("[CMD] listar_alertas → erro: {}", err),
    }
    resultado
}

#[tauri::command]
pub async fn resolver_alerta(estado: State<'_, Arc<Estado>>, id: String) -> Result<bool, String> {
    log::info!("[CMD] resolver_alerta chamado (id={})", id);
    let resultado = db::resolver_alerta(&estado, id).await;
    match &resultado {
        Ok(ok) => log::info!("[CMD] resolver_alerta → {}", if *ok { "resolvido" } else { "não encontrado" }),
        Err(err) => log::error!("[CMD] resolver_alerta → erro: {}", err),
    }
    resultado
}

#[tauri::command]
pub fn status_conexao(estado: State<'_, Arc<Estado>>) -> StatusConexao {
    let status = estado.conectado.lock().unwrap().clone();
    log::debug!("[CMD] status_conexao → conectado={}", status.conectado);
    status
}

#[tauri::command]
pub fn listar_conexoes(estado: State<'_, Arc<Estado>>, app: AppHandle) -> Vec<ConexaoSalva> {
    log::info!("[CMD] listar_conexoes chamado");
    let mut conexoes = estado.conexoes_salvas.lock().unwrap();
    if conexoes.is_empty() {
        log::info!("[CMD] listar_conexoes → cache vazio, carregando do disco...");
        *conexoes = persistencia::carregar_conexoes(&app);
    }
    let lista = conexoes.clone();
    log::info!("[CMD] listar_conexoes → {} conexão(ões) encontrada(s)", lista.len());
    lista
}

#[tauri::command]
pub fn salvar_conexao(estado: State<'_, Arc<Estado>>, app: AppHandle, nome: String, url: String) -> Vec<ConexaoSalva> {
    let nome = nome.trim().to_string();
    let url = url.trim().to_string();
    log::info!("[CMD] salvar_conexao chamado (nome='{}', url='{}')", nome, url);

    if nome.is_empty() || url.is_empty() {
        log::warn!("[CMD] salvar_conexao → nome ou URL em branco, ignorando.");
        return listar_conexoes(estado.clone(), app.clone());
    }

    let mut conexoes = estado.conexoes_salvas.lock().unwrap();
    let id = format!("conn-{}", chrono::Utc::now().timestamp_millis());
    conexoes.push(ConexaoSalva { id: id.clone(), nome: nome.clone(), url: url.clone() });
    let snapshot = conexoes.clone();
    drop(conexoes);

    match persistencia::salvar_conexoes(&app, &snapshot) {
        Ok(_) => log::info!("[CMD] salvar_conexao → conexão '{}' salva (id={})", nome, id),
        Err(err) => log::error!("[CMD] salvar_conexao → erro ao persistir: {}", err),
    }
    snapshot
}

#[tauri::command]
pub fn atualizar_conexao(estado: State<'_, Arc<Estado>>, app: AppHandle, id: String, nome: String, url: String) -> Vec<ConexaoSalva> {
    let nome = nome.trim().to_string();
    let url = url.trim().to_string();
    log::info!("[CMD] atualizar_conexao chamado (id='{}', nome='{}', url='{}')", id, nome, url);

    if nome.is_empty() || url.is_empty() {
        log::warn!("[CMD] atualizar_conexao → nome ou URL em branco, ignorando.");
        return listar_conexoes(estado.clone(), app.clone());
    }

    let mut conexoes = estado.conexoes_salvas.lock().unwrap();
    let encontrada = conexoes.iter_mut().find(|item| item.id == id);
    if let Some(item) = encontrada {
        item.nome = nome.clone();
        item.url = url.clone();
        log::info!("[CMD] atualizar_conexao → conexão '{}' atualizada.", id);
    } else {
        log::warn!("[CMD] atualizar_conexao → conexão '{}' não encontrada.", id);
    }
    let snapshot = conexoes.clone();
    drop(conexoes);

    match persistencia::salvar_conexoes(&app, &snapshot) {
        Ok(_) => log::debug!("[CMD] atualizar_conexao → persitência OK."),
        Err(err) => log::error!("[CMD] atualizar_conexao → erro ao persistir: {}", err),
    }
    snapshot
}

#[tauri::command]
pub fn deletar_conexao(estado: State<'_, Arc<Estado>>, app: AppHandle, id: String) -> Vec<ConexaoSalva> {
    log::info!("[CMD] deletar_conexao chamado (id='{}')", id);
    let mut conexoes = estado.conexoes_salvas.lock().unwrap();
    let antes = conexoes.len();
    conexoes.retain(|item| item.id != id);
    let depois = conexoes.len();
    let snapshot = conexoes.clone();
    drop(conexoes);

    if antes != depois {
        log::info!("[CMD] deletar_conexao → conexão '{}' removida.", id);
    } else {
        log::warn!("[CMD] deletar_conexao → conexão '{}' não encontrada.", id);
    }

    match persistencia::salvar_conexoes(&app, &snapshot) {
        Ok(_) => log::debug!("[CMD] deletar_conexao → persistência OK."),
        Err(err) => log::error!("[CMD] deletar_conexao → erro ao persistir: {}", err),
    }
    snapshot
}

#[tauri::command]
pub fn conectar_websocket(estado: State<'_, Arc<Estado>>, app: AppHandle, url: String) -> Result<bool, String> {
    let url = url.trim().to_string();
    log::info!("[CMD] conectar_websocket chamado (url='{}')", url);

    if url.is_empty() {
        log::warn!("[CMD] conectar_websocket → URL vazia, rejeitado.");
        return Err("Informe a URL do WebSocket.".into());
    }

    *estado.ws_url.lock().unwrap() = url.clone();

    if let Some(handle) = estado.ws_task.lock().unwrap().take() {
        log::info!("[CMD] conectar_websocket → abortando tarefa WebSocket anterior...");
        handle.abort();
    }

    let estado_clone = estado.inner().clone();
    let app_clone = app.clone();
    let handle = tauri::async_runtime::spawn(async move {
        log::info!("[WS] Iniciando tarefa de conexão WebSocket...");
        let app_task = app_clone.clone();
        if let Err(err) = crate::simulador::conectar_e_ouvir(app_task, estado_clone).await {
            log::error!("[WS] Tarefa WebSocket encerrada com erro: {}", err);
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
    log::info!("[CMD] conectar_websocket → tarefa iniciada para '{}'", url);
    Ok(true)
}

#[tauri::command]
pub fn desconectar_websocket(estado: State<'_, Arc<Estado>>, app: AppHandle) -> bool {
    log::info!("[CMD] desconectar_websocket chamado");

    if let Some(handle) = estado.ws_task.lock().unwrap().take() {
        handle.abort();
        log::info!("[CMD] desconectar_websocket → tarefa WebSocket abortada.");
    } else {
        log::warn!("[CMD] desconectar_websocket → nenhuma tarefa ativa encontrada.");
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
    log::info!("[CMD] desconectar_websocket → status emitido (desconectado).");
    true
}
