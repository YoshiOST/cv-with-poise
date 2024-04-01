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
use std::sync::Mutex; // Import Mutex from the standard library
use std::string::String; // Import String from the standard library

use std::env::var;
use std::sync::atomic::{AtomicU32};

use poise::serenity_prelude as serenity;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
pub struct Data {
    version: Arc<Mutex<String>>
} // User data, which is stored and accessible in all command invocations

// todo: make data keyvalue pair
// todo: read database to generate hashmap on reload
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

impl Data {
    // Constructor method to create a new instance of Data
    fn new(version: String) -> Self {
        // Wrap the String in Arc<Mutex<String>>
        let version = Arc::new(Mutex::new(version));
        Data { version }
    }

    // Method to modify the string inside the Mutex
    fn modify_version(&self, new_version: String) {
        // Lock the Mutex to obtain a mutable reference to the string
        let mut locked_version = self.version.lock().unwrap();
        // Modify the string
        *locked_version = new_version;
    }

    // Method to access the string inside the Mutex
    fn get_version(&self) -> String {
        // Lock the Mutex to obtain a read-only reference to the string
        let locked_version = self.version.lock().unwrap();
        // Clone the string and return it
        locked_version.clone()
    }
}

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

struct Handler {
    options: poise::FrameworkOptions<(), Error>,
    shard_manager: std::sync::Mutex<Option<std::sync::Arc<serenity::ShardManager>>>,
}

fn get_specific_bible(bible_name: &str) -> Option<Arc<Bible>> {
    let bibles = Arc::clone(&BIBLES); // Clone the Arc for thread-safe access
    let lookup_name = bible_name.to_uppercase(); // Convert the lookup name to lowercase
    bibles.get(&lookup_name).cloned()
}

fn get_bibles_names() -> String {
    BIBLES.keys().cloned().collect::<Vec<_>>().join(", ")
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands(ctx, true).await?;
    println!("Registering commands to slash commands");
    Ok(())
}


/// Prints all available bible versions
#[poise::command(slash_command, prefix_command)]
async fn versions(
    ctx: Context<'_>
) -> Result<(), Error> {
    println!("Printing bible versions");
    let response = format!("{}", get_bibles_names());
    ctx.say(response).await?;
    Ok(())
}
/// Sets the default bible version to use
#[poise::command(slash_command, prefix_command)]
async fn setver(
    ctx: Context<'_>,
    #[description = "Bible Version"] name: String,
) -> Result<(), Error> {
    let mut bible_name = String::new();
    bible_name = name.to_owned();
    if let Some(bible_arc) = get_specific_bible(&bible_name) {
        //set bible to
        println!("Setting bible version to {}", name);
        // if bible version doesn't exist. respond with fail
        let response = format!("Setting bible to {}", name);
        //set ctx.data.version
        ctx.data().modify_version(name);
        ctx.say(response).await?;
    } else {
        println!("Bible version not found: {}", bible_name);
        let response = format!("Error: Unavailable Version {}", bible_name);
        ctx.say(response).await?;
    }
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
    .setup(|ctx, _ready, framework| {
        Box::pin(async move {
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            //setting public data (like states)
            Ok(Data {
                version: Arc::new(Mutex::new(String::from("WEB"))),
            })
        })
    })
        .options(poise::FrameworkOptions {
        commands: vec![
            versions(),
            setver(),
            register(),
        ],
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
            // ChannelId::new(you message log channel).say(&ctx, "Log: Bot is online!").await.unwrap();
            ChannelId::new(1223310186383544441).say(&ctx, "Log: Bot is online!").await.unwrap();
        }
        serenity::FullEvent::Message { new_message } => {
            let mut bible_name = String::new();
            let mut scripture_reference = String::new();
            scripture_reference = new_message.content.trim().to_string();
            bible_name = data.get_version();
            if let Some(bible_arc) = get_specific_bible(&bible_name) {
                let bible: &Bible = &bible_arc;
                if let Some(verse) = bible.get_scripture(&scripture_reference) {
                    new_message
                    .reply(
                        ctx,
                        format!("{} {}: {}", verse.reference, bible_name , verse.scripture),
                    )
                    .await?;
                } else {
                }
            } else {
                println!("Bible version not found: {}", bible_name);
            }
        }
        _ => {}
    }
    Ok(())
}
