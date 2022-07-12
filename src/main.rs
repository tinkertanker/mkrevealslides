use clap::Parser;

use mkrevealslides::presentation::Presentation;
use mkrevealslides::ui::cli::CliArgs;
use mkrevealslides::ui::PresentationConfig;

fn main() -> Result<(), anyhow::Error> {
    let cli_args = CliArgs::parse();
    tracing_subscriber::fmt()
        .with_max_level(cli_args.get_log_level())
        .init();
    let ppt_config = PresentationConfig::try_from(cli_args)?;
    let dest_path = &ppt_config.output_file.clone();
    let mut ppt = Presentation::try_from(ppt_config)?;
    ppt.package(dest_path.parent().unwrap())?;
    Ok(())
}
