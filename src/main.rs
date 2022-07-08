use std::fs;

use clap::Parser;

use mkrevealslides::error_handling::AppError;

use tracing::debug;

use mkrevealslides::presentation::Presentation;
use mkrevealslides::ui::cli::CliArgs;
use mkrevealslides::ui::PresentationConfig;

fn main() -> Result<(), AppError> {
    let cli_args = CliArgs::parse();
    tracing_subscriber::fmt()
        .with_max_level(cli_args.get_log_level())
        .init();
    let ppt_config = PresentationConfig::try_from(cli_args)?;
    let dest_path = &ppt_config.output_file.clone();
    let ppt = Presentation::try_from(ppt_config)?;
    let output_content = ppt.render()?;
    debug!("Attempting write to file: {}", dest_path.display());
    fs::write(dest_path, output_content)?;
    println!("Wrote output to {}", dest_path.display());
    Ok(())
}
