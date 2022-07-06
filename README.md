# `mkrevealslides`

Automatically generates reveal.js slides given markdown files

## Roadmap

In rough order of priority

- [x] Better UI
- [ ] Support markdown images
- [ ] Superior error handling that does not involve CRASHING the program
- [ ] Configuration file support
- [ ] Support slide 1a, 1b etc.
- [ ] Generate zip file with revealJS deps
- [ ] Check if the file is actually a `.md` file
- [ ] Support revealJS animations



## Usage

For full help, call `mkrevealslides --help`


Place all your markdown files in some directory.
The slides should be named as such
`<slide number>_<whatever you want>`.md

Alternatively, they can literally just be `<slide number>.md`

A small example is shown in the `input/` directory in this repo.

Call `mkrevealslides <SLIDE_DIR>` to generate the slides. Note that
if you installed this from crates.io, you will need to download the template.

### Configuration file
You can set up a configuration file that tells `mkrevealslides`
what files to include in the presentation. Name the file
`slides.yml`, and pass it to `mkrevealslides`.

Example `slides.yaml`
```yaml
include_files:
  - "file_1.md"
  - "file_2.md"
```

Note: Include files will include files in the order they were passed in.
This overrides the slide number based file naming.