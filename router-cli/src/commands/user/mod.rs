pub mod models;

use crate::client::ApiClient;
use crate::common::MessageResponse;
use crate::error::CliError;
use crate::output::{self, Format};
use clap::Subcommand;
use models::{CreateUserRequest, UpdateUserRequest, User};

#[derive(Subcommand)]
pub enum UserAction {
    /// List all users (admin only)
    List,
    /// Get a user by ID
    Get { id: String },
    /// Create a new user (admin only)
    Create {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
        /// Role: admin, staff, or user
        #[arg(long, default_value = "user")]
        role: String,
    },
    /// Update a user
    Update {
        id: String,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        role: Option<String>,
    },
    /// Delete a user
    Delete {
        id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

pub async fn run(
    client: &mut ApiClient,
    action: &UserAction,
    fmt: &Format,
) -> Result<(), CliError> {
    match action {
        UserAction::List => list(client, fmt).await,
        UserAction::Get { id } => get(client, id, fmt).await,
        UserAction::Create {
            username,
            email,
            password,
            role,
        } => create(client, username, email, password, role, fmt).await,
        UserAction::Update {
            id,
            username,
            email,
            password,
            role,
        } => {
            update(
                client,
                id,
                username.as_deref(),
                email.as_deref(),
                password.as_deref(),
                role.as_deref(),
                fmt,
            )
            .await
        }
        UserAction::Delete { id, yes } => delete(client, id, *yes).await,
    }
}

async fn list(client: &mut ApiClient, fmt: &Format) -> Result<(), CliError> {
    let users: Vec<User> = client.get("/api/v1/users/admin").await?;

    match fmt {
        Format::Json => output::print_json(&users),
        Format::Yaml => output::print_yaml(&users),
        Format::Table => {
            let headers = &["ID", "USERNAME", "EMAIL", "ROLE", "CREATED_AT"];
            let rows: Vec<Vec<String>> = users
                .iter()
                .map(|u| {
                    vec![
                        u.id.clone(),
                        u.username.clone(),
                        u.email.clone(),
                        u.role.clone(),
                        u.created_at.clone().unwrap_or_default(),
                    ]
                })
                .collect();
            output::print_table(headers, &rows);
            Ok(())
        }
    }
}

async fn get(client: &mut ApiClient, id: &str, fmt: &Format) -> Result<(), CliError> {
    let user: User = client.get(&format!("/api/v1/users/{id}")).await?;

    match fmt {
        Format::Json => output::print_json(&user),
        Format::Yaml => output::print_yaml(&user),
        Format::Table => {
            println!("User: {}", user.username);
            println!("  ID:      {}", user.id);
            println!("  Email:   {}", user.email);
            println!("  Role:    {}", user.role);
            if let Some(ref ts) = user.created_at {
                println!("  Created: {ts}");
            }
            if let Some(ref ts) = user.updated_at {
                println!("  Updated: {ts}");
            }
            Ok(())
        }
    }
}

async fn create(
    client: &mut ApiClient,
    username: &str,
    email: &str,
    password: &str,
    role: &str,
    fmt: &Format,
) -> Result<(), CliError> {
    let input = CreateUserRequest {
        username: username.to_string(),
        email: email.to_string(),
        password: password.to_string(),
        role: Some(role.to_string()),
    };

    let result: User = client.post_json("/api/v1/users/admin", &input).await?;

    match fmt {
        Format::Json => output::print_json(&result),
        Format::Yaml => output::print_yaml(&result),
        Format::Table => {
            println!("User created: {} ({})", result.username, result.id);
            Ok(())
        }
    }
}

async fn update(
    client: &mut ApiClient,
    id: &str,
    username: Option<&str>,
    email: Option<&str>,
    password: Option<&str>,
    role: Option<&str>,
    fmt: &Format,
) -> Result<(), CliError> {
    let input = UpdateUserRequest {
        username: username.map(String::from),
        email: email.map(String::from),
        password: password.map(String::from),
        role: role.map(String::from),
    };

    let result: User = client
        .put_json(&format!("/api/v1/users/{id}"), &input)
        .await?;

    match fmt {
        Format::Json => output::print_json(&result),
        Format::Yaml => output::print_yaml(&result),
        Format::Table => {
            println!("User updated: {} ({})", result.username, result.id);
            Ok(())
        }
    }
}

async fn delete(client: &mut ApiClient, id: &str, yes: bool) -> Result<(), CliError> {
    if !yes {
        eprint!("Are you sure you want to delete user '{id}'? [y/N]: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let resp: MessageResponse = client.delete(&format!("/api/v1/users/{id}")).await?;
    println!("{}", resp.message);
    Ok(())
}
