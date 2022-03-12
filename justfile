# Meta task running all the linters at once.
lint: lint-md

# Lint markown files.
lint-md:
    npx --yes markdownlint-cli2 "**/*.md" "#target"

# Meta tasks running all formatters at once.
fmt: fmt-md fmt-just

# Format the jusfile.
fmt-just:
    just --fmt --unstable

# Format markdown files.
fmt-md:
    npx --yes prettier --write --prose-wrap always **/*.md
