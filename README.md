# 🐇 NetHop

**NetHop** is a developer-centric, scriptable HTTP client for the terminal. It bridges the gap between `curl` and heavy GUI clients like Postman, allowing you to define, execute, and inspect complex API workflows using a simple, human-readable `.hop` scripting format.

## ✨ Features

* **Scriptable Workflows**: Define connection settings and multiple request blocks in a single `.hop` file.
* **Persistent Connections**: Optimized for speed by reusing a single `TcpStream` and `BufReader` across multiple requests.
* **Secure by Default**: Native TLS support for HTTPS calls.
* **Modern HTTP Support**: Handles `Transfer-Encoding: chunked` and large-scale JSON payloads.
* **Smart Paging**: Automatically detects large responses and pipes them into a pager (`less`) with JSON pretty-printing.
* **High Performance**: Built in Rust with zero-copy parsing techniques to handle large batch files without memory bloat.

---

## 🚀 Quick Start

### 1. Create a `.hop` script
Save the following as `poke_test.hop`:

```hop
<connect>
host = pokeapi.co
</connect>

<query>
method = GET
url = /api/v2/pokemon/mewtwo
</query>

```

### 2. Run the Engine

```bash
cargo run -- poke_test.hop

```

---

## 🛠️ Script Syntax

The `.hop` format uses a simple tag-based structure:

* **`<connect>`**: Define your target `host` and `port`. Use the `unsafe` keyword to switch to port 80/HTTP.
* **`<query>`**: Define a request.
* `method`: GET, POST, PUT, DELETE, etc.
* `url`: The endpoint path.
* `content-type`: Defaults to `application/json`.


* **`<body>`**: Place your request payload between `<body>` and `</body>` tags inside a query block.

---

## 📦 Project Architecture

NetHop is designed with a strict separation of concerns to ensure safety and speed:

* **`network.rs`**: Manages the `Stream` enum (switching between raw TCP and TLS) and handles the physical connection.
* **`http.rs`**: Implements the HTTP/1.1 protocol, including header parsing and chunked-body assembly.
* **`ui.rs`**: Manages the terminal output and integration with system pagers.
* **`main.rs`**: The script orchestrator that parses `.hop` files into executable instructions.

---

## 🔧 Installation

1. **Install Rust**: Ensure you have the [Rust toolchain](https://rustup.rs/) installed.
2. **Clone & Build**:
```bash
git clone [https://github.com/yourusername/nethop.git](https://github.com/yourusername/nethop.git)
cd nethop
cargo build --release

```

> NetHop has limited support for mime-types as of now. A list of supported mime types and future roadmap can be found [here](./docs/mime-type-support.md)

## 🗺️ Roadmap

* [x] **HTTP + TLS Support**: Basic HTTP requests with safe TLS support.
* [x] **Scripting**: Custom scripting language to write network queries.
* [ ] **Workspace testing suite**: Support for workspace detection and automatically run hop files inside a project + metrics and summaries on the queries similar to Jest & Vitest suites.
* [ ] **Content Types**: Support for more accaptable content types other than text and json.
* [ ] **Async Streaming**: Execute background requests while viewing the current response.
* [ ] **Variable Injection**: Support environment variables like `{{API_KEY}}` inside `.hop` files.
* [ ] **Header Customization**: Add support for custom header blocks in scripts.

---

## 📜 License

Licensed under the MIT License. See `LICENSE` for details.

---

*Built with 🦀 by developers, for developers.*
