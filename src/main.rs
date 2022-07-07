use std::{fs};

use std::path::PathBuf;
use clap::{command, ArgAction, Arg, value_parser, ArgGroup};
use tracing::{debug, info, Level};
use mkrevealslides::conf::PresentationConfig;
use mkrevealslides::error_handling::AppError;

use mkrevealslides::presentation::Presentation;

fn main() -> Result<(), AppError> {

    let matches = command!()
        .arg(
            Arg::new("slide_dir")
                .help("The directory containing the slides. Cannot be used with --conf")
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            Arg::new("output_file")
                .help("The file to output to. Cannot be used with --conf")
                .takes_value(true)
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            Arg::new("template_file")
                .help("The template file to generate the slides from. \
                Cannot be used with --conf")
                .takes_value(true)
                .value_parser(value_parser!(PathBuf))
        )
        .group(
            ArgGroup::new("gen_options")
                .required(false)
                .args(&["slide_dir", "output_file", "template_file"])
                .conflicts_with("config_file")
        )
        .arg(
            Arg::new("config_file")
                .short('c')
                .long("conf")
                .help("The config file to read. \
                 Cannot be used with slide_dir, output_file and template_file")
                .required(false)
                .takes_value(true)
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enables verbose tracing/logging")
                .long_help("Displays tracing (logging) information to the console. \
                Adding more 'v's will increase the tracing level. ")
                .action(ArgAction::Count)
        ).get_matches();
    let log_level = match matches.get_one::<u8>("verbose").expect("default 0") {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();
    let config_fp = matches.get_one::<PathBuf>("config_file");

    let slide_config = if let Some(conf_path) = config_fp {
        PresentationConfig::read_config_file(conf_path)?
    } else {
        info!("No config file given, using default");
        let conf: PresentationConfig = PresentationConfig::process_args(matches)?;
        conf
    };

    debug!("Processed config: {:?}", slide_config);
    slide_config.validate_include_paths()?;
    debug!("Generating presentation");
    let presentation = Presentation::from_config(&slide_config)?;
    let output_content = presentation.render()?;
    debug!("Attempting write to file: {}", slide_config.output_file.display());
    fs::write(&slide_config.output_file, output_content)?;
    println!("Wrote output to {}", &slide_config.output_file.display());
    Ok(())

}
