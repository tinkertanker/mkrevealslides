use std::{fs, io};
use std::process::exit;
use tera::{Tera, Context};
use mkrevealslides::{create_dir_if_not_exists, FileEntry};


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

    let mut files_to_process = std::collections::BinaryHeap::<FileEntry>::new();

    for p in inp_dir {
        let p = p.expect("Directory entry could not be read");
        let ft = p.file_type().expect("Could not get file type");
        if !ft.is_file() {
            println!("Warning: Skipping {} because it is not a file", p.path().display());
            continue;
        }
        // TODO: check if the file ends in .md at least
        let path = p.path();
        let read_path = path.display().to_string();
        let file_name = path.file_stem().expect("Could not get file name").to_str().expect("Could not get file name as string");

        let fp_splice = file_name.split("_");

        let f_num = fp_splice.collect::<Vec<&str>>();
        let f_num = f_num.first().expect("Could not get file number");
        let f_num = f_num.parse::<i32>().expect("Could not parse file number");

        // Hack to make a min heap
        files_to_process.push(FileEntry {
            idx: -f_num,
            file_path: read_path
        });
    }

    let mut ingested_files = Vec::<String>::new();
    while !files_to_process.is_empty() {
        let f = files_to_process.pop().expect("Could not pop file from heap");
        let f = f.file_path;
        println!("Processing {}", f);
        let f = fs::read_to_string(f).expect("Could not read file");
        ingested_files.push(f);
    }


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
