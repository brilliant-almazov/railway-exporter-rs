# Contributing

Contributions are welcome!

## How to contribute

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/my-feature`)
3. **Commit** your changes (`git commit -m 'Add my feature'`)
4. **Push** to the branch (`git push origin feature/my-feature`)
5. **Open** a Pull Request

## Issues

Found a bug or have a feature request? [Open an issue](https://github.com/brilliant-almazov/railway-exporter-rs/issues/new).

## Development

```bash
# Clone
git clone https://github.com/brilliant-almazov/railway-exporter-rs.git
cd railway-exporter-rs

# Build
cargo build

# Run tests
cargo test

# Run locally
export RAILWAY_API_TOKEN=your-token
export RAILWAY_PROJECT_ID=your-project-id
cargo run
```

## Code style

- Run `cargo fmt` before committing
- Run `cargo clippy` to check for warnings
