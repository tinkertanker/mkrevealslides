# `mkrevealslides`

Automatically generates reveal.js slides given markdown files

## Roadmap

In rough order of priority

- [ ] Better UI
- [ ] Superior error handling that does not involve CRASHING the program
- [ ] Check if the file is actually a `.md` file
- [ ] Support revealJS animations


## Usage

Place all your markdown files in the `input/` directory.
The slides should be named as such
`<slide number>_<whatever you want>`.md

Alternatively, they can literally just be `<slide number>.md`

A small example is shown in the `input/` directory in this repo.

Call `mkrevealslides`. It will process the files and dump your slides in
`output/slides.html`. (The name currently cannot be customized)