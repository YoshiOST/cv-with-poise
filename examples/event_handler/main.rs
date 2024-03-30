// chapterverse imports
use bible::csv_import::bible_import;
use bible::scripture::bible::Bible;
use helpers::env_variables::get_env_variable;
// use helpers::print_color::PrintCommand;
use lazy_static::lazy_static;
use std::collections::HashMap;
// use std::io::Write;
use std::sync::Arc;
use std::{env, fs};

use std::env::var;
use std::sync::atomic::{AtomicU32};

use poise::serenity_prelude as serenity;

mod helpers;

lazy_static! {
    static ref BIBLES: Arc<HashMap<String, Arc<Bible>>> = {
        let import_bibles_path = get_env_variable("IMPORT_BIBLES_PATH", "bibles");

        let bibles_directory = match env::current_dir().map(|dir| dir.join(import_bibles_path)) {
            Ok(dir) => dir,
            Err(e) => {
                println!("Error getting current directory: {}", e);
                return Arc::new(HashMap::new());
            }
        };

        let mut bibles = HashMap::new();

        let files = match fs::read_dir(bibles_directory) {
            Ok(files) => files,
            Err(e) => {
                println!("Error reading bibles directory: {}", e);
                return Arc::new(HashMap::new());
            }
        };

        for file in files {
            let entry = match file {
                Ok(entry) => entry,
                Err(e) => {
                    println!("Error reading file in directory: {}", e);
                    continue; // Skip to the next iteration
                }
            };

            if entry.path().is_file()
                && entry.path().extension().and_then(|s| s.to_str()) == Some("csv")
            {
                let file_stem = entry
                    .path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string()
                    .to_uppercase();
                let file_path = entry.path().to_string_lossy().to_string();
                match bible_import(&entry.path().to_string_lossy()) {
                    Ok(imported_bible) => {
                        bibles.insert(file_stem, Arc::new(imported_bible));
                    }
                    Err(err) => {
                        println!("Error running import for file '{}': {}", file_path, err);
                    }
                }
            }
        }

        Arc::new(bibles)
    };
}

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(unused)]
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    poise_mentions: AtomicU32,
}

fn get_specific_bible(bible_name: &str) -> Option<Arc<Bible>> {
    let bibles = Arc::clone(&BIBLES); // Clone the Arc for thread-safe access
    let lookup_name = bible_name.to_uppercase(); // Convert the lookup name to lowercase
    bibles.get(&lookup_name).cloned()
}

fn get_bibles_names() -> String {
    BIBLES.keys().cloned().collect::<Vec<_>>().join(", ")
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn versions(
    ctx: Context<'_>
) -> Result<(), Error> {
    let response = format!("{}", get_bibles_names());
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    // let mut bible_name = String::new();
    let token = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                versions()
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    poise_mentions: AtomicU32::new(0),
                })
            })
        })
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            let mut bible_name = String::new();
            let mut scripture_reference = String::new();
            scripture_reference = new_message.content.trim().to_string();
            bible_name = "WEB".to_owned();
            if let Some(bible_arc) = get_specific_bible(&bible_name) {
                let bible: &Bible = &bible_arc;
                if let Some(verse) = bible.get_scripture(&scripture_reference) {
                    new_message
                        .reply(
                            ctx,
                            format!("{}: {}", verse.reference, verse.scripture),
                        )
                        .await?;
                } else {
                }
            } else {
                println!("Bible version not found: {}", bible_name);
            }
            // if new_message.content.to_lowercase().contains("poise")
            //     && new_message.author.id != ctx.cache.current_user().id
            // {
            //     let old_mentions = data.poise_mentions.fetch_add(1, Ordering::SeqCst);
            //     new_message
            //         .reply(
            //             ctx,
            //             format!("Poise has been mentioned {} times", old_mentions + 1),
            //         )
            //         .await?;
            // }
        }
        _ => {}
    }
    Ok(())
}
