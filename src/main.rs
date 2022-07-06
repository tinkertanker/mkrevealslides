use std::{fs, io};
use std::process::exit;
use tera::{Tera, Context};

/// Creates a directory if it does not
/// already exist
///
/// # Arguments
/// The path to the directory to create
///
/// # Errors
/// Returns an error if the directory could not be created (not because it already exists)
fn create_dir_if_not_exists(path: &str) -> Result<(), io::Error> {
    if fs::metadata(path).is_err() {
        fs::create_dir_all(path)?
    }
    Ok(())
}

fn main() {
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parse error(s): {}", e);
            exit(1);
        }
    };
    let mut ctx = Context::new();

    create_dir_if_not_exists("./output").expect("Could not create output directory");

    let inp_dir = fs::read_dir("input").expect("Could not read input directory");

    let mut ingested_files = Vec::<String>::new();

    for p in inp_dir {
        let p = p.expect("Directory entry could not be read");
        let ft = p.file_type().expect("Could not get file type");
        if !ft.is_file() {
            println!("Warning: Skipping {} because it is not a file", p.path().display());
            continue;
        }
        let file_name = p.path().display().to_string();
        println!("Ingesting {}", file_name);
        ingested_files.push(fs::read_to_string(file_name).expect("Could not read file"));
    }

    println!("{:?}", ingested_files.len());

    ctx.insert("slide_title", "Generated from rust");
    ctx.insert("ingested_files", &ingested_files);

    let rendered = match tera.render("slides.html", &ctx) {
        Ok(r) => r,
        Err(e) => {
            println!("Template error(s): {}", e);
            exit(1);
        }
    };
    fs::write("output/slides.html", rendered).expect("Could not write output file");
    println!("Done");
}
