# WiRu (Window Rust)

**WiRu** is a lightweight Windows window management utility written in Rust. It allows users to control application windows through customizable keyboard shortcuts, providing a faster and more convenient desktop experience.

All keyboard shortcuts and application settings are managed through a single JSON configuration file.

---

## ✨ Features

- **Window Management:** Control application windows using keyboard shortcuts.
- **Close Windows:** Close the currently active window.
- **Fullscreen Mode:** Switch the active window to fullscreen mode.
- **Exit Fullscreen:** Restore a fullscreen window to its previous windowed state.
- **Minimize Windows:** Minimize the currently active window to the taskbar.
- **Custom Shortcuts:** Define and customize keyboard shortcuts.
- **JSON Configuration:** Manage all settings and shortcuts through a JSON file.
- **Lightweight:** Designed to run efficiently with minimal resource consumption.
- **Windows Native:** Built specifically for Microsoft Windows using Rust.

---

## ⌨️ Keyboard Shortcuts

WiRu uses customizable keyboard shortcuts to perform window management operations.

The following table illustrates the intended default shortcut configuration:

| Action            | Default Shortcut    | Description                  |
| ----------------- | ------------------- | ---------------------------- |
| Close Window      | `Alt + Shift + D`   | Closes the active window.    |
| Toggle Fullscreen | `Alt + Shift + F`   | Enters fullscreen mode.      |
| Minimize Window   | `Alt + Shift + I`   | Minimizes the active window. |

> **Note:** These are proposed default shortcuts. Actual shortcuts depend on the configuration in `config.json` and the implemented application behavior.

---

## ⚙️ Configuration

WiRu uses a JSON configuration file to manage its settings and keyboard shortcuts.

All configurable shortcuts are stored in `wiruconfig.json`.

### Example Configuration

```json
{
  "current_mode": "floating",
  "hotkeys": {
    "toggle_mode": "ALT+M",
    "focus_up": "ALT+UP",
    "focus_down": "ALT+DOWN",
    "focus_left": "ALT+LEFT",
    "focus_right": "ALT+RIGHT",
    "close_window": "ALT+X",
    "maximize_window": "ALT+F",
    "minimize_window": "ALT+I"
  }
}
```

> **Important:** The configuration file defines the intended settings. Each option must be supported by the application implementation to take effect.

---

## 📦 Installation

### Prerequisites

- Microsoft Windows
- Rust toolchain
- Cargo package manager

Install Rust using the official Rust website:

[https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Clone the Repository

```bash
git clone https://github.com/PyNetCoder/WiRu.git
```

### Navigate to the Project Directory

```bash
cd WiRu
```

### Build the Application

```bash
cargo build --release
```

### Run WiRu

```bash
cargo run --release
```

Alternatively, execute the compiled binary:

```bash
target/release/WiRu.exe
```

---

## 🗂️ Project Structure

An example project structure:

```text
WiRu/
│
├── src/
│   ├── main.rs
│
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
└── .gitignore
```

| File           | Description                             |
| -------------- | --------------------------------------- |
| `main.rs`      | Application entry point.                |
| `Cargo.toml`   | Rust project dependencies and metadata. |
| `LICENSE`      | Apache License 2.0.                     |

---

## 🖥️ System Requirements

| Requirement          | Specification      |
| -------------------- | ------------------ |
| Operating System     | Microsoft Windows  |
| Architecture         | x86_64 (64-bit)    |
| Programming Language | Rust               |
| Configuration Format | JSON               |
| Build System         | Cargo              |
| License              | Apache License 2.0 |

---

## 🔒 Security & Permissions

WiRu is designed to manage application windows through Windows system APIs.

- WiRu should only perform actions requested through configured keyboard shortcuts.
- Configuration files should be validated before being loaded.
- Invalid shortcuts should be handled safely.
- WiRu should not require administrator privileges for ordinary window management operations unless a specific Windows restriction requires them.

---

## 🛠️ Development

WiRu is developed using Rust, focusing on performance, reliability, and maintainability.

### Check the Project

```bash
cargo check
```

### Run Tests

```bash
cargo test
```

### Build an Optimized Release

```bash
cargo build --release
```

---

## 🤝 Contributing

Contributions are welcome!

If you want to contribute:

1. Fork the repository.
2. Create a new branch.
3. Make your changes.
4. Test your implementation.
5. Submit a Pull Request.

Please ensure that your changes follow Rust coding conventions and maintain compatibility with supported Windows versions.

---

## 🐛 Bug Reports & Feature Requests

If you encounter a bug or have an idea for a new feature, please open an issue in the GitHub repository.

When reporting a bug, include:

- Windows version
- WiRu version
- Steps to reproduce the issue
- Expected behavior
- Actual behavior
- Relevant error messages or logs

---

## 📜 License

WiRu is licensed under the **Apache License, Version 2.0**.

You may use, modify, distribute, and contribute to this project under the terms and conditions of the Apache License 2.0.

See the [LICENSE](.\LICENSE) file for the complete license text.

For more information:

[https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0)

---

## 👨‍💻 Project Information

| Property      | Value                             |
| ------------- | --------------------------------- |
| Project Name  | WiRu                              |
| Full Name     | Window Rust                       |
| Language      | Rust                              |
| Platform      | Windows                           |
| Configuration | JSON                              |
| License       | Apache-2.0                        |
| Project Type  | Desktop Window Management Utility |

---

**WiRu: Your Windows. Your Shortcuts. Your Control.**
