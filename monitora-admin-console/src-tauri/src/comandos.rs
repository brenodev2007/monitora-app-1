use crate::estado::Estado;
use crate::models::{Alerta, Servico};
use std::sync::Arc;
use tauri::State;

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
pub fn status_conexao(estado: State<Arc<Estado>>) -> bool {
    *estado.conectado.lock().unwrap()
}
