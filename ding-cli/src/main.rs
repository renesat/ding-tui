use anyhow::Result;
use clap::builder::ArgPredicate;
use clap::ValueHint;
use clap::{CommandFactory, Parser, Subcommand};
use colored_json::to_colored_json_auto;
use url::Url;

use ding_rs::{BookmarkRequest, BookmarksRequest, DingClient, TagRequest, TagsRequest};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short='H', long, env = "DING_HOST", global = true, hide_env_values = true, value_hint = ValueHint::Url)]
    host: Option<Url>,

    #[arg(long, env = "DING_TOKEN", global = true, hide_env_values = true)]
    token: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Unarchive {
        #[arg(short, long, conflicts_with = "url")]
        id: Option<u64>,

        #[arg(short, long, conflicts_with = "id")]
        url: Option<Url>,
    },
    Archive {
        #[arg(short, long, conflicts_with = "url")]
        id: Option<u64>,

        #[arg(short, long, conflicts_with = "id")]
        url: Option<Url>,
    },
    Delete {
        #[arg(short, long, conflicts_with = "url")]
        id: Option<u64>,

        #[arg(short, long, conflicts_with = "id")]
        url: Option<Url>,
    },
    AddTag {
        #[arg(short, long)]
        name: String,
    },
    Add {
        url: Url,

        #[arg(short = 'T', long)]
        title: Option<String>,

        #[arg(short, long)]
        description: Option<String>,

        #[arg(short, long)]
        notes: Option<String>,

        #[arg(short, long)]
        is_archived: Option<bool>,

        #[arg(short, long)]
        unread: Option<bool>,

        #[arg(short, long)]
        shared: Option<bool>,

        #[arg(short, long)]
        tag_names: Option<Vec<String>>,
    },
    Completion {
        shell: clap_complete::Shell,
    },
    Tags {
        #[arg(short, long)]
        limit: Option<u64>,

        #[arg(short, long)]
        offset: Option<u64>,

        #[arg(
            short,
            long,
            conflicts_with_all = ["limit", "offset"]
        )]
        all: bool,
    },
    Bookmarks {
        #[arg(short, long)]
        query: Option<String>,

        #[arg(
            short,
            long,
            default_value_if("all", ArgPredicate::IsPresent, None),
            default_value = "100"
        )]
        limit: Option<u64>,

        #[arg(short, long)]
        offset: Option<u64>,

        #[arg(
            short,
            long,
            conflicts_with_all = ["limit", "offset"]
        )]
        all: bool,

        #[arg(short = 'A', long)]
        archived: bool,
    },
}

fn create_client(cli: &Cli) -> Result<DingClient> {
    Ok(DingClient::new(
        cli.host.clone().expect("Not Found URL"),
        cli.token.clone().expect("Not Found Token").to_string(),
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Archive { id, url } => {
            let client = create_client(&cli)?;
            let bookmark = match (id, url) {
                (Some(id), None) => client.archive_bookmark(*id).await?,
                (None, Some(_url)) => {
                    todo!()
                }
                _ => {
                    todo!()
                }
            };
            println!("{}", to_colored_json_auto(&bookmark)?);
        }
        Commands::Unarchive { id, url } => {
            let client = create_client(&cli)?;
            let bookmark = match (id, url) {
                (Some(id), None) => client.unarchive_bookmark(*id).await?,
                (None, Some(_url)) => {
                    todo!()
                }
                _ => {
                    todo!()
                }
            };
            println!("{}", to_colored_json_auto(&bookmark)?);
        }
        Commands::Delete { id, url } => {
            let client = create_client(&cli)?;
            let bookmark = match (id, url) {
                (Some(id), None) => client.delete_bookmark(*id).await?,
                (None, Some(_url)) => {
                    todo!()
                }
                _ => {
                    todo!()
                }
            };
            println!("{}", to_colored_json_auto(&bookmark)?);
        }
        Commands::Tags { all, limit, offset } => {
            let client = create_client(&cli)?;
            let tags = match all {
                true => client.all_tags(Default::default()).await?,
                false => {
                    client
                        .tags(TagsRequest {
                            limit: *limit,
                            offset: *offset,
                        })
                        .await?
                        .results
                }
            };
            println!("{}", to_colored_json_auto(&tags)?);
        }
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            let cmd_name: String = cmd.get_name().into();
            clap_complete::generate(*shell, &mut cmd, cmd_name, &mut std::io::stdout());
        }
        Commands::AddTag { name } => {
            let client = create_client(&cli)?;
            let tag = client.create_tag(TagRequest { name: name.clone() }).await?;
            println!("{}", to_colored_json_auto(&tag)?);
        }
        Commands::Add {
            url,
            title,
            description,
            notes,
            is_archived,
            unread,
            shared,
            tag_names,
        } => {
            let client = create_client(&cli)?;
            let req = BookmarkRequest {
                url: Some(url.clone()),
                title: title.clone(),
                description: description.clone(),
                notes: notes.clone(),
                is_archived: *is_archived,
                unread: *unread,
                shared: *shared,
                tag_names: tag_names.clone(),
            };
            let bookmark = client.create_bookmark(req).await?;
            println!("{}", to_colored_json_auto(&bookmark)?);
        }
        Commands::Bookmarks {
            query,
            limit,
            offset,
            all,
            archived,
        } => {
            let client = create_client(&cli)?;
            let bookmarks = match (all, archived) {
                (true, true) => {
                    client
                        .all_archived(BookmarksRequest {
                            query: query.clone(),
                            ..Default::default()
                        })
                        .await?
                }
                (true, false) => {
                    client
                        .all_bookmarks(BookmarksRequest {
                            query: query.clone(),
                            ..Default::default()
                        })
                        .await?
                }
                (false, false) => {
                    client
                        .bookmarks(BookmarksRequest {
                            query: query.clone(),
                            limit: *limit,
                            offset: *offset,
                        })
                        .await?
                        .results
                }
                (false, true) => {
                    client
                        .archived(BookmarksRequest {
                            query: query.clone(),
                            limit: *limit,
                            offset: *offset,
                        })
                        .await?
                        .results
                }
            };
            println!("{}", to_colored_json_auto(&bookmarks)?);
        }
    };
    Ok(())
}
