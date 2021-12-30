mod client;
mod messages;
mod relay;
mod utils;

use clap::{crate_version, App, Arg, SubCommand};
use client::{Client, GetParams, SendParams};
use relay::Relay;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let matches = App::new("sstp")
    .version(crate_version!())
    .author("Steve James. <0x2t1ff@gmail.com>")
    .about("Steve's Super Transfer Program. Rust implementation of the Croc prgram.")
    .subcommand(
      SubCommand::with_name("send")
        .about("Sends a file")
        .arg(Arg::with_name("FILEPATH").index(1).required(true))
        .arg(
          Arg::with_name("relay")
            .long("relay")
            .value_name("RELAY")
            .takes_value(true),
        ),
    )
    .subcommand(
      SubCommand::with_name("get")
        .about("Downloads a file")
        .arg(Arg::with_name("CODE").index(1).required(true))
        .arg(
          Arg::with_name("relay")
            .long("relay")
            .value_name("RELAY")
            .takes_value(true),
        ),
    )
    .subcommand(SubCommand::with_name("relay").about("Starts a Relay Server"))
    .get_matches();

  match matches.subcommand() {
    ("send", Some(sub_m)) => {
      let params = SendParams::new(sub_m.value_of("FILEPATH"), sub_m.value_of("relay"));
      let client = Client::new();
      client.send(params).await?;
      Ok(())
    }
    ("get", Some(sub_m)) => {
      let params = GetParams::new(sub_m.value_of("CODE"), sub_m.value_of("relay"));
      let client = Client::new();
      client.get(params).await?;
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
