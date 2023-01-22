use parity_wordlist::random_phrase;
use soketto::connection::{Receiver as ReceiverSk, Sender as SenderSk};
use soketto::handshake::server::Response;
use soketto::handshake::{Client, ServerResponse};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

// Server with public IP I run as a relay
const PUBLIC_RELAY: &str = "139.177.178.244:8004";

// CONNECTIONS
pub async fn start_ws_conn(
    relay_addr: SocketAddr,
) -> Result<(SenderSk<Compat<TcpStream>>, ReceiverSk<Compat<TcpStream>>), Box<dyn Error>> {
    let socket = TcpStream::connect(relay_addr).await?;
    let str_addr = relay_addr.clone().to_string();
    let mut client = Client::new(socket.compat(), &str_addr, "/");

    let (sender, receiver) = match client.handshake().await? {
        ServerResponse::Accepted { .. } => client.into_builder().finish(),
        ServerResponse::Redirect { .. } => unimplemented!("f"),
        ServerResponse::Rejected { .. } => unimplemented!("f"),
    };

    Ok((sender, receiver))
}

pub async fn start_ws_handshake(
    stream: TcpStream,
) -> Result<(SenderSk<Compat<TcpStream>>, ReceiverSk<Compat<TcpStream>>), Box<dyn Error>> {
    let mut server = soketto::handshake::Server::new(stream.compat());
    let websocket_key = {
        let req = server.receive_request().await?;
        req.key()
    };
    let accept = Response::Accept {
        key: websocket_key,
        protocol: None,
    };
    server.send_response(&accept).await?;

    let (sender, receiver) = server.into_builder().finish();
    Ok((sender, receiver))
}

// Rand
pub fn gen_room_key() -> String {
    let phrase = random_phrase(3);
    str::replace(&phrase, " ", "-")
}

// Format
pub fn calc_chunks(size: u64) -> u64 {
    ((size as f32 + 1_000_000.0 - 1.0) / 1_000_000.0) as u64
}

// Validation
pub fn parse_relay_addr(addr: Option<&str>) -> SocketAddr {
    if let Some(addr) = addr {
        let addr = addr.parse().expect("Couldn't parse the provided address");
        addr
    } else {
        PUBLIC_RELAY.parse().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_gen_room_key() {
        // Make sure the room key has the right length
        let phrase = gen_room_key();
        assert_eq!(phrase.split('-').collect::<Vec<&str>>().len(), 3);
    }

    #[test]
    fn test_default_parse_address() {
        // Return the default relay address when none specified
        let default_addr = parse_relay_addr(None);
        let relay_ip = IpAddr::V4(Ipv4Addr::new(139, 177, 178, 244));

        assert_eq!(default_addr, SocketAddr::new(relay_ip, 8004));
    }

    #[test]
    fn test_custom_parse_address() {
        // Return the default relay address when none specified
        let custom_addr = parse_relay_addr(Some("127.0.0.1:8003"));
        let relay_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        assert_eq!(custom_addr, SocketAddr::new(relay_ip, 8003));
    }
}
