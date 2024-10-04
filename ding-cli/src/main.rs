use anyhow::Result;
use clap::builder::ArgPredicate;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum, ValueHint};
use colored_json::to_colored_json_auto;
use crossterm::style::Stylize;
use iocraft::ElementExt;
use serde::Serialize;
use url::Url;

use ding_rs::{Bookmark, BookmarkRequest, BookmarksRequest, DingClient, TagRequest, TagsRequest};

#[derive(ValueEnum, Clone, Default)]
enum OutputFormat {
    #[default]
    Human,
    Json,
    FlattenJson,
    Csv,
}

trait ToOutput: Serialize {
    fn to_human_format(&self) -> Result<String>;
    fn to_csv_format(&self) -> Result<String>;
    fn to_json_format(&self) -> Result<String> {
        Ok(to_colored_json_auto(&self)?)
    }
    fn to_flatten_json_format(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
    fn to_format(&self, format: OutputFormat) -> Result<String> {
        match format {
            OutputFormat::Human => self.to_human_format(),
            OutputFormat::Json => self.to_json_format(),
            OutputFormat::FlattenJson => self.to_flatten_json_format(),
            OutputFormat::Csv => self.to_csv_format(),
        }
    }
}

impl ToOutput for Bookmark {
    fn to_human_format(&self) -> Result<String> {
        let title = match (&self.website_title, &self.title) {
            (_, Some(title)) => title,
            (Some(title), None) => title,
            (None, None) => &self.url.to_string(),
        };
        let description = match (&self.website_description, &self.description) {
            (_, Some(description)) => description,
            (Some(description), None) => description,
            (None, None) => "-",
        };
        let notes = if let Some(notes) = &self.notes {
            notes
        } else {
            "-"
        };
        let (width, _) = crossterm::terminal::size()?;
        let formated_description = iocraft::prelude::element! {
            iocraft::prelude::Box(
                border_style: iocraft::prelude::BorderStyle::None,
                max_width: width,
                padding_left: iocraft::prelude::Padding::Length(2),
                padding_right: iocraft::prelude::Padding::Length(2),
            ) {
                iocraft::prelude::Text(content: format!("{} {}", "Description:".to_string().magenta(), description))
            }
        }.to_string();
        let formated_notes = iocraft::prelude::element! {
            iocraft::prelude::Box(
                border_style: iocraft::prelude::BorderStyle::None,
                max_width: width,
                padding_left: iocraft::prelude::Padding::Length(2),
                padding_right: iocraft::prelude::Padding::Length(2),
            ) {
                iocraft::prelude::Text(content: format!("{} {}", "Notes:".to_string().magenta(), notes))
            }
        }.to_string();
        Ok(format!(
            "{} {}\n  {} {}\n  {} {}\n{}{}",
            format!(
                "(ID: {}{}{})",
                self.id,
                if self.is_archived { ",📦" } else { "" },
                if self.unread { ",📕" } else { ",📖" }
            )
            .green()
            .bold(),
            title.clone().bold().blue(),
            "Url:".to_string().magenta(),
            self.url,
            "Tags:".to_string().magenta(),
            self.tag_names.join(" "),
            formated_description,
            formated_notes,
        ))
    }
    fn to_csv_format(&self) -> Result<String> {
        todo!()
    }
}

impl ToOutput for Vec<Bookmark> {
    fn to_human_format(&self) -> Result<String> {
        Ok(self
            .iter()
            .map(|x| x.to_human_format())
            .collect::<Result<Vec<String>>>()?
            .join("\n"))
    }
    fn to_csv_format(&self) -> Result<String> {
        todo!()
    }
}

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

    #[arg(short, long, global = true, default_value_t)]
    verbose: bool,

    #[arg(short = 'F', long, global = true, default_value_t, value_enum)]
    output_format: OutputFormat,
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
            match (id, url) {
                (Some(id), None) => client.archive_bookmark(*id).await?,
                (None, Some(_url)) => {
                    todo!()
                }
                _ => {
                    todo!()
                }
            };
            let bookmark: Bookmark = client.bookmark(id.expect("LOOL")).await?;
            println!("{}", bookmark.to_format(cli.output_format)?);
        }
        Commands::Unarchive { id, url } => {
            let client = create_client(&cli)?;
            match (id, url) {
                (Some(id), None) => client.unarchive_bookmark(*id).await?,
                (None, Some(_url)) => {
                    todo!()
                }
                _ => {
                    todo!()
                }
            };
            let bookmark: Bookmark = client.bookmark(id.expect("LOOL")).await?;
            println!("{}", bookmark.to_format(cli.output_format)?);
        }
        Commands::Delete { id, url } => {
            let client = create_client(&cli)?;
            match (id, url) {
                (Some(id), None) => client.delete_bookmark(*id).await?,
                (None, Some(_url)) => {
                    todo!()
                }
                _ => {
                    todo!()
                }
            };
            let bookmark: Bookmark = client.bookmark(id.expect("LOOL")).await?;
            println!("{}", bookmark.to_format(cli.output_format)?);
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
            println!("{}", bookmark.to_format(cli.output_format)?);
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
            println!("{}", bookmarks.to_format(cli.output_format)?);
        }
    };
    Ok(())
}
