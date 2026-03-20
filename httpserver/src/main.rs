pub mod handler;
pub mod router;
pub mod server;
use server::Server;
fn main() {
    // 新建一个Server，并赋予地址
    let server = Server::new("127.0.0.1:3000");
    // 启动Server
    server.run();
}
