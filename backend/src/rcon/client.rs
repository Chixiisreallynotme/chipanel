use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, warn};

use crate::error::AppError;

const SERVERDATA_AUTH: i32 = 3;
const SERVERDATA_EXECCOMMAND: i32 = 2;
const SERVERDATA_RESPONSE_VALUE: i32 = 0;

#[derive(Debug)]
pub struct RconPacket {
    pub id: i32,
    pub packet_type: i32,
    pub payload: String,
}

pub struct RconClient {
    host: String,
    port: u16,
    password: String,
    stream: Option<TcpStream>,
    request_id: i32,
}

impl RconClient {
    /// Connect to an RCON server with the given host, port, and password.
    /// Performs handshaking and authentication with a 5-second timeout limit.
    pub async fn connect(host: &str, port: u16, password: &str) -> Result<Self, AppError> {
        let mut client = RconClient {
            host: host.to_string(),
            port,
            password: password.to_string(),
            stream: None,
            request_id: 1,
        };

        client.reconnect().await?;
        Ok(client)
    }

    /// Re-establishes the TCP connection and authenticates.
    pub async fn reconnect(&mut self) -> Result<(), AppError> {
        self.stream = None;
        let addr = format!("{}:{}", self.host, self.port);

        let connect_fut = TcpStream::connect(&addr);
        let mut stream = tokio::time::timeout(Duration::from_secs(5), connect_fut)
            .await
            .map_err(|_| AppError::InternalError(format!("RCON connection to {} timed out after 5s", addr)))?
            .map_err(|e| AppError::InternalError(format!("Failed to connect to RCON at {}: {}", addr, e)))?;

        if let Err(err) = self.send_auth(&mut stream).await {
            self.stream = None;
            return Err(err);
        }

        self.stream = Some(stream);
        Ok(())
    }

    /// Authenticates with the RCON server.
    async fn send_auth(&mut self, stream: &mut TcpStream) -> Result<(), AppError> {
        let auth_id = self.next_id();
        let pkt_bytes = encode_packet(auth_id, SERVERDATA_AUTH, &self.password);

        let write_fut = stream.write_all(&pkt_bytes);
        tokio::time::timeout(Duration::from_secs(5), write_fut)
            .await
            .map_err(|_| AppError::InternalError("RCON authentication write timed out".to_string()))?
            .map_err(|e| AppError::InternalError(format!("RCON auth write error: {}", e)))?;

        // Read until auth response packet
        let start = std::time::Instant::now();
        let mut read_buf = Vec::with_capacity(4096);
        loop {
            if start.elapsed() >= Duration::from_secs(5) {
                return Err(AppError::InternalError("RCON authentication response timed out".to_string()));
            }

            let remaining = Duration::from_secs(5).saturating_sub(start.elapsed());
            let pkt = tokio::time::timeout(remaining, read_packet(stream, &mut read_buf))
                .await
                .map_err(|_| AppError::InternalError("RCON authentication read timed out".to_string()))??;

            if pkt.id == -1 {
                return Err(AppError::AuthError("RCON authentication failed: invalid password".to_string()));
            }

            if pkt.id == auth_id {
                debug!("RCON authentication successful");
                return Ok(());
            }
        }
    }

    /// Executes an RCON command and returns the response output.
    /// Handles multi-packet responses and auto-reconnects on connection drops.
    pub async fn exec(&mut self, command: &str) -> Result<String, AppError> {
        if self.stream.is_none() {
            self.reconnect().await?;
        }

        match self.exec_internal(command).await {
            Ok(output) => Ok(output),
            Err(err) => {
                warn!("RCON command execution failed ({}); attempting reconnect...", err);
                self.reconnect().await?;
                self.exec_internal(command).await
            }
        }
    }

    async fn exec_internal(&mut self, command: &str) -> Result<String, AppError> {
        let req_id = self.next_id();
        let dummy_id = self.next_id();

        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| AppError::InternalError("RCON client is not connected".to_string()))?;

