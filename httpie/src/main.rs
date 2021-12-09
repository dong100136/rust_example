use clap::{AppSettings, Clap};
use anyhow::Result;
use anyhow::anyhow;
use std::str::FromStr;
use std::collections::HashMap;
use reqwest::{header, Client, Response, Url};
use colored::*;
use mime::Mime;

/// A nvive httpie implementation with Rust, can you imagine how easy it is?
#[derive(Clap,Debug)]
#[clap(version="1.0", author="stone")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand{
    Get(Get),
    Post(Post)
}

/// feed get with an url and we will retrieve the response for you
#[derive(Clap, Debug)]
struct Get{
    #[clap(parse(try_from_str = parse_url))]
    url:String,
}

/// feed post with an url and optional key=value pairs. We will post the data
/// as JSON, and retrieve the response for you
#[derive(Clap, Debug)]
struct Post{
    #[clap(parse(try_from_str = parse_url))]
    url: String,
    #[clap(parse(try_from_str = parse_kv_pair))]
    body: Vec<KvPair>,
}

#[derive(Debug,PartialEq)]
struct KvPair {
    k: String,
    v: String
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to paese {}", s));
        Ok (Self {
            k: (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

fn parse_kv_pair(s:&str) -> Result<KvPair> {
    Ok(s.parse()?)
}

fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?;
    Ok(s.into())
}

async fn get(client:Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    // println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }
    let resp = client.post(&args.url).json(&body).send().await?;
    // println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}

fn print_headers(resp: &Response){
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }
    print!("\n");
}

fn print_status(resp:&Response){
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

fn print_body(m: Option<Mime>, body:&String){
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
        }
        _ => println!("{}", body),
    }
}

async fn print_resp(resp:Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers().get(header::CONTENT_TYPE).map(|v| v.to_str().unwrap().parse().unwrap())
}

#[tokio::main]
async fn main()->Result<()> {
    let opts: Opts = Opts::parse();
    let client = Client::new();

    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) => post(client, args).await?,
    };

    Ok(result)
}


// 仅在 cargo test 时才编译
#[cfg(test)]
mod tests {   
     use super::*;  

     #[test]    
     fn parse_url_works() {
        assert!(parse_url("abc").is_err());        
        assert!(parse_url("http://abc.xyz").is_ok());        
        assert!(parse_url("https://httpbin.org/post").is_ok());    
    }    

    #[test]    
    fn parse_kv_pair_works() {
        assert!(parse_kv_pair("a").is_err());
        assert_eq!(            
            parse_kv_pair("a=1").unwrap(),
            KvPair {                
                k: "a".into(),                
                v: "1".into()
            }        
        );        
        assert_eq!(
            parse_kv_pair("b=").unwrap(),
            KvPair {                
                k: "b".into(),                
                v: "".into()            
            }        
        );    
    }
}