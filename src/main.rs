use clap::Parser;
use ripdiff::app::App;

fn main() -> anyhow::Result<()> {
    simple_logger::init().expect("Failed to initialize logger");
    log::set_max_level(log::LevelFilter::Info);
    App::parse().run()
}
