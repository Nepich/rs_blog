use std::{env, fs};

use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;

const DEFAULT_HTTP_ADDRESS: &str = "http://localhost:3000/api";
const DEFAULT_GRPC_ADDRESS: &str = "http://localhost:50051";
const TOKEN_FILE: &str = ".blog_token";

#[derive(Parser)]
#[command(author, version, about = "Blog CLI for testing backend scenarios", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Use gRPC transport instead of HTTP
    #[arg(long)]
    grpc: bool,

    /// Server address override, for example localhost:8080 or http://localhost:8080
    #[arg(long)]
    server: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    Get {
        #[arg(long)]
        id: u64,
    },
    Update {
        #[arg(long)]
        id: u64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        content: Option<String>,
    },
    Delete {
        #[arg(long)]
        id: u64,
    },
    List {
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = Cli::parse();
    let transport = build_transport(&cli);
    let mut client = match BlogClient::new(transport).await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("Failed to initialize client: {err}");
            std::process::exit(1);
        }
    };

    if let Some(token) = load_token() {
        client.set_token(token);
    }

    if let Err(err) = execute_command(&cli, &mut client).await {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn build_transport(cli: &Cli) -> Transport {
    let raw_address = match cli.server.clone() {
        Some(server) => server,
        None => {
            let env_key = if cli.grpc {
                "BLOG_GRPC_SERVER"
            } else {
                "BLOG_SERVER"
            };
            env::var(env_key).unwrap_or_else(|_| {
                if cli.grpc {
                    DEFAULT_GRPC_ADDRESS.to_string()
                } else {
                    DEFAULT_HTTP_ADDRESS.to_string()
                }
            })
        }
    };

    let address = normalize_server_address(&raw_address, cli.grpc);
    if cli.grpc {
        Transport::Grpc(address)
    } else {
        Transport::Http(address)
    }
}

fn normalize_server_address(raw_address: &str, grpc: bool) -> String {
    let trimmed = raw_address.trim();
    if trimmed.is_empty() {
        return if grpc {
            DEFAULT_GRPC_ADDRESS.to_string()
        } else {
            DEFAULT_HTTP_ADDRESS.to_string()
        };
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    }
}

fn load_token() -> Option<String> {
    fs::read_to_string(TOKEN_FILE)
        .ok()
        .map(|content| content.trim().to_owned())
        .filter(|token| !token.is_empty())
}

fn save_token(token: &str) -> Result<(), std::io::Error> {
    fs::write(TOKEN_FILE, token)
}

async fn execute_command(
    cli: &Cli,
    client: &mut BlogClient,
) -> Result<(), Box<dyn std::error::Error>> {
    match &cli.command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let auth = client
                .register(username.clone(), email.clone(), password.clone())
                .await?;
            if let Some(token) = auth.token.as_ref() {
                save_token(token)?;
                println!("Registered successfully. Token saved to {TOKEN_FILE}.");
            } else {
                println!("Registered successfully. Token was not returned by the server.");
            }
            if let Some(user) = auth.user {
                println!("User: {} (id={})", user.email, user.id);
            }
        }
        Commands::Login { username, password } => {
            let auth = client.login(username.clone(), password.clone()).await?;
            let token = auth
                .token
                .as_ref()
                .ok_or("Login succeeded but server returned no token")?;
            save_token(token)?;
            println!("Login successful. Token saved to {TOKEN_FILE}.");
            if let Some(user) = auth.user {
                println!("User: {} (id={})", user.email, user.id);
            }
        }
        Commands::Create { title, content } => {
            let post = client.create_post(title.clone(), content.clone()).await?;
            print_post(&post);
        }
        Commands::Get { id } => {
            let post = client.get_post(*id).await?;
            print_post(&post);
        }
        Commands::Update { id, title, content } => {
            if title.is_none() && content.is_none() {
                return Err("Update requires --title or --content".into());
            }

            if cli.grpc && title.is_some() {
                println!(
                    "Note: gRPC update currently sends content only; title changes may be ignored."
                );
            }

            let existing = client.get_post(*id).await?;
            let title = title.clone().unwrap_or(existing.title);
            let content = content.clone().unwrap_or(existing.content);
            let updated = client.update_post(*id, title, content).await?;
            print_post(&updated);
        }
        Commands::Delete { id } => {
            client.delete_post(*id).await?;
            println!("Post #{} deleted.", id);
        }
        Commands::List { limit, offset } => {
            let posts = client.list_posts(*limit, *offset).await?;
            print_posts(&posts, *limit, *offset);
        }
    }

    Ok(())
}

fn print_post(post: &blog_client::Post) {
    println!("Post #{}", post.id);
    println!("Title: {}", post.title);
    println!("Content: {}", post.content);
}

fn print_posts(posts: &[blog_client::Post], limit: u32, offset: u32) {
    println!(
        "Posts (limit={}, offset={}): {} returned",
        limit,
        offset,
        posts.len()
    );
    for post in posts {
        println!("- #{} {}", post.id, post.title);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_server_address_http_default() {
        assert_eq!(normalize_server_address("", false), DEFAULT_HTTP_ADDRESS);
    }

    #[test]
    fn test_normalize_server_address_grpc_default() {
        assert_eq!(normalize_server_address("", true), DEFAULT_GRPC_ADDRESS);
    }

    #[test]
    fn test_normalize_server_address_preserves_scheme() {
        assert_eq!(
            normalize_server_address("https://example.com", false),
            "https://example.com"
        );
    }

    #[test]
    fn test_normalize_server_address_adds_http_scheme() {
        assert_eq!(
            normalize_server_address("localhost:8080", false),
            "http://localhost:8080"
        );
    }

    #[test]
    fn test_build_transport_uses_server_override() {
        let cli = Cli {
            command: Commands::List {
                limit: 1,
                offset: 0,
            },
            grpc: false,
            server: Some("localhost:1234".to_string()),
        };

        match build_transport(&cli) {
            Transport::Http(address) => assert_eq!(address, "http://localhost:1234"),
            Transport::Grpc(_) => panic!("expected HTTP transport"),
        }
    }

    #[test]
    fn test_build_transport_grpc_uses_server_override() {
        let cli = Cli {
            command: Commands::List {
                limit: 1,
                offset: 0,
            },
            grpc: true,
            server: Some("grpc.example.com:50051".to_string()),
        };

        match build_transport(&cli) {
            Transport::Grpc(address) => assert_eq!(address, "http://grpc.example.com:50051"),
            Transport::Http(_) => panic!("expected gRPC transport"),
        }
    }
}
