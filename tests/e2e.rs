use std::time::Instant;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Seek, SeekFrom, Write},
    thread,
};

use sstp::client::{Getter, Sender};

/// Test 126 bytes file transfer.
#[tokio::test]
async fn test_e2e_tiny() {
    let file = File::open("tests/test_file_tiny.txt").unwrap();
    e2e_with_file(file).await;
}

/// Test 60 KB file transfer
#[tokio::test]
async fn test_e2e_small() {
    let file = File::open("tests/test_file_small.png").unwrap();
    e2e_with_file(file).await;
}

/// Test 2.9 MB file transfer
#[tokio::test]
async fn test_e2e_medium() {
    let file = File::open("tests/test_file_medium.png").unwrap();
    e2e_with_file(file).await;
}

// Simple e2e test using default relay
async fn e2e_with_file(file: File) {
    // Sender setup
    let sender = setup_sender(&file).await;
    let room_code = sender.code.as_ref().unwrap().clone();

    // Getter setup
    let getter = setup_getter(room_code).await;

    // Start and time transfer
    let start = Instant::now();
    let handle1 = thread::spawn(|| finish_sending(sender, file));
    let handle2 = thread::spawn(|| finish_getting(getter));

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
    let duration = start.elapsed();
    println!("Transfer took {:?}", duration);
}

// SETUPS
async fn setup_sender(file: &File) -> Sender {
    // Create sender client
    let mut sender = Sender::new(
        "this_doesnt_matter.txt".to_string(),
        file.metadata().unwrap().len().try_into().unwrap(),
        None,
    );
    // Start connection with relay + upgrade to websocket
    sender.connect().await.unwrap();

    // Create a room on the relay
    sender.create_room().await.unwrap();
    sender
}

async fn setup_getter(room_code: String) -> Getter {
    // Create getter client for a specific room
    let mut getter = Getter::new(Some(room_code.as_str()), None);

    // Start connection with relay + upgrade to websocket
    getter.connect().await.unwrap();

    // Try to get the room belonging to this client
    getter.get_room().await.unwrap();

    // Always approve the "approval_request" step
    getter.send_approval(true).await.unwrap();

    getter
}

// TRANSFERS
async fn finish_sending(mut client: Sender, file: File) {
    // Waiting on a recv call
    client.wait_for_receiver().await.unwrap();

    let mut br = BufReader::new(file);

    client
        .start_transfer(&mut br, |_: u64| {})
        .await
        .expect("boop");
    client.finish().await.unwrap();
}

async fn finish_getting(mut client: Getter) {
    let mut file = File::create("tests/out").unwrap();
    let mut bw = BufWriter::new(file.try_clone().unwrap());
    client.start_transfer(&mut bw, |_: u64| {}).await.unwrap();
    bw.flush().unwrap();
    assert_eq!(client.size.unwrap(), file.seek(SeekFrom::End(0)).unwrap());
    std::fs::remove_file("tests/out").expect("error removing test file");
}
