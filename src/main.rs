mod messages;
mod relay;
mod sender;

use clap::{App, Arg, SubCommand};
use relay::Relay;
use sender::Sender;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let matches = App::new("stp")
    .version("0.1")
    .author("Steve James. <0x2t1ff@gmail.com>")
    .about("Steve's Transfer Program. Rust implementation of the Croc prgram.")
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
      println!("Sending file {}", filepath);
      let client = Sender::new();
      client.send(filepath).await?;
      Ok(())
    }
    ("get", Some(sub_m)) => {
      let code = sub_m.value_of("CODE").unwrap();
      let client = Sender::new();
      client.get(code).await?;
      Ok(())
    }
    ("relay", _) => {
      let mut server = Relay::new();
      server.start().await?;
      Ok(())
    }
    _ => {
      println!("{}", matches.usage.unwrap());
      Ok(())
    }
  }
}
