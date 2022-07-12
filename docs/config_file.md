# Using `mkrevealslides` with a config file

This is the recommended way of using `mkrevealslides`.

The config file schema is as follows

```yaml
title: "Demo Slides"
# These paths are all relative to the directory containing the config file
slide_dir: "input/"
output_file: "output/index.html"
template_file: "../../templates/slides.html"
include_files:
  # include_files are relative to the slide_dir
  - "file_1.md"
  - "file_2.md"
```

Note that `include_files` is optional. If it is left blank,
`mkrevealslides` will search in your `slide_dir`, and include
all `*.md` files found in alphabetical order