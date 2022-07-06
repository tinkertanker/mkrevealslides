use std::{fs};
use mkrevealslides::{build_proc_pq, create_dir_if_not_exists, fetch_file_indices, gen_output_content, indices_and_paths_to_entries, read_files_to_string};

use std::path::PathBuf;
use clap::{command, ArgAction, Arg};
use tracing::{debug, Level};

fn main() {

    let matches = command!()
        .arg(
            Arg::new("slide_dir")
                .help("The directory containing the slides")
                .required(true)
                .index(1)
                .validator(|s| {
                    if PathBuf::from(s).is_dir() {
                        Ok(())
                    } else {
                        Err(String::from("Slide directory must be an existing directory"))
                    }
                })
        )
        .arg(
            Arg::new("output_dir")
                .short('o')
                .long("output-dir")
                .help("The directory to output the generated files to")
                .required(false)
                .default_value("output/")
                .validator(|s| {
                    if PathBuf::from(s).is_dir() {
                        Ok(())
                    } else {
                        Err(String::from("Output directory must exist"))
                    }
                })

        )
        .arg(Arg::new("template_file")
            .short('t')
            .long("template-file")
            .help("The template file to generate the slides from")
            .required(false)
            .default_value("templates/slides.html")
            .validator(|s| {
                if PathBuf::from(s).is_file() {
                    Ok(())
                } else {
                    Err(String::from("Template file must be a file"))
                }
            })
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enables verbose tracing")
                .action(ArgAction::Count)
        ).get_matches();
    let log_level = match matches.get_one::<u8>("verbose").expect("default 0") {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();
    debug!("slide_dir: {:?}", matches.value_of("slide_dir").unwrap());
    debug!("output_dir: {:?}", matches.value_of("output_dir").unwrap());
    debug!("template_file: {:?}", matches.value_of("template_file").unwrap());
    debug!("verbose: {:?}", matches.get_one::<u8>("verbose").expect("verbose defaults to 0"));

    create_dir_if_not_exists("output/").expect("Could not create output directory");

    let entries = fetch_file_indices(matches.value_of("slide_dir").unwrap()).expect("Could not read slide directory");
    let entries = indices_and_paths_to_entries(entries).expect("Could not parse indices and paths");
    let files_to_proc = build_proc_pq(entries);
    let file_contents = read_files_to_string(files_to_proc).expect("Could not read files");
    let output_content = gen_output_content(matches.value_of("template_file").unwrap(), file_contents).expect("Could not generate html");
    fs::write(format!("{}/index.html", matches.value_of("output_dir").unwrap()), output_content).expect("Could not write output file");

}
