# `mkrevealslides` documentation


## Basic usage

Place all your markdown files in some directory.
The order they will be presented is in alphabetical order.

For example.

```
1a.md
1b.md
1c.md
2a.md
```
etc. etc.

You may see examples in the `examples/` directory.

Make sure you have a suitable template to generate
the slides from. A simple template is provided in `templates/`

After doing so, run the following command:

`mkrevealslides from-cli <SLIDE_DIR> <TEMPLATE_FILE> <OUTPUT_DIR>`.

This will generate the slides and place them in `output_dir`/


