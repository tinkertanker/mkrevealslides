# `mkrevealslides`

Automatically generates reveal.js slides given markdown files

## Roadmap

In rough order of priority

- [ ] Better UI
- [ ] Superior error handling that does not involve CRASHING the program
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