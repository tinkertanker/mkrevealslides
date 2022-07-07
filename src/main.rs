use std::{fs};
use mkrevealslides::*;

use std::path::PathBuf;
use clap::{command, ArgAction, Arg, value_parser, ArgGroup};
use tracing::{debug, info, Level};
use mkrevealslides::conf::SlideConfig;
use mkrevealslides::error_handling::AppError;

fn main() -> Result<(), AppError> {

    let matches = command!()
        .arg(
            Arg::new("slide_dir")
                .help("The directory containing the slides")
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("output-file")
                .help("The file to output to")
                .takes_value(true)
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            Arg::new("template_file")
                .short('t')
                .long("template-file")
                .help("The template file to generate the slides from")
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
                .help("The config file to read")
                .required(false)
                .takes_value(true)
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enables verbose tracing")
                .long_help("Displays tracing information to the console. \
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
        debug!("Discovered config file: {}", conf_path.display());
        let conf_contents = fs::read_to_string(conf_path)?;
        let conf: SlideConfig = serde_yaml::from_str(&conf_contents)?;
        conf
    } else {
        info!("No config file given, using default");
        let conf: SlideConfig = SlideConfig::proc_args(matches)?;
        conf
    };

    debug!("Processed config: {:?}", slide_config);

    let slide_contents = read_files_to_string(slide_config.to_full_paths()?.as_ref())?;
    let output_content = gen_output_content(slide_config.template_file,
                                            &slide_config.title,
                                            slide_contents)?;
    fs::write(&slide_config.output_file, output_content)?;
    println!("Wrote output to {}", &slide_config.output_file.display());
    Ok(())

}
