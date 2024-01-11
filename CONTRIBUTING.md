# CONTRIBUTING

This is a Rust project, so [rustup](https://rustup.rs/) is the best place to start.

## The guide

The guide for this project is built using [mdBook](https://rust-lang-nursery.github.io/mdBook/index.html).
Review their guide for more details on how to work with mdBook. Here are a few of the pertinents:

```
# Install the CLI.
cargo install mdbook

# Build the guide.
cd guide-directory \\nmdbook build

# Watch the FS for changes & rebuild.
mdbook watch
```

## Working with git

- **Write the commit message like writing an email to your friends**. There is a great [git commit format guide](https://cbea.ms/git-commit/).

- Do **rebase** and **squash** the branch onto the latest `main` branch before publishing a PR.

- Do **NOT** **rebase** after publishing PR. Only merge.


## Release checklist

- First, make sure you have installed the necessary dependencies through cargo and have a proper configuration for the Rust project.
- To run the unit tests, use `cargo test`.
- To perform a format check, use `cargo fmt`.
- To perform a clippy check, use `cargo clippy`.

- Any documentation updates should also be reflected in the guide.

- Ensure the Cargo.toml version for openraft or memstore has been updated, depending on which is being released.

- Once the release CI has been finished, navigate to the release page, update the release info and publish the release.
