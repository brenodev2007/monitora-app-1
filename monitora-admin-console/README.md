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

### 1. Iniciar o Banco de Dados MySQL (Docker)

Certifique-se de que o **Docker Desktop** está instalado e em execução no seu sistema.

Na raiz do projeto (`monitora-admin-console`), execute:

```bash
docker compose up -d
```

> **Atenção (Windows)**: Se você receber o erro `docker : O termo 'docker' não é reconhecido...`, significa que o Docker Desktop não está instalado no sistema ou não foi adicionado às variáveis de ambiente (PATH). Baixe e instale o [Docker Desktop para Windows](https://www.docker.com/products/docker-desktop/).

Para verificar se o container MySQL está ativo e saudável:

```bash
docker compose ps
```

### 2. Executar o App Desktop (Tauri)

```bash
npm install
npm run tauri:dev
```

### 3. Testando em Outras Máquinas / Conexão Remota

Se você quiser rodar o MySQL em uma máquina (Servidor/Host) e o app Tauri em outra máquina da rede:

1. **Na máquina onde o MySQL rodará**:
   Execute `docker compose up -d`.
   Certifique-se de que a porta `3306` está aberta no Firewall do Windows/Linux.

2. **Na máquina cliente (onde o app Tauri será executado)**:
   Edite o arquivo `src-tauri/.env` alterando `DB_HOST` para o endereço IP da máquina do servidor:
   ```env
   DB_HOST="192.168.x.x"  # substitua pelo IP da máquina com MySQL
   DB_PORT="3306"
   DB_USER="root"
   DB_PASSWORD="root"
   DB_NAME="monitora"
   ```
   A aplicação Rust conta com reconexão automática e retry loop (até 10 tentativas), aguardando a disponibilidade da rede/banco.

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
