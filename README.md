# `mkrevealslides`

[![Rust](https://github.com/tinkertanker/mkrevealslides/actions/workflows/rust.yml/badge.svg)](https://github.com/tinkertanker/mkrevealslides/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/tinkertanker/mkrevealslides/branch/master/graph/badge.svg?token=YZ12W93CCX)](https://codecov.io/gh/tinkertanker/mkrevealslides)

Automatically generates reveal.js slides given markdown files

## Roadmap

In rough order of priority

- [x] Better UI
- [x] Check if the file is actually a `.md` file
- [x] Superior error handling that does not involve CRASHING the program
  - [ ] More helpful error messages 
- [x] Configuration file support
    - [ ] Force paths relative to config file dir instead of invocation dir
- [ ] Support markdown images
- [ ] Support slide 1a, 1b etc.
- [ ] Generate zip file with revealJS deps
- [ ] Automatically download revealJS if not present 
- [ ] Support revealJS animations
- [ ] Tests
    - [x] Unit tests
    - [ ] Integration tests


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
title: Lesson 1
include_files:
  - "file_1.md"
  - "file_2.md"
```

Note: Include files will include files in the order they were passed in.
This overrides the slide number based file naming.