pub mod handler;
pub mod server;
pub mod router;
use server::Server;
fn main() {
    let server = Server::new("127.0.0.1:3000");
    server.run();
}
