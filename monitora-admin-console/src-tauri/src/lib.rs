mod comandos;
mod db;
mod estado;
mod models;
mod persistencia;
mod simulador;

use estado::Estado;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Inicializa o logger. RUST_LOG pode ser definido no .env ou na variável de ambiente.
    // Exemplo: RUST_LOG=info  ou  RUST_LOG=admin_desktop_lib=debug,sqlx=warn
    dotenvy::dotenv().ok();

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .format_timestamp_secs()
    .init();

    log::info!("=== MONITORA+ Admin iniciando ===");

    // Lista as rotas (comandos Tauri) registradas para facilitar diagnóstico no terminal
    let rotas: &[&str] = &[
        "autenticar",
        "listar_servicos",
        "listar_alertas",
        "resolver_alerta",
        "status_conexao",
        "listar_conexoes",
        "salvar_conexao",
        "atualizar_conexao",
        "deletar_conexao",
        "conectar_websocket",
        "desconectar_websocket",
    ];

    log::info!("Rotas Tauri registradas ({} total):", rotas.len());
    for rota in rotas {
        log::info!("  ✓  {}", rota);
    }

    let estado = Arc::new(Estado::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(estado.clone())
        .setup(move |_app| {
            log::info!("Setup do Tauri concluído — janela pronta.");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            comandos::autenticar,
            comandos::listar_servicos,
            comandos::listar_alertas,
            comandos::resolver_alerta,
            comandos::status_conexao,
            comandos::listar_conexoes,
            comandos::salvar_conexao,
            comandos::atualizar_conexao,
            comandos::deletar_conexao,
            comandos::conectar_websocket,
            comandos::desconectar_websocket,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o MONITORA+ Admin");
}
