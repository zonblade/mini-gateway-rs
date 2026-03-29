# ADR-0005: CLI Domain Separation

## Date

2026-03-29

## Context

After implementing the CLI control panel (ADR-0003), the codebase had three files that grew with every new resource:

- `cli.rs` (300+ lines): All subcommand enums for every resource mixed together
- `models.rs` (150+ lines): Every API request/response type in one file
- Each command handler imported types from both

Adding a new resource (e.g. `gwrs log`) required touching cli.rs (add enum), models.rs (add types), and commands/ (add handler). Three files for one concern.

## Architecture

```mermaid
graph TD
    subgraph "Before (flat)"
        cli_old[cli.rs<br/>8 enums mixed] --> models_old[models.rs<br/>all types mixed]
        proxy_old[commands/proxy.rs] --> cli_old
        proxy_old --> models_old
        gwnode_old[commands/gwnode.rs] --> cli_old
        gwnode_old --> models_old
    end
```

```mermaid
graph TD
    subgraph "After (domain-separated)"
        cli_new[cli.rs<br/>top-level Commands only]

        subgraph proxy_dir [commands/proxy/]
            proxy_mod[mod.rs<br/>ProxyAction + handler]
            proxy_models[models.rs<br/>Proxy types]
        end

        subgraph gwnode_dir [commands/gwnode/]
            gwnode_mod[mod.rs<br/>GwnodeAction + handler]
            gwnode_models[models.rs<br/>GatewayNode]
        end

        subgraph user_dir [commands/user/]
            user_mod[mod.rs<br/>UserAction + handler]
            user_models[models.rs<br/>User types]
        end

        common[common.rs<br/>MessageResponse, IdBody, SyncResponse]

        cli_new --> proxy_mod
        cli_new --> gwnode_mod
        cli_new --> user_mod
        proxy_mod --> proxy_models
        gwnode_mod --> gwnode_models
        gwnode_mod --> common
        user_mod --> user_models
        user_mod --> common
    end
```

```mermaid
graph LR
    subgraph "Domain independence"
        proxy[proxy/] ~~~ gwnode[gwnode/]
        gwnode ~~~ gateway[gateway/]
        gateway ~~~ user[user/]
        user ~~~ cert[cert/]
        cert ~~~ sync[sync/]
    end
    domain[domain/] -->|reuses ProxyDomain| proxy
```

## Decision

Restructure into domain-separated folders where each resource owns its CLI definition, models, and handler:

```
commands/
  proxy/
    mod.rs      -- ProxyAction enum + handler
    models.rs   -- Proxy, ProxyDomain, ProxyDetail, etc.
  domain/
    mod.rs      -- DomainAction enum + handler (reuses proxy/models::ProxyDomain)
  gwnode/
    mod.rs + models.rs
  gateway/
    mod.rs + models.rs
  user/
    mod.rs + models.rs
  cert/
    mod.rs + models.rs
  sync/
    mod.rs      -- no models, uses common::SyncResponse
  init.rs, config.rs, export.rs, monitor.rs  -- unchanged (no domain models)
```

Shared types (`MessageResponse`, `IdBody`, `SyncResponse`) move to `common.rs` at the crate root. `cli.rs` shrinks to only the top-level `Cli` struct and `Commands` enum, referencing each domain's action enum.

## Consequences

### Positive

- Adding a new resource means creating one folder. No changes to existing files.
- Each domain is self-contained: CLI args, types, and logic live together.
- Domains don't cross-reference each other (except domain/ reusing proxy/models, which is a real data relationship).
- Deleting a resource means deleting a folder.

### Negative

- More files and directories.
- Imports become slightly longer (`commands::proxy::ProxyAction` vs `cli::ProxyAction`).

### Risks

- If two domains start sharing types heavily, `common.rs` could grow. Mitigated by keeping only truly shared types there.

## References

- [ADR-0002](0002-cli-architecture-redesign.md): Original module structure this supersedes
- [ADR-0003](0003-cli-control-panel.md): CRUD commands that drove the growth
