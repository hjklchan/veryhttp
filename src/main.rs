use anyhow::{anyhow, Ok, Result};
use clap::{Parser, Subcommand};
use reqwest::{Client, Url};
use std::{collections::HashMap, str::FromStr};

#[derive(Parser, Debug)]
struct Cli {
    #[clap(subcommand)]
    subcmd: Subcommands,
}

/// ## Subcommands
///
/// Request methods
/// ```bash
/// cargo run post http://localhost name=hjkl1
/// cargo run get http://localhost
///
/// ./veryhttp.exe post http://localhost name=hjkl1
/// ./veryhttp.exe get http://localhost
/// ```
#[derive(Subcommand, Debug)]
enum Subcommands {
    Get(Get),
    Post(Post),
    // TODO: Will support as bellow methods in the following.
    // Put(()),
    // Delete(()),
    // Patch(()),
}

/// Get method
#[derive(Parser, Debug)]
struct Get {
    #[arg(value_parser = parse_url)]
    url: String,
}

/// Post method
#[derive(Parser, Debug)]
struct Post {
    #[arg(value_parser = parse_url)]
    url: String,
    #[arg(value_parser = parse_kv_pair)]
    body: Vec<KeyValuePair>,
}

#[derive(Debug, Clone)]
struct KeyValuePair {
    #[allow(dead_code)]
    k: String,
    #[allow(dead_code)]
    v: String,
}

/// Implement FromStr for KeyValuePair
impl FromStr for KeyValuePair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let mut splitted = s.split('=');
        let err = || anyhow!(format!("failed to parse body {}", s));
        Ok(Self {
            // {key=value, key=value}
            // FIXME: Need valid the value of KeyValuePair
            k: (splitted.next().ok_or_else(err)?).to_string(),
            // FIXME: Need valid the value of KeyValuePair
            v: (splitted.next().ok_or_else(err)?).to_string(),
        })
    }
}

/// parse_kv_pair
fn parse_kv_pair(s: &str) -> Result<KeyValuePair> {
    Ok(s.parse()?)
}

/// parse_url
fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?;
    Ok(s.into())
}

#[tokio::main]
async fn main() {
    // Introduce tokio to process async, dep: tokio

    // Parse command, dep: clap
    let cli: Cli = Cli::parse();
    println!("{:?}", cli);

    // Request target url, dep: reqwest
    // create a new http client
    let client = Client::new();
    // match each subcommands
    let result = match cli.subcmd {
        Subcommands::Get(ref fields) => get_handler(client, fields).await,
        Subcommands::Post(ref fields) => post_handler(client, fields).await,
    };
    _ = result;

    // Formatted output, dep: colored, jsonxf, mime

    // Error handing, dep: anyhow

    println!("Hello, world!");
}

async fn get_handler(client: Client, args: &Get) -> Result<()> {
    // Initiate a request client
    let resp = client.get::<&str>(args.url.as_ref()).send().await?;
    // Output the response text
    println!("{:?}", resp.text().await?);

    Ok(())
}

async fn post_handler(client: Client, args: &Post) -> Result<()> {
    // Prepare body
    let mut body = HashMap::<&str, &str>::new();
    for pair in &args.body {
        body.insert(&pair.k, &pair.v);
    }
    // Initiate a request client
    let resp = client
        .post::<&str>(args.url.as_ref())
        .json(&body)
        .send()
        .await?;

    println!("{:?}", resp.text().await?);

    Ok(())
}
