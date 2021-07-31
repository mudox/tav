use log::error;

pub fn init() {
    std::env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();

    std::panic::set_hook(Box::new(|info| {
        error!("panicked: {:#?}", info);
    }));
}
