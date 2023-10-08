use std::{collections::HashMap, net::UdpSocket};

// Based on https://wiki.vg/Query

fn handshake(socket: &UdpSocket) -> i32 {
  let handshake: Vec<u8> = vec![0xFE, 0xFD, 0x09, 0x00, 0x00, 0x00, 0x01];

  socket.send(handshake.as_slice()).unwrap();

  let mut buff = [0; 1024];
  let read = socket.recv(&mut buff).unwrap();
  dbg!(&buff[0..read]);
  let challenge_str = String::from_utf8_lossy(&buff[5..read - 1]).to_string();
  challenge_str.parse().unwrap()
}

fn full_status_query(socket: &UdpSocket, challenge_token: i32) -> String {
  //
  let status_request: Vec<u8> = vec![0xFE, 0xFD, 0x00, 0x00, 0x00, 0x00, 0x01];

  let mut sr_and_ct = status_request.clone();
  sr_and_ct.append(&mut challenge_token.to_be_bytes().to_vec());
  sr_and_ct.append(&mut vec![0x00, 0x00, 0x00, 0x00]);
  dbg!(&sr_and_ct);

  socket.send(sr_and_ct.as_slice()).unwrap();

  let mut buff = [0; 1024];
  let read = socket.recv(&mut buff).unwrap();
  // Pruning the 16 first bytes, that are identifiers
  String::from_utf8_lossy(&buff[16..read]).to_string()
}

#[derive(Debug)]
struct Status {
  #[allow(unused)]
  infos: HashMap<String, String>,
  players: Vec<String>,
}

fn deserialize_status(status: &str) -> Status {
  const PLAYER_SECTION_START: &str = "\x00\x01player_\x00\x00";
  let infos_end = status.find(PLAYER_SECTION_START).unwrap();
  let infos: HashMap<_, _> = status[0..infos_end]
    .split('\0')
    .collect::<Vec<_>>()
    .chunks_exact(2)
    .map(|kv| (kv[0].to_string(), kv[1].to_string()))
    .collect();
  // between the end of infos and start of players there is a 10 padding bytes
  let players: Vec<String> = status[infos_end + PLAYER_SECTION_START.len()..]
    .trim_end_matches("\0\0")
    .split('\0')
    .map(String::from)
    .collect();

  Status { infos, players }
}

pub fn list_players() -> Vec<String> {
  let socket = UdpSocket::bind("0.0.0.0:34254").unwrap();
  socket.connect("todo:25565").unwrap();

  let challenge = handshake(&socket);
  let status = full_status_query(&socket, challenge);
  deserialize_status(&status).players
}

#[test]
fn test_status_packet() {
  let socket = UdpSocket::bind("0.0.0.0:34254").unwrap();
  socket.connect("todo:25565").unwrap();

  let challenge = dbg!(handshake(&socket));
  let status = dbg!(full_status_query(&socket, challenge));
  dbg!(deserialize_status(&status));
}
