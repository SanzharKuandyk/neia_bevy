use neia_defaults::CorePlugin;
use neia_mod::ModCollection;
use neia_server::{Config, Server};
use neia_shared::ServerError;
use neia_shared::server::registry::RegistryBuilder;

#[tokio::main]
pub async fn main() -> Result<(), ServerError> {
    let (config, game_config) = Config::new()?;
    let loaded_mods = ModCollection::scan(config.args.root.join(&config.app.mods.manifests_dir))
        .map_err(|err| ServerError::InitializationError(Some(err.into())))?;

    let mut builder = RegistryBuilder::new();
    builder.add_plugin(&CorePlugin);
    let registry = builder.build();

    let server = Server::new(config, game_config, loaded_mods, registry)
        .map_err(|error| ServerError::InitializationError(Some(error.to_string().into())))?;

    server.start(true).await;
    Ok(())
}
