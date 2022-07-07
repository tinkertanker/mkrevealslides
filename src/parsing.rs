use pulldown_cmark::{Event, LinkType, Options, Parser, Tag};


/// Parses some markdown contents and only pulls out image links.
///
/// # Arguments
/// * `contents` - The markdown contents to parse.
///
/// # Example
/// ```rust
/// use mkrevealslides::parsing::grab_image_links;
/// let markdown = "![alt text](https://example.com/image.png)\n![img2](https://example.com/image2.png)";
/// let links = grab_image_links(markdown);
/// assert_eq!(links, vec!["https://example.com/image.png", "https://example.com/image2.png"]);
/// ```
///
/// # Returns
/// A vector of image links. It may be empty, if there are no links
pub fn grab_image_links(md_contents: &str) -> Vec<String> {
    let mut results: Vec<String> = vec![];
    let parser = Parser::new_ext(md_contents, Options::all());
    for event in parser {
        if let Event::Start(Tag::Image(link_type, url, _)) = event {
            if link_type == LinkType::Inline {
                results.push(url.to_string());
            }
        }
    }
    results

}

/// Given some links, pulls out only local links.
///
/// # Arguments
/// * `links` - A vector of links.
///
/// # Example
/// ```rust
/// use mkrevealslides::parsing::get_local_links;
/// let links = vec!["https://example.com/image.png".to_string(), "https://example.com/image2.png".to_string(), "/path/to/image.png".to_string()];
///
/// let local_links = get_local_links(links);
/// assert_eq!(local_links, vec!["/path/to/image.png".to_string()]);
/// ```
///
/// # Returns
/// A vector of local links. It may be empty, if there are no links.
pub fn get_local_links(links: Vec<String>) -> Vec<String> {
    let mut results: Vec<String> = vec![];
    for link in links {
        if !link.contains("://") {
            results.push(link.clone());
        }
    }
    results
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_grab_image_link() {
        let md_contents = r#"
foo

![](image_3.png "hello world")

bar
Group 1 will be image 2.png, and group 2 will be hello world.

The problem appears when I try to parse a link without title:

foo

![](image_2.png)
![some_image](path/to/image.png "image desc")

bar
        "#.to_string();
        let results = grab_image_links(&md_contents);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], "image_3.png");
        assert_eq!(results[1], "image_2.png");
        assert_eq!(results[2], "path/to/image.png");
    }

    #[test]
    fn test_grab_only_local_image_links() {
        let links = vec![
            "image_1.png".to_string(),
            "http://www.google.com/image.png".to_string(),
            "https://www.google.com/image.png".to_string(),
            "/some/local/image.png".to_string(),
            "assets/image.png".to_string(),
            r#"/home/noob\ user/image.png"#.to_string(),
            r#"C:\Users\BillGates\Microsoft_Logo.png"#.to_string(),
            r#"C:\Users\BillGates\Microsoft%20Logo.png"#.to_string(),
            r#"C:\Users\Bill\ Gates\Microsoft\ Logo.png"#.to_string(),
            "ftp://ftp.google.com/image.png".to_string(),
            "ftps://ftp.google.com/image.png".to_string(),
        ];
        let results = get_local_links(links);
        assert_eq!(results.len(), 7);
    }

}