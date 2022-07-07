use mkrevealslides::conf::PresentationConfig;
use mkrevealslides::presentation::Presentation;
use std::fs;
use std::fs::File;
use std::io::Write;
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

    let output_file = tmp_dir.path().join("output.html");

    let template_contents = "{{ slide_title }} {%for fc in ingested_files %}'{{fc}}'{%endfor%}";
    let template_file = tmp_dir.path().join("template.html");
    let mut h_template_file = File::create(&template_file).unwrap();
    h_template_file
        .write_all(template_contents.as_bytes())
        .unwrap();

    let cfg = PresentationConfig {
        title: "Test Presentation".to_string(),
        slide_dir,
        output_file,
        template_file,
        include_files: Some(vec![
            "1_slide1.md".to_string(),
            "2_slide2.md".to_string(),
            "3_slide3.md".to_string(),
        ]),
    };

    let ppt = Presentation::from_config(&cfg).unwrap();
    assert_eq!(ppt.title, "Test Presentation");
    assert_eq!(ppt.template, template_contents);
    assert_eq!(ppt.slides.len(), 3);
    assert_eq!(ppt.slides[0].contents, "Slide 1");
    assert_eq!(ppt.slides[1].contents, "Slide 2");
    assert_eq!(ppt.slides[2].contents, "Slide 3");
}
