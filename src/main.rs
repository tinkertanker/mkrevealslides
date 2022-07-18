use clap::Parser;
use mkrevealslides::presentation::PresentationConfig;

use mkrevealslides::ui::cli::CliArgs;

fn main() -> Result<(), anyhow::Error> {
    let cli_args = CliArgs::parse();
    tracing_subscriber::fmt()
        .with_max_level(cli_args.get_log_level())
        .init();
    let ppt_config = PresentationConfig::try_from(cli_args)?;
    ppt_config.package()?;
    Ok(())
}
