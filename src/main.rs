mod client;
mod messages;
mod relay;
mod utils;

use clap::{App, Arg, SubCommand};
use client::Client;
use relay::Relay;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let matches = App::new("sstp")
    .version("0.2")
    .author("Steve James. <0x2t1ff@gmail.com>")
    .about("Steve's Super Transfer Program. Rust implementation of the Croc prgram.")
    .subcommand(
      SubCommand::with_name("send")
        .about("Sends a file")
        .arg(Arg::with_name("FILEPATH").index(1).required(true)),
    )
    .subcommand(
      SubCommand::with_name("get")
        .about("Downloads a file")
        .arg(Arg::with_name("CODE").index(1).required(true)),
    )
    .subcommand(SubCommand::with_name("relay").about("Starts a Relay Server"))
    .get_matches();

  match matches.subcommand() {
    ("send", Some(sub_m)) => {
      let filepath = sub_m.value_of("FILEPATH").unwrap();
      let client = Client::new();
      client.send(filepath).await?;
      Ok(())
    }
    ("get", Some(sub_m)) => {
      let code = sub_m.value_of("CODE").unwrap();
      let client = Client::new();
      client.get(code).await?;
      Ok(())
    }
    ("relay", _) => {
      Relay::start().await?;
      Ok(())
    }
    _ => {
      println!("{}", matches.usage.unwrap());
      Ok(())
    }
  }
}
