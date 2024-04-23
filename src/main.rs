use anyhow::{anyhow, Ok, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use mime::Mime;
use reqwest::{header, Client, Response, Url};
use std::{collections::HashMap, str::FromStr};

#[derive(Parser, Debug)]
#[clap(version)]
struct Cli {
    #[clap(subcommand)]
    subcmd: Subcommands,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    Get(Get),
    Post(Post),
    // TODO: The following will support as below methods in the future.
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
async fn main() -> Result<()> {
    // Introduce tokio to process async, dep: tokio

    // Parse command, dep: clap
    let cli: Cli = Cli::parse();
    println!("{:?}", cli);

    // Request target url, dep: reqwest
    // initial and add some http headers
    let mut headers = header::HeaderMap::new();
    headers.insert("X-POWERED-BY", "Rust Http Cli".parse()?);
    headers.insert(header::USER_AGENT, "Veryhttp by Rust".parse()?);
    // create a new http client
    let client = Client::builder().default_headers(headers).build()?;
    // match each subcommands
    let result = match cli.subcmd {
        Subcommands::Get(ref fields) => get_handler(client, fields).await,
        Subcommands::Post(ref fields) => post_handler(client, fields).await,
    };
    _ = result;

    // Formatted output, dep: colored, jsonxf, mime

    // Error handing, dep: anyhow

    Ok(())
}

async fn get_handler(client: Client, args: &Get) -> Result<()> {
    // Initiate a request client
    let resp: Response = client.get::<&str>(args.url.as_ref()).send().await?;
    // Output the response text
    Ok(print_resp(resp).await?)
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

    Ok(print_resp(resp).await?)
}

/// ## get_content_type
///
/// get content type from response
fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|value| value.to_str().unwrap().parse().unwrap())
}

/// print_status
/// TODO: Make the formatting more pretty..
///
/// print version and http status code
fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status().as_str().blue());
    println!("{}\n", status);
}

/// print_headers
/// TODO: Make the formatting more pretty..
///
/// print headers of response
fn print_headers(resp: &Response) {
    for (k, v) in resp.headers().into_iter() {
        // TODO: This line should be optimistic**.
        println!("{} = {:?}", (k.as_ref() as &str).green(), v);
    }
    print!("\n");
}

/// print_body
/// TODO: Make the formatting more pretty..
///
/// print body of response
fn print_body(m: Option<Mime>, body: &String) {
    match m {
        // if body type is json
        Some(v) if v == mime::APPLICATION_JSON => println!("{}", jsonxf::pretty_print(body).unwrap().cyan()),
        // or else direct output html
        _ => println!("{}", body),
    }
}

async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);

    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);

    Ok(())
}
