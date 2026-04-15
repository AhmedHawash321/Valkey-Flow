# Valkey Flow 🚀
A high-performance **Rust** backend service for processing data streams using **Valkey/Redis**.

## Features
- **Async Consumer:** Built with `tokio` for non-blocking stream processing.
- **Graceful Shutdown:** Uses `CancellationToken` for clean exits.
- **CI/CD:** Automated testing pipeline via GitHub Actions.

## How to Run Locally
1. Ensure you have **Valkey** or **Redis** running on `6379`.
2. Create a `.env` file (see `.env.example`).
3. Run the project:
   ```bash
   cargo run --example c05