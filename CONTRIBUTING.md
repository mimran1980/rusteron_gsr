# Contributing to rusteron

Thanks for your interest in contributing to **rusteron**! Whether you're fixing bugs, improving docs, or adding features, contributions are welcome.

## Before You Start

- Submit changes via pull requests from a fork.
- Run `cargo fmt` and `cargo test` before submitting.
- All contributions are dual-licensed under MIT and Apache-2.0.

## Reporting Issues

Use [GitHub Issues](https://github.com/gsrxyz/rusteron/issues) to report bugs, request features, or ask questions. Include clear steps to reproduce and any relevant error messages or context.

## Pull Request Workflow

1. Fork the repo and clone it:
   ```bash
   git clone https://github.com/your-username/rusteron.git
   cd rusteron
````

2. Create a new branch:

   ```bash
   git checkout -b your-branch-name
   ```

3. Make changes, run tests, and format:

   ```bash
   cargo test
   cargo fmt
   ```

4. Commit and push:

   ```bash
   git commit -m "Describe your change"
   git push origin your-branch-name
   ```

5. Open a pull request on GitHub.

## Documentation

Typos, clarifications, or missing details? Feel free to open a PR. No test coverage required for doc-only changes.

## Code Style & Commit Tips

* Keep PRs focused and scoped.
* Use clear commit messages (e.g. `"Fix: prevent panic on empty buffer"`).
* Test coverage is expected for fixes or new features.
