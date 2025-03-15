# Git Identity Manager

Git Identity Manager is a command-line tool written in Rust that helps you
manage multiple Git identities for your repositories. It allows you to add new
identities, view stored identities, and apply an identity to the local Git
repository (without affecting your global configuration).

## Overview

Git Identity Manager stores your Git identities (name and email) in a
human-readable TOML configuration file located in an OS-appropriate directory:

- **Linux/Unix:** `$XDG_CONFIG_HOME/git_identity_manager/git_identities.toml`
  (or defaults to `$HOME/.config/git_identity_manager/git_identities.toml`)
- **macOS:** `$HOME/Library/Application
Support/git_identity_manager/git_identities.toml`
- **Windows:** `%APPDATA%\git_identity_manager\git_identities.toml`

When you run the tool inside a Git repository, it presents an interactive menu
that allows you to:

- **Select an Existing Identity:** Choose from a list of saved identities to
  apply locally.
- **Add a New Identity:** Input a new Git name and email. The name is used as
  the display name, while a sanitized version (spaces replaced with underscores)
  is used as the key in the configuration file.
- **Quit:** Exit the program without making any changes.

Once an identity is applied, the tool sets the local Git configuration (i.e.,
`git config user.name` and `git config user.email`), overwriting any existing
local settings.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable release is
  recommended)
- A Git repository (to test local Git identity settings)

### Build and Install

1. **Clone the Repository:**

   ```bash
   git clone https://git.musselman.dev/Musselman/git-identity-manager
   cd git-identity-manager
   ```

2. **Build the Project:**

   Use Cargo to build the project:

   `cargo build --release`

   The compiled binary will be available in the `target/release` directory.

3. **(Optional) Install the Binary:**

   You can install the binary to your system (if you have Cargo's bin directory
   in your PATH):

   `cargo install --path .`

## Usage

Run the tool from within a Git repository directory using the appropriate binary:

`./git-identity-manager` or `git-identity-manager.exe`

Upon running, you will see a menu similar to this:

```bash
=== Git Identity Manager ===

Available identities:
1. James_Musselman: James Musselman <email@example.com>

Options: [number] - Select and set that identity
          a        - Add a new identity
          q        - Quit

Enter your choice:
```

### Options

- **Selecting an Identity:** Enter the corresponding number to apply the
  identity. The tool will run:

  ```bash
  git config user.name "Your Name"
  git config user.email "your_email@example.com"
  ```

  _Note:_ These commands will update your local repository's Git configuration.

- **Adding a New Identity:** Type `a` and follow the prompts to enter a Git
  name and email address.

  - **Key Sanitization:** If your Git name contains spaces (e.g., "James
    Musselman"), spaces will be replaced with underscores when saving as a key,
    but the display and applied identity remain unchanged.
  - After adding, the new identity is saved to the configuration file and
    immediately applied locally.

- **Quitting:** Enter `q` to exit without making any changes.

## Configuration File

The identities are stored in a TOML file in an OS-appropriate directory. A
sample configuration file may look like:

```toml
[identities.James_Musselman]
name  = "James Musselman"
email = "email@example.com"

[identities.Jane_Doe]
name  = "Jane Doe"
email = "jane.doe@example.com"
```

Each identity is stored under a key where spaces in the Git name are replaced
with underscores.

## Overwriting Behavior

When applying an identity, the tool uses `git config` (without the `--global`
flag) to set the identity in the local repository configuration. This will
overwrite any existing local Git identity settings.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request
with your changes.

## License

This project is licensed under the [MIT License](LICENSE).
