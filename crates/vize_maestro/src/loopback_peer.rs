//! Which account opened a connection to the TCP transport.
//!
//! `vize lsp --port` listens on loopback, which every account on the machine
//! can reach. The server takes the first connection whose socket belongs to
//! the account running it, so another account cannot hand it a workspace.

use std::net::SocketAddr;

/// Whether the socket at `peer`, connected to `local`, belongs to this account.
///
/// Linux lists every socket's owner in `/proc/net/tcp`. Elsewhere the peer
/// account calls (`getpeereid`, `LOCAL_PEERCRED`) answer only for Unix-domain
/// sockets, so a TCP connection is taken as before and the pairing token
/// (`crate::pairing`) is what ties the server to its client.
#[cfg(target_os = "linux")]
pub(crate) fn opened_by_this_account(local: SocketAddr, peer: SocketAddr) -> bool {
    use std::os::unix::fs::MetadataExt;
    let Ok(own) = std::fs::metadata("/proc/self").map(|meta| meta.uid()) else {
        return false;
    };
    let table = if peer.is_ipv4() {
        "/proc/net/tcp"
    } else {
        "/proc/net/tcp6"
    };
    std::fs::read_to_string(table)
        .is_ok_and(|rows| socket_owner(&rows, peer.port(), local.port()) == Some(own))
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn opened_by_this_account(_local: SocketAddr, _peer: SocketAddr) -> bool {
    true
}

/// The owner of the socket bound to `from` and connected to `to`, read from
/// the rows of `/proc/net/tcp`: `sl local rem st queues timer retrnsmt uid …`,
/// with addresses written as `HEXADDR:HEXPORT`.
#[cfg(any(target_os = "linux", test))]
fn socket_owner(rows: &str, from: u16, to: u16) -> Option<u32> {
    let port = |address: &str| {
        let (_, port) = address.rsplit_once(':')?;
        u16::from_str_radix(port, 16).ok()
    };
    rows.lines().skip(1).find_map(|row| {
        let mut fields = row.split_whitespace();
        let local = fields.nth(1)?;
        let remote = fields.next()?;
        if port(local)? != from || port(remote)? != to {
            return None;
        }
        fields.nth(4)?.parse().ok()
    })
}

#[cfg(test)]
mod tests {
    use super::socket_owner;

    const ROWS: &str = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:2537 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 101 1
   1: 0100007F:D431 0100007F:2537 01 00000000:00000000 00:00000000 00000000  1001        0 102 1
   2: 0100007F:2537 0100007F:D431 01 00000000:00000000 00:00000000 00000000  1000        0 103 1
";

    #[test]
    fn the_owner_is_read_from_the_peer_side_row() {
        // Peer 127.0.0.1:54321 (0xD431) connected to the server on 9527 (0x2537).
        assert_eq!(socket_owner(ROWS, 0xD431, 0x2537), Some(1001));
        assert_eq!(socket_owner(ROWS, 0x2537, 0xD431), Some(1000));
        assert_eq!(socket_owner(ROWS, 0xD432, 0x2537), None);
        assert_eq!(socket_owner("", 0xD431, 0x2537), None);
    }
}
