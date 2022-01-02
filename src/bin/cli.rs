use bytesize::to_string;
use clap::{crate_version, App, Arg, SubCommand};
use dialoguer::{console, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use sstp::client::{Getter, Sender};
use sstp::relay::Relay;
use std::error::Error;
use std::fs;
use std::path::Path;

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
      let filepath = sub_m.value_of("FILEPATH").unwrap();
      let filename = validate_filepath(filepath);
      let file = fs::read(filepath)?;

      let mut client = Sender::new(filename.to_string(), file.len(), sub_m.value_of("relay"));
      client.connect().await?;
      client.create_room().await?;
      print_after_send_help(&client);
      client.wait_for_receiver().await?;
      println!("Sending (->{})", client.peer_addr.unwrap());

      let pb = create_pb(client.size);
      for chunk in file.chunks(1_000_000) {
        client.send_chunk(chunk).await?;
        pb.inc(chunk.len().try_into()?);
      }
      pb.finish_and_clear();
      println!("Succesfully sent! ✅");
      client.finish().await?;
      Ok(())
    }

    ("get", Some(sub_m)) => {
      let mut client = Getter::new(sub_m.value_of("CODE"), sub_m.value_of("relay"));
      client.connect().await?;
      client.get_room().await?;
      let approved = req_keyboard_approval(client.filename.as_ref().unwrap(), client.size.unwrap());
      client.send_approval(approved).await?;
      if !approved {
        return Ok(());
      }
      let pb = create_pb(client.size.unwrap());
      println!("Receiving (<-{})", client.peer_addr.unwrap());
      let buffer = client
        .start_transfer(|x: usize| {
          pb.inc(x.try_into().unwrap());
        })
        .await?;
      fs::write(client.filename.unwrap(), buffer).expect("Could not write to file");
      pb.finish_and_clear();
      println!("Downloaded ✅");

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

fn print_after_send_help(client: &Sender) {
  println!(
    "Sending '{}' ({})",
    client.filename,
    to_string(
      client.size.try_into().expect("Can't parse usize to u64"),
      false
    )
  );
  println!("Code is: {}", client.code.as_ref().unwrap());
  println!("In the other computer run");
  println!("\nsstp get {}\n", client.code.as_ref().unwrap());
}

fn validate_filepath(filepath: &str) -> &str {
  let path = Path::new(filepath);
  if !path.is_file() {
    panic!("Please provide a valid file.");
  }
  path
    .file_name()
    .expect("Coudln't get filename")
    .to_str()
    .expect("Errored when parsing OsStr")
}

fn create_pb(size: usize) -> ProgressBar {
  let bar = ProgressBar::new(size.try_into().expect("Error when parsing usize to u64"));
  bar.set_style(ProgressStyle::default_bar()
    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
    .progress_chars("#>-"));
  bar
}

fn req_keyboard_approval(filename: &str, size: usize) -> bool {
  let output = format!(
    "Accept {} ({})?",
    filename,
    to_string(
      size.try_into().expect("Error when parsing usize to u64"),
      false
    )
  );
  let approved = Confirm::new()
    .with_prompt(output)
    .interact()
    .expect("Error when requesting input");
  let term = console::Term::stdout();
  term
    .clear_last_lines(1)
    .expect("Could not clear terminal line");
  approved
}
