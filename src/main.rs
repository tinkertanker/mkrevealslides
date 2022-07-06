use std::{fs};
use mkrevealslides::val::*;
use mkrevealslides::*;

use std::path::PathBuf;
use clap::{command, ArgAction, Arg};
use tracing::{debug, info, Level, trace, warn};
use mkrevealslides::conf::SlideConfig;
use mkrevealslides::error_handling::AppError;

fn main() -> Result<(), AppError> {

    let matches = command!()
        .arg(
            Arg::new("slide_dir")
                .help("The directory containing the slides")
                .required(true)

                .validator(validate_dir_path)
        )
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("output-file")
                .help("The file to output to")
                .required(false)
                .default_value("output/index.html")
                .validator(|s| {
                    let p = PathBuf::from(s);
                    if p.exists() {
                        warn!("Output file {} already exists, will overwrite", s);
                    }
                    if let Some(parent_dir) = p.parent() {
                        info!("Automatically creating output directory {}", parent_dir.display());
                        create_dir_if_not_exists(parent_dir)
                            .map_err(|e| format!("Could not create output directory {}: {}", parent_dir.display(), e))
                    } else {
                        Err(String::from("Output file must be a valid path"))
                    }
                })

        )
        .arg(Arg::new("template_file")
            .short('t')
            .long("template-file")
            .help("The template file to generate the slides from")
            .required(false)
            .default_value("templates/slides.html")
            .validator(validate_file_path)
        )
        .arg(Arg::new("config_file")
            .short('c')
            .long("conf")
            .help("The config file to read")
            .required(false)
            .takes_value(true)
            .validator(validate_file_path)
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
    let slide_dir = matches.value_of("slide_dir").unwrap();
    let output_file = matches.value_of("output_file").unwrap();
    let template_file = matches.value_of("template_file").unwrap();
    let config_file = matches.value_of("config_file");

    debug!("slide_dir: {:?}", slide_dir);
    debug!("output_file: {:?}", output_file);
    debug!("template_file: {:?}", template_file);
    debug!("config_file: {:?}", config_file);

    let mut presentation_title = "mkrevealslides output".to_string();
    let mut files_to_process = Vec::<FileEntry>::new();

    if let Some(conf_path) = config_file {
        // Only config if a config file is given
        let conf_contents = fs::read_to_string(conf_path)?;
        let conf: SlideConfig = serde_yaml::from_str(&conf_contents)?;
        debug!("conf: {:?}", conf);
        if let Some(title) = conf.title {
            debug!("title: {:?}", title);
            presentation_title = title;
        }

        if let Some(include_files) = conf.include_files {
            debug!("include_files: {:?}", include_files);
            for (i, include_file) in include_files.iter().enumerate() {
                validate_file_path(include_file)?;
                let file_path = PathBuf::from(include_file);
                let file_entry = FileEntry {
                    idx: i as i32,
                    file_path
                };
                files_to_process.push(file_entry);
            }
        } else {
            // todo: clean up code duplication
            let entries = fetch_file_indices(slide_dir)?;
            let entries = indices_and_paths_to_entries(entries)?;
            files_to_process = build_proc_pq(entries);
        }

    } else {
        trace!("No config file given, using default");
        // Process as per normal
        let entries = fetch_file_indices(slide_dir)?;
        let entries = indices_and_paths_to_entries(entries)?;
        files_to_process = build_proc_pq(entries);

    }
    let slide_contents = read_files_to_string(files_to_process)?;
    let output_content = gen_output_content(template_file,
                                            &presentation_title,
                                            slide_contents)?;
    fs::write(output_file, output_content)?;
    Ok(())

}
