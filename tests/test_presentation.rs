use mkrevealslides::presentation::Presentation;
use mkrevealslides::ui::conf::PresentationConfigFile;
use std::fs;
use std::fs::File;
use std::io::Write;

use mkrevealslides::ui::PresentationConfig;
use tempfile::tempdir;

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
output_file: "output.html"
template_file: "template.html"
"#;
    let mut h_cfg_file = File::create(&cfg_file).unwrap();
    h_cfg_file.write_all(cfg_str.as_bytes()).unwrap();

    let cfg_file_obj = PresentationConfigFile::read_config_file(cfg_file).unwrap();
    let cfg = PresentationConfig::try_from(cfg_file_obj).unwrap();
    let ppt = Presentation::try_from(cfg).unwrap();
    assert_eq!(ppt.title, "Test Presentation");
    assert_eq!(ppt.template, template_contents);
    assert_eq!(ppt.slides.len(), 3);
    assert_eq!(ppt.slides[0].contents, "Slide 1");
    assert_eq!(ppt.slides[1].contents, "Slide 2");
    assert_eq!(ppt.slides[2].contents, "Slide 3");
    tmp_dir.close().unwrap();
}

#[test]
fn test_presentation_from_config_with_image() {
    let tmp_dir = tempdir().unwrap();

    let slide_dir = tmp_dir.path().join("slides");
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

    let img_dir = tmp_dir.path().join("img");
    fs::create_dir(&img_dir).unwrap();

    let img_file_1 = img_dir.join("1_img1.png");
    File::create(&img_file_1).unwrap();

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
output_file: "output.html"
template_file: "template.html"
"#;
    let mut h_cfg_file = File::create(&cfg_file).unwrap();
    h_cfg_file.write_all(cfg_str.as_bytes()).unwrap();

    let cfg_file_obj = PresentationConfigFile::read_config_file(cfg_file).unwrap();
    let cfg = PresentationConfig::try_from(cfg_file_obj).unwrap();
    let mut ppt = Presentation::try_from(cfg).unwrap();

    fs::create_dir(tmp_dir.path().join("output")).unwrap();

    ppt.package(tmp_dir.path().join("output")).unwrap();
    assert_eq!(ppt.title, "Test Presentation");
    assert_eq!(ppt.template, template_contents);
    assert_eq!(ppt.slides.len(), 3);
    assert_eq!(ppt.slides[0].contents, "![](../img/1_img1.png)");
    assert_eq!(ppt.slides[1].contents, "Slide 2");
    assert_eq!(ppt.slides[2].contents, "Slide 3");
    assert!(fs::read(tmp_dir.path().join("output/img/1_img1.png")).is_ok());
    tmp_dir.close().unwrap();
}