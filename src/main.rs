use std::fs::{create_dir_all, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let config_file = get_config_file_path();
    let mut config = load_config(&config_file).unwrap_or_else(create_empty_config);

    loop {
        println!("\n=== Git Identity Manager ===\n");

        if let Some(identities_table) = config.get("identities").and_then(|v| v.as_table()) {
            if identities_table.is_empty() {
                println!("No identities found.");
            } else {
                println!("Available identities:");
                let mut keys: Vec<&String> = identities_table.keys().collect();
                keys.sort();
                for (i, key) in keys.iter().enumerate() {
                    if let Some(identity) = identities_table.get(*key) {
                        let name = identity
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A");
                        let email = identity
                            .get("email")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A");
                        println!("  {}. {} <{}>", i + 1, name, email);
                    }
                }
            }
        } else {
            println!("Identities configuration missing.");
        }

        println!("\nOptions:");
        println!("  [number] - Select and set that identity");
        println!("  a        - Add a new identity");
        println!("  q        - Quit");

        print!("\nEnter your choice: ");
        io::stdout().flush().unwrap();
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        if choice.eq_ignore_ascii_case("q") {
            println!("Exiting without changes.");
            break;
        } else if choice.eq_ignore_ascii_case("a") {
            let (key, new_identity) = add_new_identity();
            {
                let identities_table = config
                    .get_mut("identities")
                    .and_then(|v| v.as_table_mut())
                    .expect("Identities must be a table");
                identities_table.insert(key.clone(), new_identity.clone());
            }
            if let Err(e) = save_config(&config_file, &config) {
                println!("Error saving config: {}", e);
                break;
            }
            if let (Some(name), Some(email)) = (
                new_identity.get("name").and_then(|v| v.as_str()),
                new_identity.get("email").and_then(|v| v.as_str()),
            ) {
                if apply_identity(name, email) {
                    println!("Git identity set to {} <{}>", name, email);
                } else {
                    println!("Failed to set git identity. Make sure you're in a Git repository.");
                }
            }
            break; 
        } else if let Ok(num) = choice.parse::<usize>() {
            let keys: Vec<String> = {
                if let Some(identities_table) = config.get("identities").and_then(|v| v.as_table())
                {
                    let mut keys: Vec<String> = identities_table.keys().cloned().collect();
                    keys.sort();
                    keys
                } else {
                    vec![]
                }
            };
            if num > 0 && num <= keys.len() {
                let key = &keys[num - 1];
                if let Some(identity) = config
                    .get("identities")
                    .and_then(|v| v.as_table())
                    .and_then(|table| table.get(key))
                {
                    if let (Some(name), Some(email)) = (
                        identity.get("name").and_then(|v| v.as_str()),
                        identity.get("email").and_then(|v| v.as_str()),
                    ) {
                        if apply_identity(name, email) {
                            println!("Git identity set to {} <{}>", name, email);
                        } else {
                            println!(
                                "Failed to set git identity. Make sure you're in a Git repository."
                            );
                        }
                    }
                }
                break;
            } else {
                println!("Invalid selection.");
            }
        } else {
            println!("Invalid input.");
        }
    }
}

/// Returns the path to the configuration file using the dirs crate.
/// This places the file under:
/// - Linux/Unix: $XDG_CONFIG_HOME/git_identity_manager/git_identities.toml (or $HOME/.config/...)
/// - macOS: $HOME/Library/Application Support/git_identity_manager/git_identities.toml
/// - Windows: %APPDATA%\git_identity_manager\git_identities.toml
fn get_config_file_path() -> PathBuf {
    let base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_path = base_dir.join("git_identity_manager");
    if let Err(e) = create_dir_all(&config_path) {
        eprintln!("Failed to create config directory: {}", e);
    }
    config_path.join("git_identities.toml")
}

fn load_config(path: &Path) -> Option<toml::Value> {
    if !path.exists() {
        return None;
    }
    let mut file = File::open(path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    toml::from_str(&contents).ok()
}

fn save_config(path: &Path, config: &toml::Value) -> io::Result<()> {
    let toml_string = format_config(config);
    let mut file = File::create(path)?;
    file.write_all(toml_string.as_bytes())
}

/// Formats the configuration in a more conventional TOML style.
/// For example:
///
/// [identities.The_Linux_Developer]
/// name = "The Linux Developer"
/// email = "email@thelinux.dev"
fn format_config(config: &toml::Value) -> String {
    let mut output = String::new();
    if let Some(identities) = config.get("identities").and_then(|v| v.as_table()) {
        for (key, value) in identities {
            output.push_str(&format!("[identities.{}]\n", key));
            if let Some(inner) = value.as_table() {
                for (k, v) in inner {
                    if let Some(s) = v.as_str() {
                        output.push_str(&format!("{} = \"{}\"\n", k, s));
                    } else {
                        output.push_str(&format!("{} = {}\n", k, v));
                    }
                }
            }
            output.push('\n');
        }
    }
    output
}

fn create_empty_config() -> toml::Value {
    let mut table = toml::value::Table::new();
    table.insert(
        "identities".to_string(),
        toml::Value::Table(toml::value::Table::new()),
    );
    toml::Value::Table(table)
}

/// Interactively prompts the user to add a new identity.
/// The entered Git name is used both as the displayed name and, after replacing spaces with underscores,
/// as the key in the configuration.
fn add_new_identity() -> (String, toml::Value) {
    println!("\nAdding new identity:");
    print!("Enter the Git name for this identity: ");
    io::stdout().flush().unwrap();
    let mut display_name = String::new();
    io::stdin().read_line(&mut display_name).unwrap();
    let display_name = display_name.trim().to_string();

    print!("Enter your Git email: ");
    io::stdout().flush().unwrap();
    let mut email = String::new();
    io::stdin().read_line(&mut email).unwrap();
    let email = email.trim().to_string();

    let key = display_name.replace(" ", "_");
    let mut id_table = toml::value::Table::new();
    id_table.insert("name".to_string(), toml::Value::String(display_name));
    id_table.insert("email".to_string(), toml::Value::String(email));

    (key, toml::Value::Table(id_table))
}

fn apply_identity(name: &str, email: &str) -> bool {
    let status_name = Command::new("git")
        .args(["config", "user.name", name])
        .status();
    let status_email = Command::new("git")
        .args(["config", "user.email", email])
        .status();
    status_name.map(|s| s.success()).unwrap_or(false)
        && status_email.map(|s| s.success()).unwrap_or(false)
}
