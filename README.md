# Neia Bevy

A multiplayer game template with shared config loading, lightweight mod manifests, a native client transport crate, an authoritative server, and a minimal Bevy app crate.
This is the Bevy-oriented template variant of [neia](https://github.com/SanzharKuandyk/neia).
It is not finished yet. The current goal is to provide a small but clean base that shows the architecture and main flow, not a production-ready game shell.
Mostly AI-assisted with my view on things and how they should work, reviewed by me.

> **Bevy template is unfinished the most neia_game/

Built on [renet](https://github.com/lucaspoffo/renet) + [axum](https://github.com/tokio-rs/axum) + [Bevy](https://bevyengine.org/).

## Crate structure

```text
neia_shared/   - shared types, config schemas, messages, deltas, registry contracts
neia_config/   - config loading and path discovery from configs/
neia_mod/      - lightweight mod manifest scanning
neia_client/   - native renet client transport wrapper
neia_defaults/ - built-in core plugin with lobby messages and deltas
neia_logic/    - your game logic
neia_server/   - authoritative server
neia_game/     - Bevy app shell, screens, minimal menu flow, client/net split
```

## Configs

Runtime config lives under `configs/`:

- `configs/app.toml` for server, client, network, lobby, HTTP, shutdown, and mod settings
- `configs/game.toml` for game rules

Both client and server read from the same config source instead of duplicating transport settings.
`neia_config` also owns path discovery, so config and mods can be found even when the process starts from a crate directory instead of the workspace root.

## Mods

`neia_mod` scans `mods/*/mod.toml` and exposes loaded manifest metadata. The template does not do dynamic loading yet, but it gives you a clean extension point and a shared location for mod resources.

## neia_game

`neia_game` is intentionally small. Right now it includes:

- a `client/` layer for app-facing intent flow
- a `net/` layer for transport-facing polling and apply
- a minimal locale/audio/asset-tracking setup
- simple title, settings, lobby, gameplay, and pause flow
- clickable Bevy UI buttons for the basic menu screens

The current app flow is roughly:

```text
input/UI -> ClientIntent -> route -> neia_client send -> neia_server -> deltas -> neia_client poll -> neia_game apply -> replica/resources -> screen updates
```

This part is still incomplete on purpose. It is a starting point for a Bevy client architecture, not a finished frontend framework.

## Quick start

```bash
cargo run -p neia_server --bin neia_server_bin
cargo run -p neia_game
```

The server reads config from the workspace root by default. You can override the server root with:

```bash
cargo run -p neia_server --bin neia_server_bin -- --root path/to/project
```

## Current controls

- Title screen: `Connect`, `Settings`, `Quit` buttons, plus keyboard shortcuts
- Lobby: `Toggle Ready`, `Start Game`, `Settings`, `Disconnect`
- Gameplay: `Esc` opens pause
- Pause: `Continue`, `Settings`, `Quit to Title`

## Current limitations

- `neia_game` is still very small and intentionally unfinished
- the UI layer is minimal and not yet a reusable design system
- settings are mostly example-only right now
- gameplay-side client flow only demonstrates a narrow built-in lobby/game start path
- mod loading is manifest-level only; there is no dynamic code/content loading yet

## What to extend first

- replace the placeholder screens with your own real game UI
- add your own game messages and deltas in `neia_defaults` or your own plugin crate
- grow the inbound apply path in `neia_game::net`
- add your own gameplay resources and world rendering in `neia_game`
- decide how far you want `neia_mod` to go beyond manifest scanning
