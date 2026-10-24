# Architecture overview

Oracle Edit separates the document engine from presentation and transport. The desktop UI, CLI, JSON control channel, and MCP server reach editing behavior through the command registry rather than maintaining separate implementations.

```text
desktop UI       CLI       JSON control       MCP
    |             |              |              |
    +-------------+--------------+--------------+
                          |
                   command registry
                          |
                    engine Session
                          |
       document model + operations + compositors
                          |
             PSD / raster / .oracle-edit I/O
```

The enforced dependency direction runs from format-independent foundation crates through the document and operation layers to composition, I/O, the engine, and finally UI/automation. `cargo xtask layers` checks this rule.

The complete design narrative and dependency diagram remain in [`docs/architecture.md`](https://github.com/Marvingxx/oracle-edit/blob/main/docs/architecture.md). That document contains historical design material as well as implemented architecture; confirm behavior in current source when security or compatibility depends on it.

## Core properties

- `oracle-edit-doc` holds the pure-data document model.
- `oracle-edit-engine::Session` owns open documents, active state, history, and command dispatch.
- `oracle-edit-compose` is the CPU reference compositor; `oracle-edit-gpu` accelerates supported interactive paths.
- `oracle-edit-io` maps between documents and PSD or flat formats.
- `oracle-edit-format` reads and writes the native `.oracle-edit` bundle.
- `oracle-edit-ui-egui` is a thin shell over engine commands.
- `oracle-edit-automation` exposes the headless engine and the live-GUI bridge.

See [Crates and dependency layers](crates.md) for the complete workspace map.
