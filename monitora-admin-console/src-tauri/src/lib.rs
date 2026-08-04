mod comandos;
mod estado;
mod models;
mod simulador;

use estado::Estado;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let estado = Arc::new(Estado::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(estado.clone())
        .setup(move |_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            comandos::autenticar,
            comandos::listar_servicos,
            comandos::listar_alertas,
            comandos::resolver_alerta,
            comandos::status_conexao,
            comandos::conectar_websocket,
            comandos::desconectar_websocket,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o MONITORA+ Admin");
}
