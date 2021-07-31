use tav::config::Config;
use tav::logging;
use tav::run::run;

fn main() {
    logging::init();
    run(Config::load());
}
