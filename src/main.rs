use std::fs::{create_dir_all, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

use toml::{Table, Value};

trait ExtValue {
    fn get_table(&self, key: &str) -> Option<&Table>;
    fn get_table_keys(&self, key: &str) -> Option<Vec<String>>;
    fn get_str(&self, key: &str) -> Option<&str>;
}

impl ExtValue for Value {
    fn get_table(&self, key: &str) -> Option<&Table> {
        self.get(key).and_then(Value::as_table)
    }

    fn get_table_keys(&self, key: &str) -> Option<Vec<String>> {
        let mut keys = self
            .get(key)
            .and_then(Value::as_table)
            .map(|table| table.keys().cloned().collect::<Vec<_>>())?;
        keys.sort();
        Some(keys)
    }

    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
    }
}

fn read_str() -> String {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    buffer.trim().into()
}

fn prompt(text: &str) {
    print!("{text}");
    io::stdout().flush().unwrap();
}

fn add_identities(config: &mut Value, config_file: &Path) -> Result<(), String> {
    let (key, new_identity) = add_new_identity();

    let identities_table = config
        .get_mut("identities")
        .and_then(|v| v.as_table_mut())
        .ok_or_else(|| "give this a sensible error".to_string())?;

    identities_table.insert(key, new_identity.clone());

    save_config(config_file, config).map_err(|e| format!("Error saving config: {e}"))?;

    let name = new_identity.get_str("name");
    let email = new_identity.get_str("email");

    let Some((name, email)) = name.zip(email) else {
        return Ok(());
    };

    apply_identity(name, email)
}

fn help() {
    println!("\nOptions:");
    println!("  [number] - Select and set that identity");
    println!("  a        - Add a new identity");
    println!("  q        - Quit");
}

fn run() -> Result<(), String> {
    let config_file = get_config_file_path();
    let mut config = load_config(&config_file).unwrap_or_else(create_empty_config);

    loop {
        println!("\n=== Git Identity Manager ===\n");

        match config.get_table("identities") {
            Some(identities_table) if identities_table.is_empty() => {
                println!("No identities found.")
            }
            Some(identities_table) => {
                println!("Available identities:");

                let mut keys = identities_table.keys().collect::<Vec<_>>();
                keys.sort();

                for (i, key) in keys.iter().enumerate() {
                    let Some(identity) = identities_table.get(*key) else {
                        continue;
                    };
                    let name = identity.get_str("name").unwrap_or("N/A");
                    let email = identity.get_str("email").unwrap_or("N/A");
                    println!("  {}. {name} <{email}>", i + 1);
                }
            }
            None => println!("Identities configuration missing."),
        }

        help();

        prompt("\nEnter your choice: ");
        let choice = read_str();

        match choice.as_str() {
            "q" | "Q" => {
                println!("Exiting without changes.");
                break Ok(());
            }
            "a" | "A" => break add_identities(&mut config, &config_file),
            num => {
                let keys = Option::unwrap_or(config.get_table_keys("identities"), vec![]);

                // Get the relevant key based on the index
                let key = match num.parse::<usize>() {
                    Ok(n) if n == 0 || n > keys.len() => {
                        eprintln!("Invalid selection");
                        eprintln!("Enter a number between 1 and {}", keys.len());
                        continue;
                    }
                    Err(_) => {
                        eprintln!("Invalid input, enter a number between 1 and {}", keys.len());
                        continue;
                    }
                    Ok(n) => &keys[n - 1],
                };

                let Some(identity) = config
                    .get_table("identities")
                    .and_then(|table| table.get(key))
                else {
                    break Ok(());
                };

                let name = identity.get_str("name");
                let email = identity.get_str("email");

                if let Some((name, email)) = name.zip(email) {
                    apply_identity(name, email)?;
                }

                break Ok(());
            }
        }
    }
}

/// Returns the path to the configuration file using the dirs crate.
/// This places the file under:
/// ```text
/// - Linux/Unix: $XDG_CONFIG_HOME/git_identity_manager/git_identities.toml (or $HOME/.config/...)
/// - macOS: $HOME/Library/Application Support/git_identity_manager/git_identities.toml
/// - Windows: %APPDATA%\git_identity_manager\git_identities.toml
/// ```
fn get_config_file_path() -> PathBuf {
    let base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_path = base_dir.join("git_identity_manager");
    if let Err(e) = create_dir_all(&config_path) {
        eprintln!("Failed to create config directory: {}", e);
    }
    config_path.join("git_identities.toml")
}

fn load_config(path: &Path) -> Option<toml::Value> {
    let mut file = File::open(path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    toml::from_str(&contents).ok()
}

fn save_config(path: &Path, config: &Value) -> io::Result<()> {
    let toml_string = format_config(config);
    let mut file = File::create(path)?;
    file.write_all(toml_string.as_bytes())
}

/// Formats the configuration in a more conventional TOML style.
/// For example:
///
/// ```text
/// [identities.The_Linux_Developer]
/// name = "The Linux Developer"
/// email = "email@thelinux.dev"
/// ```
fn format_config(config: &toml::Value) -> String {
    let mut output = String::new();

    let Some(identities) = config.get("identities") else {
        return output;
    };
    let Some(identities) = identities.as_table() else {
        return output;
    };

    for (key, value) in identities {
        let Some(inner) = value.as_table() else {
            continue;
        };

        output.push_str(&format!("[identities.{}]\n", key));

        for (key, value) in inner {
            match value.as_str() {
                Some(value) => output.push_str(&format!("{key} = \"{value}\"\n")),
                None => output.push_str(&format!("{key} = {value}\n")),
            }
        }

        output.push('\n');
    }

    output
}

fn create_empty_config() -> toml::Value {
    let mut table = toml::value::Table::new();
    table.insert("identities".to_string(), Value::Table(Table::new()));
    Value::Table(table)
}

/// Interactively prompts the user to add a new identity.
/// The entered Git name is used both as the displayed name and, after replacing spaces with underscores,
/// as the key in the configuration.
fn add_new_identity() -> (String, toml::Value) {
    println!("\nAdding new identity:");

    prompt("Enter the Git name for this identity: ");
    let display_name = read_str();

    prompt("Enter your Git email: ");
    let email = read_str();

    let key = display_name.replace(char::is_whitespace, "_");
    let mut id_table = toml::value::Table::new();
    id_table.insert("name".to_string(), toml::Value::String(display_name));
    id_table.insert("email".to_string(), toml::Value::String(email));

    (key, toml::Value::Table(id_table))
}

fn apply_identity(name: &str, email: &str) -> Result<(), String> {
    let status_name = Command::new("git")
        .args(["config", "user.name", name])
        .status()
        .map_err(|e| e.to_string())?;
    let status_email = Command::new("git")
        .args(["config", "user.email", email])
        .status()
        .map_err(|e| e.to_string())?;

    let result = status_name.success() && status_email.success();
    if result {
        println!("Git identity set to {} <{}>", name, email);
        Ok(())
    } else {
        Err("Failed to set git identity. Make sure you're in a Git repository.".into())
    }
}
