# Valkey Flow

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/AhmedHawash321/valkey-flow/actions/workflows/rust-tests.yml/badge.svg)](https://github.com/AhmedHawash321/valkey-flow/actions)

**A production-ready Redis/Valkey stream consumer library written in Rust with consumer groups, graceful shutdown, and configuration management.**

---

## 📋 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
  - [Basic Example](#basic-example)
  - [Multiple Consumers with Groups](#multiple-consumers-with-groups)
- [Testing](#testing)
- [Docker Setup](#docker-setup)
- [CI/CD](#cicd)
- [Project Structure](#project-structure)
- [API Documentation](#api-documentation)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## 🎯 Overview

**Valkey Flow** is a lightweight, high-performance Rust library for working with Redis/Valkey streams. It provides a clean abstraction over Redis Streams with built-in support for:

- Consumer groups for distributed message processing
- Graceful shutdown using cancellation tokens
- Environment-based configuration
- Integration testing with real Redis/Valkey instances
- Docker support for local development

This library is ideal for building **event-driven systems**, **message queues**, and **real-time data pipelines** in production environments.

---

## ✨ Features

| Feature | Status | Description |
|---------|--------|-------------|
| **Consumer Groups** | ✅ | Distribute messages across multiple consumers |
| **Graceful Shutdown** | ✅ | Clean shutdown with `CancellationToken` |
| **Auto-ACK** | ✅ | Automatic message acknowledgment after processing |
| **Config Management** | ✅ | `.env` + environment variables support |
| **Integration Tests** | ✅ | Full flow tests with real Redis/Valkey |
| **Docker Support** | ✅ | `docker-compose.yml` for local development |
| **CI/CD** | ✅ | GitHub Actions automated testing |
| **Documentation** | ✅ | Comprehensive API docs with examples |

---

## 🏗️ Architecture

┌─────────────┐ ┌─────────────────────────────────────┐
│ Writer │────▶│ Redis/Valkey Stream │
│ (Producer) │ │ │
└─────────────┘ │ ┌──────────┐ ┌──────────┐ │
│ │ Group A │ │ Group B │ │
│ └────┬─────┘ └────┬─────┘ │
│ │ │ │
│ ┌────▼─────┐ ┌────▼─────┐ │
│ │Consumer 1│ │Consumer 3│ │
│ │Consumer 2│ │Consumer 4│ │
│ └──────────┘ └──────────┘ │
└─────────────────────────────────────┘


- **Writer**: Produces messages to the stream
- **Consumer Groups**: Isolated groups with their own message processing logic
- **Consumers**: Individual workers within a group (messages are distributed among them)
- **ACK**: Automatic acknowledgment after successful processing

---

## 📦 Prerequisites

- **Rust** 1.70 or later ([install](https://www.rust-lang.org/tools/install))
- **Redis** 6.0+ or **Valkey** 7.0+ (or use Docker as shown below)
- **Docker** (optional, for containerized Redis/Valkey)

---


- **Writer**: Produces messages to the stream
- **Consumer Groups**: Isolated groups with their own message processing logic
- **Consumers**: Individual workers within a group (messages are distributed among them)
- **ACK**: Automatic acknowledgment after successful processing

---

## 📦 Prerequisites

- **Rust** 1.70 or later ([install](https://www.rust-lang.org/tools/install))
- **Redis** 6.0+ or **Valkey** 7.0+ (or use Docker as shown below)
- **Docker** (optional, for containerized Redis/Valkey)

---

## 🚀 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
valkey-flow = "0.1.0"
redis = { version = "0.32.1", features = ["tokio-comp"] }
tokio = { version = "1", features = ["full"] }

```
Or clone the Repositry:
```
 git clone https://github.com/AhmedHawash321/valkey-flow.git
cd valkey-flow
```
---

## ⚙️ Configuration
1. Set up environment variables
Copy the example configuration:

```
cp .env.example .env

```
**Edit .env with your settings:**

```
VALKEY_URL=redis://127.0.0.1:6379
PRIMARY_STREAM=stream-c05
GROUP_ONE=group_01
GROUP_TWO=group_02
```

---

**2. Configuration options**

Variable	Description	Default
VALKEY_URL	Redis/Valkey connection URL	redis://127.0.0.1:6379
PRIMARY_STREAM	Stream name	stream-c05
GROUP_ONE	First consumer group name	group_01
GROUP_TWO	Second consumer group name	group_02

---

**3. Using the config in code**

```
use valkey_flow::ValkeyConfig;

let config = ValkeyConfig::from_env();
println!("Connecting to: {}", config.url);
```

---

## 💻 Usage

**Basic Example: Single Consumer**

```
use valkey_flow::{create_group, run_consumer};
use redis::Client;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::open("redis://127.0.0.1:6379")?;
    let stream_name = "mystream";
    let group_name = "mygroup";
    let token = CancellationToken::new();

    // Create consumer group
    create_group(&client, stream_name, group_name).await?;

    // Run consumer
    let handle = run_consumer(
        &client,
        stream_name.to_string(),
        group_name.to_string(),
        "consumer1".to_string(),
        token.clone(),
    ).await?;

    // Send some messages
    let mut con = client.get_multiplexed_async_connection().await?;
    for i in 0..5 {
        let _: String = redis::cmd("XADD")
            .arg(stream_name).arg("*").arg("msg").arg(i.to_string())
            .query_async(&mut con).await?;
    }

    // Wait and shutdown gracefully
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    token.cancel();
    handle.await?;

    Ok(())
}
```

---

## 🧪 Testing

**Run integration tests**

```
# Start Redis/Valkey (using Docker)
docker-compose up -d

# Run tests
cargo test --test valkey_integrations -- --nocapture

# Clean up
docker-compose down
```
**Test output example**

```
running 1 test
test test_full_stream_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s
```

---

## 🐳 Docker Setup

**Start Redis/Valkey**

```
docker-compose up -d
```
**Persist data**

Data is automatically persisted in the valkey_data Docker volume.

---

## 🔄 CI/CD

This project uses GitHub Actions for continuous integration:

Runs on every push and pull_request to main/master

Spins up a Valkey container for integration tests

Executes cargo test --test valkey_integrations

See .github/workflows/rust-tests.yml for details.

---

## 📁 Project Structure

```
valkey-flow/
├── .github/
│   └── workflows/
│       └── rust-tests.yml      # CI/CD pipeline
├── src/
│   ├── lib.rs                  # Public API exports
│   ├── logic.rs                # Core business logic
│   └── config.rs               # Configuration management
├── tests/
│   └── valkey_integrations.rs  # Integration tests
├── examples/
│   ├── c01-simple.rs           # Basic usage example
│   ├── c02-xadd.rs             # Producer example
│   ├── c03-async.rs            # Async consumer example
│   ├── c04-xgroup.rs           # Consumer groups example
│   └── c05-groups.rs           # Multiple groups example
├── .env.example                # Example environment variables
├── .gitignore                  # Git ignore rules
├── Cargo.toml                  # Dependencies
├── Cargo.lock                  # Locked dependencies
├── docker-compose.yml          # Docker setup
├── README.md                   # This file
└── LICENSE                     # MIT license
```

---

## 📚 API Documentation

create_group
Creates a consumer group for a Redis/Valkey stream.

```
pub async fn create_group(
    client: &Client,
    stream_name: &str,
    group_name: &str,
) -> Result<(), Box<dyn std::error::Error>>

```

**run_consumer**
Spawns a consumer task that reads messages from a stream.

```
pub async fn run_consumer(
    client: &Client,
    stream_name: String,
    group_name: String,
    consumer_name: String,
    token: CancellationToken,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>>

```
**ValkeyConfig**

Configuration struct with environment variable support.

```
pub struct ValkeyConfig {
    pub url: String,
    pub stream: String,
    pub g1: String,
    pub g2: String,
}

impl ValkeyConfig {
    pub fn from_env() -> Self;
}
```

---

## 🗺️ Roadmap

Add support for custom error types (thiserror)

Add unit tests for logic.rs

Add benchmark tests (criterion)

Support for Redis Cluster

Add tracing for structured logging

Publish to crates.io

---

## 🤝 Contributing

**Contributions are welcome! Please follow these steps:**

Fork the repository

Create a feature branch (git checkout -b feature/amazing-feature)

Commit your changes (git commit -m 'Add amazing feature')

Push to the branch (git push origin feature/amazing-feature)

Open a Pull Request

---

**Development guidelines**

Run cargo fmt before committing

Run cargo clippy to catch common mistakes

Add tests for new functionality

Update documentation as needed

---

## 📄 License
This project is licensed under the MIT License - see the LICENSE file for details.

---

## 🙏 Acknowledgments
**Redis** for the amazing in-memory data store

**Valkey** for the open-source alternative

**Tokio** for the async runtime

---

## 👨‍💻 Author

**Ahmed Hawash**
Backend Engineer · Node.js & Rust · Blockchain Developer

[![LinkedIn](https://img.shields.io/badge/LinkedIn-Ahmed_Hawash-blue?style=for-the-badge&logo=linkedin)](https://www.linkedin.com/in/ahmed-hawash-21b992149/)
[![GitHub](https://img.shields.io/badge/GitHub-AhmedHawash321-black?style=for-the-badge&logo=github)](https://github.com/AhmedHawash321)

---

⭐ If you find this project useful, please consider giving it a star! ⭐