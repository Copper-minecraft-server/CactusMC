//! This module manages the TCP server and how/where the packets are managed/sent.
use crate::packet::data_types::{string, varint, CodecError};
use crate::packet::Packet;
use crate::{config, gracefully_exit};
use byteorder::{BigEndian, ReadBytesExt};
use log::{debug, error, info, warn};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

/// Global buffer size when allocating a new packet (in bytes).
const BUFFER_SIZE: usize = 1024;

/// Listens for every incoming TCP connection.
pub async fn listen() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Settings::new();
    let server_address = format!("0.0.0.0:{}", config.server_port);
    let listener = TcpListener::bind(server_address).await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr).await {
                warn!("Error handling connection from {addr}: {e}");
            }
        });
    }
}

/// State of each connection. (e.g.: handshake, play, ...)
#[derive(Debug)]
enum ConnectionState {
    Handshake,
    Status,
    Login,
    Transfer,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Handshake
    }
}

/// Object representing a TCP connection.
struct Connection<'a> {
    state: ConnectionState,
    socket: &'a mut TcpStream,
}

impl<'a> Connection<'a> {
    fn new(socket: &'a mut TcpStream) -> Self {
        Self {
            state: ConnectionState::default(),
            socket,
        }
    }
}

/// Handles each connection
async fn handle_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("New connection: {addr}");
    // TODO: Maybe have a bigger/dynamic buffer?
    let mut buf = [0; BUFFER_SIZE];
    //let mut state = ConnectionState::default();
    let mut connection = Connection {
        state: ConnectionState::default(),
        socket: &mut socket,
    };

    loop {
        let read_bytes = connection.socket.read(&mut buf).await?;
        if read_bytes == 0 {
            debug!("Connection closed: {addr}");
            return Ok(()); // TODO: Why Ok? It's supposed to be an error right?
        }

        let response = handle_packet(&mut connection, &buf[..read_bytes]).await?;

        // TODO: Assure that sent packets are big endians (data types).
        connection.socket.write_all(&response).await?;
    }
}

/// Takes a packet buffer and returns a reponse.
async fn handle_packet<'a>(
    conn: &'a mut Connection<'_>,
    buffer: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    print!("\n\n\n"); // So that each logged packet is clearly visible.

    let packet = Packet::new(buffer)?;
    debug!("NEW PACKET ({}): {}", packet.len(), packet);

    // TODO: Implement a fmt::Debug trait for the Packet, such as it prints info like id, ...
    //debug!("PACKET INFO: {packet:?}");

    let packet_id_value: i32 = packet.get_id().get_value();
    debug!("PACKET ID: {packet_id_value}");

    match packet_id_value {
        0x00 => match conn.state {
            ConnectionState::Handshake => {
                warn!("Handshake packet detected!");
                let next_state = read_handshake_next_state(&packet).await?;
                println!("next_state is {:?}", &next_state);
                conn.state = next_state;

                // TODO: CLEANUP THIS MESS. Done hastily to check if it would work (it works!!).

                if let ConnectionState::Status = conn.state {
                    // Send JSON
                    let json = r#"{"version":{"name":"1.21.2","protocol":768},"players":{"max":100,"online":5,"sample":[{"name":"thinkofdeath","id":"4566e69f-c907-48ee-8d71-d7ba5aa00d20"}]},"description":{"text":"Hello, CactusMC!"},"favicon":"data:image/png;base64,<data>","enforcesSecureChat":false}"#;

                    // TODO: Make a packet builder!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

                    let lsp_packet_json = string::write(json)?;
                    let lsp_packet_id: u8 = 0x00;
                    let lsp_packet_len =
                        varint::write((lsp_packet_json.len() + size_of::<u8>()) as i32);

                    let mut lsp_packet: Vec<u8> = Vec::new();
                    lsp_packet.extend_from_slice(&lsp_packet_len);
                    lsp_packet.push(lsp_packet_id);
                    lsp_packet.extend_from_slice(&lsp_packet_json);

                    if let Err(e) = conn.socket.write_all(&lsp_packet).await {
                        error!("Failed to write JSON to client: {e}");
                    }
                }
            }
            _ => {
                warn!("packet id is 0x00 but State is not yet supported");
            }
        },
        _ => {
            warn!("Packet ID (0x{packet_id_value:X}) not yet supported.");
        }
    }

    // create a response

    let mut response = Vec::new();
    response.extend_from_slice(b"Received: ");
    response.extend_from_slice(buffer);

    print!("\n\n\n");
    Ok(response)
}

async fn read_handshake_next_state(packet: &Packet<'_>) -> Result<ConnectionState, CodecError> {
    let data = packet.get_payload();
    let mut offset: usize = 0;

    let protocol_version: (i32, usize) = varint::read(data)?;
    offset += protocol_version.1;
    info!("Handshake protocol version received: {protocol_version:?}");

    let server_address: (String, usize) = string::read(&data[offset..])?;
    offset += server_address.1;
    info!("Handshake server address received: {server_address:?}");

    // Read 2 bytes
    let mut slice = &data[offset..offset + 2]; // Create a slice of the two bytes
    let server_port: u16 = byteorder::ReadBytesExt::read_u16::<byteorder::BigEndian>(&mut slice)
        .expect("Unable to read port");
    info!("Handshake server port received: {server_port}");
    offset += 2;

    let next_state: (i32, usize) = varint::read(&data[offset..])?;
    info!("Handshake next state received: {next_state:?}");

    match next_state.1 {
        1 => {
            // 1 is for status
            debug!("Next state from handshake is status (1)");
            Ok(ConnectionState::Status)
        }
        2 => {
            // 2 is for login
            error!("Next state from handshake login (2) not yet supported!");
            gracefully_exit(0);
        }
        _ => {
            error!("Next state from handshake not yet supported!");
            gracefully_exit(0);
        }
    }
}
