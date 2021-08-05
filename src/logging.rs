#[cfg(not(test))]
#[allow(unused_imports)]
pub use log::{debug, error, info, trace, warn};

#[cfg(test)]
#[allow(unused_imports)]
pub use std::{
    println as info, println as warn, println as debug, println as error, println as trace,
};

pub fn init() {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    std::panic::set_hook(Box::new(|info| {
        error!("panicked: {:#?}", info);
    }));
}
