use std::{fs::File, io::{BufWriter, Write, Seek, SeekFrom}, thread};

use sstp::client::{Sender, Getter};


#[tokio::test]
async fn test_hello() {
    assert_eq!(1+1, 2);
}

#[tokio::test]
async fn test_e2e() {
    let filename = "test_file.txt";
    let file = File::open(filename).unwrap();

    println!("SENDER: Starting setup");
    // Sender setup
    let mut sender = Sender::new(
        filename.to_string(),
        file.metadata().unwrap().len().try_into().unwrap(),
        None,
    );
    sender.connect().await.unwrap();
    sender.create_room().await.unwrap();
    println!("SENDER: Finished creating room, waiting for receiver...");
    let room_code = sender.code.as_ref().unwrap().clone();

    // Getter setup
    let mut getter = Getter::new(Some(room_code.as_str()), None);
    getter.connect().await.unwrap();
    getter.get_room().await.unwrap();
    getter.send_approval(true).await.unwrap();

    let handle1 = thread::spawn(||{finish_sending(sender, file)});
    let handle2 = thread::spawn(|| {finish_getting(getter)});

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;

}

async fn finish_sending(mut client: Sender, mut file: File) {
      // Waiting on a recv call
      client.wait_for_receiver().await.unwrap();
      println!("SENDER: Sending (->{})", client.peer_addr.unwrap());
      client
        .start_transfer(&mut file, |_: u64| { }).await.expect("boop");
      println!("SENDER: Succesfully sent! ✅");
      client.finish().await.unwrap();
}

async fn finish_getting(mut client: Getter) {
      println!("GETTER: ");
      let mut file = File::create("tests/out.txt").unwrap();
      let mut bw = BufWriter::new(file.try_clone().unwrap());
      println!("GETTER: Receiving (<-{})", client.peer_addr.unwrap());
      client
        .start_transfer(&mut bw, |_: u64| {})
        .await.unwrap();
      bw.flush().unwrap();
      println!("GETTER: Downloaded ✅");
      assert_eq!(client.size.unwrap(), file.seek(SeekFrom::End(0)).unwrap())
}