# maigret

**Professional OSINT Username Scanner - Rust Edition**

A high-performance username investigation tool that searches across 2000+ social networks and websites. This is a complete Rust port of the original Go implementation, featuring enhanced CLI aesthetics, async concurrency, and professional-grade logging.

[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)

<p align="center">
  <img src="https://raw.githubusercontent.com/krishpranav/maigret/master/images/maigret.png" height="200"/>
</p>

## ✨ Features

- 🔎 **Comprehensive Coverage**: Search across 2000+ social networks and platforms
- ⚡ **Blazing Fast**: Async concurrent scanning with configurable worker pools (32 workers default)
- 🎨 **Beautiful CLI**: Professional OSINT-style output with colors, progress tracking, and structured logging
- 🔒 **Privacy-Focused**: Optional Tor proxy support for anonymous scanning
- 📸 **Screenshot Capture**: Automated headless Chrome screenshots of found profiles
- 📥 **Content Download**: Download profile data from supported sites (Instagram, etc.)
- 🧪 **Site Validation**: Built-in test mode to verify site configurations

## 🚀 Installation

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Chrome/Chromium** (optional, for screenshots) - Version 60+

### Build from Source

```bash
git clone https://github.com/krishpranav/maigret
cd maigret
cargo build --release
```

The compiled binary will be available at `./target/release/maigret`

### Install Globally

```bash
cargo install --path .
```

## 📖 Usage

### Basic Scan

```bash
maigret krishpranav
```

### Scan Multiple Usernames

```bash
maigret krishpranav blue red
```

### Verbose Output (Show Not Found Sites)

```bash
maigret user -v
```

### Specific Site Only

```bash
maigret user --site github
```

### With Tor Proxy

Requires Tor running on `127.0.0.1:9050`

```bash
maigret user --tor
```

### Capture Screenshots

```bash
maigret user --screenshot
```

Screenshots will be saved to `screenshots/<username>/`

### Download Content

```bash
maigret user --download
```

### Update Database

```bash
maigret user --update
```

### Test Mode (Validate Site Configurations)

```bash
maigret --test
```

### All Options

```bash
maigret --help
```

## 🎯 Example Output

```
    ███╗   ███╗ █████╗ ██╗ ██████╗ ██████╗ ███████╗████████╗
    ████╗ ████║██╔══██╗██║██╔════╝ ██╔══██╗██╔════╝╚══██╔══╝
    ██╔████╔██║███████║██║██║  ███╗██████╔╝█████╗     ██║   
    ██║╚██╔╝██║██╔══██║██║██║   ██║██╔══██╗██╔══╝     ██║   
    ██║ ╚═╝ ██║██║  ██║██║╚██████╔╝██║  ██║███████╗   ██║   
    ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝   ╚═╝   
    
    🔎 Professional OSINT Username Scanner - Rust Edition

🔎 Investigating user on:

[+] GitHub: https://www.github.com/user
[+] Instagram: https://www.instagram.com/user
[+] Twitter: https://twitter.com/user
[-] Pinterest: Not Found!

═══════════════════════════════════════
  🧠 SCAN COMPLETE
═══════════════════════════════════════
  Found: 12
  Checked: 2300
  Time: 3.2s
═══════════════════════════════════════
```

## 🛠️ CLI Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--help` | `-h` | Show help message |
| `--version` | `-V` | Show version |
| `--no-color` | | Disable colored output |
| `--verbose` | `-v` | Show not found sites |
| `--tor` | `-t` | Use Tor proxy (127.0.0.1:9050) |
| `--screenshot` | `-s` | Take screenshots of found profiles |
| `--download` | `-d` | Download profile content |
| `--update` | | Update site database from Sherlock |
| `--database <PATH>` | | Use custom database file |
| `--site <SITE>` | | Check specific site only |
| `--test` | | Run site validation tests |

## 🔧 Configuration

### Worker Pool Size

- **Default**: 32 concurrent workers
- **With Screenshots**: 8 workers (automatically reduced)

### Tor Proxy

Default proxy address: `socks5://127.0.0.1:9050`

To use Tor:
1. Install and start Tor service
2. Run maigret with `--tor` flag

## 📊 Performance

The Rust implementation provides:
- **Async I/O**: Non-blocking concurrent requests using Tokio
- **Memory Efficient**: Minimal allocations with zero-copy where possible
- **Fast Startup**: Compiled binary with instant execution
- **Resource Control**: Configurable concurrency limits

## 🎨 Technology Stack

- **Runtime**: [Tokio](https://tokio.rs/) - Async runtime
- **HTTP**: [Reqwest](https://docs.rs/reqwest/) - HTTP client with SOCKS5 support
- **CLI**: [Clap](https://docs.rs/clap/) - Command-line argument parsing
- **Logging**: [Tracing](https://docs.rs/tracing/) - Structured logging
- **UI**: [Indicatif](https://docs.rs/indicatif/) + [Console](https://docs.rs/console/) + [Colored](https://docs.rs/colored/) - Beautiful terminal output
- **Screenshots**: [Headless Chrome](https://docs.rs/headless_chrome/) - Browser automation
- **Regex**: [Fancy Regex](https://docs.rs/fancy-regex/) - Advanced pattern matching

## 📝 Data Files

The following files are **unchanged** from the Go version:

- `data.json` - Site database (2000+ sites)
- `sites.md` - Site documentation
- `generate_sites_md.py` - Site list generator

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📜 License

MIT License - see LICENSE file for details

## 🔗 Related Projects

- [Sherlock](https://github.com/sherlock-project/sherlock) - Python OSINT tool
- [WhatsMyName](https://github.com/WebBreacher/WhatsMyName) - Username enumeration

---

**Made with ❤️ and Rust**
