use std::{fs};
use mkrevealslides::val::*;
use mkrevealslides::*;

use std::path::PathBuf;
use clap::{command, ArgAction, Arg, value_parser, ArgGroup};
use tracing::{debug, info, Level, trace, warn};
use mkrevealslides::conf::SlideConfig;
use mkrevealslides::error_handling::AppError;

fn create_output_folder_if_exists_and_warn_else(s: &str) -> Result<PathBuf, ValError> {
    let output_fp = PathBuf::from(s);
    if output_fp.exists() {
        warn!("Output file {} already exists, will overwrite", s);
    }
    if let Some(parent_dir) = output_fp.parent() {
        info!("Automatically creating output directory {}", parent_dir.display());
        create_dir_if_not_exists(parent_dir)
            .map_err(|e| format!("Could not create output directory {}: {}", parent_dir.display(), e))?;
        Ok(output_fp)
    } else {
        Err(String::from("Output file must be a valid path"))
    }
}

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

    if let Some(conf_path) = config_fp {
        let conf_contents = fs::read_to_string(conf_path)?;
        let conf: SlideConfig = serde_yaml::from_str(&conf_contents)?;
        debug!("conf: {:?}", conf);

        if let Some(include_files) = conf.include_files {
            let mut files_to_process =
            trace!("config include_files: {:?}", include_files);
            for (i, include_file) in include_files.iter().enumerate() {
                let file_path = PathBuf::from(&slide_dir).join(include_file);
                validate_file_path(file_path.to_str().expect("wat the heck"))?;
                let file_entry = FileEntry {
                    idx: i as i32,
                    file_path
                };
                files_to_process.push(file_entry);
            }
        } else {
            // todo: clean up code duplication
            let entries = fetch_file_indices(&slide_dir)?;
            let entries = indices_and_paths_to_entries(entries)?;
            files_to_process = build_proc_pq(entries);
        }
    } else {
        info!("No config file given, using default");
        let conf: SlideConfig = SlideConfig::proc_args(matches)?;
    }

    debug!("slide_dir: {:?}", conf);
    debug!("output_file: {:?}", output_file);
    debug!("template_file: {:?}", template_file);
    debug!("config_file: {:?}", config_fp);


    let presentation_title = "mkrevealslides output".to_string();
    let files_to_process = Vec::<FileEntry>::new();

    let slide_contents = read_files_to_string(files_to_process)?;
    let output_content = gen_output_content(template_file,
                                            &presentation_title,
                                            slide_contents)?;
    fs::write(output_file, output_content)?;
    Ok(())

}
