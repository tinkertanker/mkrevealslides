# `mkrevealslides`
*The rusty slide compiler*

[![Rust](https://github.com/tinkertanker/mkrevealslides/actions/workflows/rust.yml/badge.svg)](https://github.com/tinkertanker/mkrevealslides/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/tinkertanker/mkrevealslides/branch/master/graph/badge.svg?token=YZ12W93CCX)](https://codecov.io/gh/tinkertanker/mkrevealslides)

Automatically generates reveal.js slides given markdown files

## Roadmap

In rough order of priority

- [x] Better UI
- [x] Check if the file is actually a `.md` file
- [x] Superior error handling that does not involve CRASHING the program
  - [x] More helpful error messages 
- [x] Configuration file support
    - [x] Force paths relative to config file dir instead of invocation dir
- [x] Support markdown images
  - [x] Allow the program to modify markdown file paths for correct resolution 
- [x] Support slide 1a, 1b etc.
- [ ] Generate zip file with revealJS deps
- [ ] Automatically download revealJS if not present 
- [ ] Support revealJS animations
- [ ] Tests
    - [x] Unit tests
    - [x] Integration tests


## User guide

See [docs/README.md](docs/README.md)
