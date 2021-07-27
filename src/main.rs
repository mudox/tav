use tav::config::Config;
use tav::run::run;

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();

    run(Config::load());
}