        let cmd_pkt = encode_packet(req_id, SERVERDATA_EXECCOMMAND, command);
        let dummy_pkt = encode_packet(dummy_id, SERVERDATA_RESPONSE_VALUE, "");

        let write_fut = async {
            stream.write_all(&cmd_pkt).await?;
            stream.write_all(&dummy_pkt).await?;
            stream.flush().await?;
            Ok::<(), std::io::Error>(())
        };

        tokio::time::timeout(Duration::from_secs(5), write_fut)
            .await
            .map_err(|_| AppError::InternalError("RCON command send timed out after 5s".to_string()))?
            .map_err(|e| AppError::InternalError(format!("Failed to send RCON command: {}", e)))?;

        let mut output = String::new();
        let start_time = std::time::Instant::now();
        let mut read_buf = Vec::with_capacity(4096);

        loop {
            if start_time.elapsed() >= Duration::from_secs(5) {
                if !output.is_empty() {
                    break;
                }
                return Err(AppError::InternalError("RCON response timed out after 5s".to_string()));
            }

            let remaining = Duration::from_secs(5).saturating_sub(start_time.elapsed());
            let pkt = match tokio::time::timeout(remaining, read_packet(stream, &mut read_buf)).await {
                Ok(Ok(pkt)) => pkt,
                Ok(Err(err)) => {
                    if !output.is_empty() {
                        break;
                    }
                    return Err(err);
                }
                Err(_) => {
                    if !output.is_empty() {
                        break;
                    }
                    return Err(AppError::InternalError("RCON read timeout".to_string()));
                }
            };

            if pkt.id == -1 {
                return Err(AppError::AuthError("RCON session invalidated".to_string()));
            }

            if pkt.id == dummy_id {
                break;
            }

            if pkt.id == req_id {
                output.push_str(&pkt.payload);
            }
        }

        Ok(output)
    }

    fn next_id(&mut self) -> i32 {
        self.request_id = self.request_id.wrapping_add(1);
        if self.request_id <= 0 || self.request_id > 1_000_000 {
            self.request_id = 1;
        }
        self.request_id
    }
}

/// Encodes an RCON packet according to protocol spec:
/// Length (i32 LE) | Request ID (i32 LE) | Type (i32 LE) | Payload (UTF-8 bytes) | Null Byte | Null Byte
fn encode_packet(id: i32, packet_type: i32, payload: &str) -> Vec<u8> {
    let payload_bytes = payload.as_bytes();
    let length = 4 + 4 + payload_bytes.len() as i32 + 2;
    let mut buf = Vec::with_capacity(4 + length as usize);

    buf.extend_from_slice(&length.to_le_bytes());
    buf.extend_from_slice(&id.to_le_bytes());
    buf.extend_from_slice(&packet_type.to_le_bytes());
    buf.extend_from_slice(payload_bytes);
    buf.push(0x00);
    buf.push(0x00);

    buf
}

/// Safely reads and parses an RCON packet from an async reader into a reusable buffer without panics.
async fn read_packet<R: AsyncReadExt + Unpin>(
    reader: &mut R,
    buf: &mut Vec<u8>,
) -> Result<RconPacket, AppError> {
    let length = reader.read_i32_le().await.map_err(|e| {
        AppError::InternalError(format!("Failed to read RCON packet length: {}", e))
    })?;

    if length < 10 || length > 65536 {
        return Err(AppError::InternalError(format!(
            "Invalid RCON packet length header: {}",
            length
        )));
    }

    let body_len = length as usize;
    buf.clear();
    buf.resize(body_len, 0);

    reader.read_exact(buf).await.map_err(|e| {
        AppError::InternalError(format!("Failed to read full RCON packet body: {}", e))
    })?;

    if buf.len() < 10 {
        return Err(AppError::InternalError("Malformed RCON packet body".to_string()));
    }

    let id = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let packet_type = i32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);

    let payload_slice = &buf[8..];
    let payload_len = payload_slice
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(payload_slice.len());

    let payload = String::from_utf8_lossy(&payload_slice[..payload_len]).into_owned();

    Ok(RconPacket {
        id,
        packet_type,
        payload,
    })
}
