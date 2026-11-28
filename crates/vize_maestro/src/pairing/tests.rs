use super::{PairingToken, Refusal, accept_client, read_paired_message};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn token() -> PairingToken {
    PairingToken::new("k3y-from-the-launcher\n").unwrap()
}

fn frame(body: &str) -> Vec<u8> {
    let mut bytes = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    bytes.extend_from_slice(body.as_bytes());
    bytes
}

fn initialize(options: &str) -> Vec<u8> {
    frame(&format!(
        r#"{{"jsonrpc":"2.0","id":0,"method":"initialize","params":{{"capabilities":{{}},"initializationOptions":{options}}}}}"#
    ))
}

fn read(bytes: &[u8]) -> Result<Vec<u8>, Refusal> {
    read_paired_message(&mut &*bytes, &token())
}

#[test]
fn a_token_is_read_without_surrounding_whitespace() {
    assert!(PairingToken::new(" \n").is_none());
    assert!(PairingToken::new("").is_none());
    let opening = initialize(r#"{"vizeToken":"k3y-from-the-launcher"}"#);
    assert_eq!(read(&opening), Ok(opening.clone()));
}

#[test]
fn the_opening_is_returned_with_the_bytes_read_after_it() {
    let mut stream = initialize(r#"{"lint":true,"vizeToken":"k3y-from-the-launcher"}"#);
    stream.extend_from_slice(&frame(r#"{"jsonrpc":"2.0","method":"initialized"}"#));
    assert_eq!(read(&stream), Ok(stream.clone()));

    let headers = frame(
        r#"{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"initializationOptions":{"vizeToken":"k3y-from-the-launcher"}}}"#,
    );
    let mut typed = b"Content-Type: application/vscode-jsonrpc; charset=utf-8\r\n".to_vec();
    typed.extend_from_slice(&headers);
    assert_eq!(read(&typed), Ok(typed.clone()));
}

#[test]
fn an_opening_without_the_token_is_refused() {
    assert_eq!(
        read(&initialize(r#"{"vizeToken":"another"}"#)),
        Err(Refusal::TokenMismatch)
    );
    assert_eq!(read(&initialize("{}")), Err(Refusal::TokenMismatch));
    assert_eq!(read(&initialize("null")), Err(Refusal::TokenMismatch));
    assert_eq!(
        read(&initialize(r#"{"vizeToken":7}"#)),
        Err(Refusal::TokenMismatch)
    );
    assert_eq!(
        read(&frame(
            r#"{"jsonrpc":"2.0","method":"initialized","params":{"initializationOptions":{"vizeToken":"k3y-from-the-launcher"}}}"#
        )),
        Err(Refusal::NotInitialize)
    );
}

#[test]
fn an_opening_that_is_not_one_message_is_refused() {
    assert_eq!(read(b""), Err(Refusal::Incomplete));
    assert_eq!(
        read(b"Content-Length: 10\r\n\r\n{}"),
        Err(Refusal::Incomplete)
    );
    assert_eq!(read(b"Content-Type: x\r\n\r\n{}"), Err(Refusal::Malformed));
    assert_eq!(
        read(b"Content-Length: -1\r\n\r\n{}"),
        Err(Refusal::Malformed)
    );
    assert_eq!(read(&frame("not json")), Err(Refusal::Malformed));
    assert_eq!(
        read(b"Content-Length: 99999999999\r\n\r\n{}"),
        Err(Refusal::Malformed)
    );
    assert_eq!(read(&vec![b'x'; 8192]), Err(Refusal::Malformed));
}

#[test]
fn the_listener_serves_the_first_connection_that_carries_the_token() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let paired = initialize(r#"{"vizeToken":"k3y-from-the-launcher"}"#);
    let expected = paired.clone();

    let clients = std::thread::spawn(move || {
        let mut stranger = TcpStream::connect(address).unwrap();
        stranger.write_all(&initialize("{}")).unwrap();
        let mut rest = Vec::new();
        // The server closes the connection without answering it.
        assert!(matches!(stranger.read_to_end(&mut rest), Ok(0) | Err(_)));
        assert!(rest.is_empty());

        let mut launcher = TcpStream::connect(address).unwrap();
        launcher.write_all(&paired).unwrap();
        launcher
    });

    let token = token();
    let (_stream, _peer, opening) =
        crate::runtime::block_on(accept_client(&listener, Some(&token))).unwrap();
    assert_eq!(opening, expected);
    drop(clients.join().unwrap());
}

#[test]
fn without_a_token_the_first_connection_is_served_unread() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let client = std::thread::spawn(move || TcpStream::connect(address).unwrap());

    let (_stream, _peer, opening) =
        crate::runtime::block_on(accept_client(&listener, None)).unwrap();
    assert!(opening.is_empty());
    drop(client.join().unwrap());
}
