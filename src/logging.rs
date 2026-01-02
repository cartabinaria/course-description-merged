use color_eyre::install;
use env_logger::{Builder, Env};
use log::error;

pub fn setup() {
    let env = Env::default().default_filter_or("info");
    Builder::from_env(env).init();
    if let Err(e) = install() {
        error!("Eyre setup: {e}");
    }
}
