use super::TcpStream;
use crate::driver::Socket;
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
};

/// A TCP socket server, listening for connections.
///
/// You can accept a new connection by using the [`accept`](`TcpListener::accept`)
/// method.
///
/// # Examples
///
/// ```
/// use tokio_uring::net::TcpListener;
/// use tokio_uring::net::TcpStream;
///
/// fn main() {
///     let listener = TcpListener::bind("127.0.0.1:2345").unwrap();
///
///     tokio_uring::start(async move {
///         let tx_fut = TcpStream::connect("127.0.0.1:2345");
///
///         let rx_fut = listener.accept();
///
///         let (tx, (rx, _)) = tokio::try_join!(tx_fut, rx_fut).unwrap();
///
///         tx.write(b"test" as &'static [u8]).await.0.unwrap();
///
///         let (_, buf) = rx.read(vec![0; 4]).await;
///
///         assert_eq!(buf, b"test");
///     });
/// }
/// ```
pub struct TcpListener {
    inner: Socket,
}

impl TcpListener {
    /// Creates a new TcpListener, which will be bound to the specified address.
    ///
    /// The returned listener is ready for accepting connections.
    ///
    /// Binding with a port number of 0 will request that the OS assigns a port
    /// to this listener. The port allocated can be queried via the `local_addr`
    /// method.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let mut sockets = addr.to_socket_addrs()?;
        while let Some(socket_addr) = sockets.next() {
            let socket = Socket::bind(socket_addr, libc::SOCK_STREAM)?;
            socket.listen(1024)?;
            return Ok(TcpListener { inner: socket });
        }
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Could not connect to supplied sockets",
        ))
    }

    /// Accepts a new incoming connection from this listener.
    ///
    /// This function will yield once a new TCP connection is established. When
    /// established, the corresponding [`TcpStream`] and the remote peer's
    /// address will be returned.
    ///
    /// [`TcpStream`]: struct@crate::net::TcpStream
    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (socket, socket_addr) = self.inner.accept().await?;
        let stream = TcpStream { inner: socket };
        let socket_addr = socket_addr.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Could not get socket IP address",
            )
        })?;
        Ok((stream, socket_addr))
    }
}
