# Meta task running all the linters at once.
lint: lint-md lint-spellcheck

# Lint markown files.
lint-md:
    npx --yes markdownlint-cli2 "**/*.md" "#target"

# Spell check the source code.
lint-spellcheck:
    @cargo spellcheck --version >/dev/null || cargo install cargo-spellcheck
    cargo spellcheck check -m 1

# Meta tasks running all formatters at once.
fmt: fmt-md fmt-just

# Format the jusfile.
fmt-just:
    just --fmt --unstable

# Format markdown files.
fmt-md:
    npx --yes prettier --write --prose-wrap always **/*.md
