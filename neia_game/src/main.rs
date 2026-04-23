// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::prelude::*;
use neia_config::{LoadedConfigs, mods_dir};
use neia_mod::ModCollection;

pub mod asset_tracking;
pub mod audio;
pub mod client;
pub mod locale;
pub mod messages;
pub mod net;
pub mod screens;
pub mod state;

use client::ClientPlugin;
use locale::LocalePlugin;
use messages::MenuPlugin;
use net::NetworkPlugin;
use screens::ScreensPlugin;
use state::{ConfigResource, LoadedModsResource, LocalProfile, Pause, Screen};

fn main() -> AppExit {
    let configs =
        LoadedConfigs::load_default().expect("failed to load configs from workspace root");
    let local_profile = LocalProfile::new(configs.app.client.default_player_name.clone());
    let loaded_mods = ModCollection::scan(mods_dir()).expect("failed to scan mods directory");

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(LocalePlugin)
        .init_state::<Screen>()
        .init_state::<Pause>()
        .insert_resource(local_profile)
        .insert_resource(ConfigResource::from_loaded(configs))
        .insert_resource(LoadedModsResource(loaded_mods))
        .add_plugins((MenuPlugin, ClientPlugin, NetworkPlugin, ScreensPlugin));

    asset_tracking::plugin(&mut app);
    audio::plugin(&mut app);

    app.run()
}
