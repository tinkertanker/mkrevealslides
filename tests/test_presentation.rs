use mkrevealslides::ui::conf::PresentationConfigFile;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use tempfile::tempdir;
use mkrevealslides::presentation::PresentationConfig;

#[test]
fn test_presentation_from_config() {
    let tmp_dir = tempdir().unwrap();

    let slide_dir = tmp_dir.path().join("slides");
    fs::create_dir(&slide_dir).unwrap();

    let slide_file_1 = slide_dir.join("1_slide1.md");
    let mut h_slide_file_1 = File::create(&slide_file_1).unwrap();
    h_slide_file_1.write_all(b"Slide 1").unwrap();
    let slide_file_2 = slide_dir.join("2_slide2.md");
    let mut h_slide_file_2 = File::create(&slide_file_2).unwrap();
    h_slide_file_2.write_all(b"Slide 2").unwrap();
    let slide_file_3 = slide_dir.join("3_slide3.md");
    let mut h_slide_file_3 = File::create(&slide_file_3).unwrap();
    h_slide_file_3.write_all(b"Slide 3").unwrap();

    let _output_file = tmp_dir.path().join("output.html");

    let template_contents = "{{ slide_title }} {%for fc in ingested_files %}'{{fc}}'{%endfor%}";
    let template_file = tmp_dir.path().join("template.html");
    let mut h_template_file = File::create(&template_file).unwrap();
    h_template_file
        .write_all(template_contents.as_bytes())
        .unwrap();

    let cfg_file = tmp_dir.path().join("config.yaml");
    let cfg_str = r#"
title: "Test Presentation"
slide_dir: "slides"
output_dir: "output"
output_file: "output.html"
template_file: "template.html"
"#;
    let mut h_cfg_file = File::create(&cfg_file).unwrap();
    h_cfg_file.write_all(cfg_str.as_bytes()).unwrap();

    let cfg_file_obj = PresentationConfigFile::read_config_file(cfg_file).unwrap();
    let cfg = PresentationConfig::try_from(cfg_file_obj).unwrap();
    cfg.package().expect("package to succeed");

    tmp_dir.close().unwrap();
}

#[test]
fn test_presentation_from_config_with_image() {
    let tmp_dir = tempdir().unwrap();

    let tmp_dir_pth = fs::canonicalize(tmp_dir.path()).expect("temp dir exists");

    let slide_dir = tmp_dir_pth.join("slides");
    fs::create_dir(&slide_dir).unwrap();

    let slide_file_1 = slide_dir.join("1_slide1.md");
    let mut h_slide_file_1 = File::create(&slide_file_1).unwrap();
    h_slide_file_1.write_all(b"![](../img/1_img1.png)").unwrap();
    let slide_file_2 = slide_dir.join("2_slide2.md");
    let mut h_slide_file_2 = File::create(&slide_file_2).unwrap();
    h_slide_file_2.write_all(b"Slide 2").unwrap();
    let slide_file_3 = slide_dir.join("3_slide3.md");
    let mut h_slide_file_3 = File::create(&slide_file_3).unwrap();
    h_slide_file_3.write_all(b"Slide 3").unwrap();

    let img_dir = tmp_dir_pth.join("img");
    fs::create_dir(&img_dir).unwrap();

    let img_file_1 = img_dir.join("1_img1.png");
    File::create(&img_file_1).unwrap();

    let _output_file = tmp_dir_pth.join("output.html");

    let template_contents = "{{ slide_title }} {%for fc in ingested_files %}'{{fc}}'{%endfor%}";
    let template_file = tmp_dir_pth.join("template.html");
    let mut h_template_file = File::create(&template_file).unwrap();
    h_template_file
        .write_all(template_contents.as_bytes())
        .unwrap();

    let cfg_file = tmp_dir_pth.join("config.yaml");
    let cfg_str = r#"
title: "Test Presentation"
slide_dir: "slides"
output_dir: "output"
output_file: "output.html"
template_file: "template.html"
"#;
    let mut h_cfg_file = File::create(&cfg_file).unwrap();
    h_cfg_file.write_all(cfg_str.as_bytes()).unwrap();

    let cfg_file_obj = PresentationConfigFile::read_config_file(cfg_file).unwrap();
    let cfg = PresentationConfig::try_from(cfg_file_obj).unwrap();

    cfg.package().expect("package to succeed");

    assert!(fs::read(tmp_dir_pth.join("output/img/1_slide1.md/1_img1.png")).is_ok());
    tmp_dir.close().unwrap();
}

#[test]
fn test_presentation_from_config_with_image_in_subdirectory() {
    let tmp_dir = tempdir().unwrap();

    let slide_dir = tmp_dir.path().join("slides");
    fs::create_dir(&slide_dir).unwrap();

    let slide_file_1 = slide_dir.join("1_slide1.md");
    let mut h_slide_file_1 = File::create(&slide_file_1).unwrap();
    h_slide_file_1
        .write_all(b"![](../img/slide1/img1.png)")
        .unwrap();
    let slide_file_2 = slide_dir.join("2_slide2.md");
    let mut h_slide_file_2 = File::create(&slide_file_2).unwrap();
    h_slide_file_2
        .write_all(b"![](../img/slide2/a/img2.png)")
        .unwrap();
    let slide_file_3 = slide_dir.join("3_slide3.md");
    let mut h_slide_file_3 = File::create(&slide_file_3).unwrap();
    h_slide_file_3
        .write_all(b"![](../img/slide3/img3.png)")
        .unwrap();

    let img_dir = tmp_dir.path().join("img");
    fs::create_dir(&img_dir).unwrap();

    let img_file_1 = img_dir.join(PathBuf::from("slide1/img1.png"));
    fs::create_dir_all(&img_file_1.parent().unwrap()).unwrap();
    File::create(&img_file_1).unwrap();

    let img_file_2 = img_dir.join(PathBuf::from("slide2/a/img2.png"));
    fs::create_dir_all(&img_file_2.parent().unwrap()).unwrap();
    File::create(&img_file_2).unwrap();

    let img_file_3 = img_dir.join(PathBuf::from("slide3/img3.png"));
    fs::create_dir_all(&img_file_3.parent().unwrap()).unwrap();
    File::create(&img_file_3).unwrap();

    let _output_file = tmp_dir.path().join("output.html");

    let template_contents = "{{ slide_title }} {%for fc in ingested_files %}'{{fc}}'{%endfor%}";
    let template_file = tmp_dir.path().join("template.html");
    let mut h_template_file = File::create(&template_file).unwrap();
    h_template_file
        .write_all(template_contents.as_bytes())
        .unwrap();

    let cfg_file = tmp_dir.path().join("config.yaml");
    let cfg_str = r#"
title: "Test Presentation"
slide_dir: "slides"
output_dir: "output"
output_file: "output.html"
template_file: "template.html"
"#;
    let mut h_cfg_file = File::create(&cfg_file).unwrap();
    h_cfg_file.write_all(cfg_str.as_bytes()).unwrap();

    let cfg_file_obj = PresentationConfigFile::read_config_file(cfg_file).unwrap();
    let cfg = PresentationConfig::try_from(cfg_file_obj).unwrap();
    cfg.package().expect("package to succeed");

    assert!(fs::read(tmp_dir.path().join("output/img/1_slide1.md/img1.png")).is_ok());
    assert!(fs::read(tmp_dir.path().join("output/img/2_slide2.md/img2.png")).is_ok());
    assert!(fs::read(tmp_dir.path().join("output/img/3_slide3.md/img3.png")).is_ok());
    tmp_dir.close().unwrap();
}
