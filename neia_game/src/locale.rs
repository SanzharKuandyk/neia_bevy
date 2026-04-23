use bevy::{asset::LoadedFolder, prelude::*};
use bevy_fluent::prelude::*;
use fluent_content::Content as _;
use unic_langid::langid;

pub struct LocalePlugin;

impl Plugin for LocalePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FluentPlugin)
            .add_systems(Startup, load_locale)
            .add_systems(
                Update,
                build_localization.run_if(not(resource_exists::<Localization>)),
            );
    }
}

#[derive(Resource)]
struct LocaleFolder(Handle<LoadedFolder>);

fn load_locale(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load_folder("locale/en-US");
    commands.insert_resource(LocaleFolder(handle));
    commands.insert_resource(Locale::new(langid!("en-US")));
}

fn build_localization(
    mut commands: Commands,
    folder: Res<LocaleFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    localization_builder: LocalizationBuilder,
) {
    if loaded_folders.get(&folder.0).is_some() {
        commands.insert_resource(localization_builder.build(&folder.0));
    }
}

pub fn t(locale: Option<&Localization>, key: &str) -> String {
    locale
        .and_then(|localization| localization.content(key))
        .unwrap_or_else(|| key.to_owned())
}
