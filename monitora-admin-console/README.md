# MONITORA+ — Painel Administrativo (Tauri + React)

Implementa a seção **6.4** da especificação: o app desktop usado pelo
administrador do MONITORA+ para acompanhar, em tempo real, todos os
clientes e serviços monitorados (API, servidor, backup) e os alertas
disparados.

## O que está pronto

- **Tauri v2 + React 19 + TypeScript**, com Tailwind v4 e Recharts.
- **Toda a "conexão em tempo real" fica no processo principal (Rust)**,
  nunca na interface — o mesmo desenho descrito na spec para evitar o
  bug de duplicação de conexão já resolvido no `ti-chamados`
  (`socket-bridge.ts` → aqui, `estado.rs` + `simulador.rs`).
- A UI (`src/`) só conversa com o Rust por **commands** e **eventos**,
  via `src/lib/bridge.ts` — o equivalente ao `preload/bridge.ts` da spec.
- Tela de login, lista de serviços com status (`ok` / `atraso` / `falha`),
  painel de detalhe com gráfico de histórico (Recharts) e painel de
  alertas por canal (e-mail / WhatsApp / Discord), com notificação nativa
  do sistema operacional ao disparar um alerta.
- Modelo de dados em Rust (`src-tauri/src/models.rs`) espelha as tabelas
  `servicos`, `heartbeats` e `alertas` da seção 7 da especificação.

## O que é simulado (e por quê)

Não existe ainda API/worker/banco de dados reais rodando — a spec descreve
5 apps no monorepo (agente, api, worker, admin-desktop, dashboard) e este
entrega o admin-desktop. Por isso `src-tauri/src/simulador.rs` faz, dentro
do próprio processo principal, o papel do agente + worker + WebSocket:
gera heartbeats a cada ~4s, decide status e dispara alertas — para o
painel já nascer funcional e demonstrável.

**Para plugar o backend real**, troque o conteúdo de `simulador::iniciar`
por uma conexão WebSocket de verdade (crate `tokio-tungstenite`, por
exemplo) para `wss://<sua-api>/admin`, mantendo os mesmos `app.emit(...)`
que já existem — a UI não muda nada, porque ela só ouve eventos.

## Como rodar

```bash
npm install
npm run tauri:dev
```

Requer o [ambiente do Tauri](https://tauri.app/start/prerequisites/)
instalado (Rust + dependências do sistema — no Windows, o instalador do
Rust já resolve a maior parte; no Linux, `webkit2gtk` e afins).

Build de produção:

```bash
npm run tauri:build
```

## Estrutura

```
src-tauri/
  src/
    models.rs      # espelha usuarios/servicos/heartbeats/alertas (seção 7)
    estado.rs       # estado central em memória, thread-safe
    simulador.rs    # hoje: mock de agente+worker. depois: WS real
    comandos.rs      # commands invocáveis pela UI (invoke)
    lib.rs / main.rs
src/
  lib/bridge.ts     # único ponto de contato com o processo principal
  hooks/useMonitora.ts  # cache do último estado conhecido no renderer
  components/       # TelaLogin, ListaServicos, PainelServico, PainelAlertas
```

## Ícones

Os ícones em `src-tauri/icons/` são placeholders gerados localmente —
troque pelo ícone real do MONITORA+ antes de gerar um build de distribuição.
