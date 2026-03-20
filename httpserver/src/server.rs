use http::httprequest::HttpRequest;
use std::io::Read;
use std::net::TcpListener;


pub struct Server<'a> {
    socket_addr: &'a str,
}


impl<'a> Server<'a> {
    pub fn new(socket_addr: &'a str) -> Self {
        Server {
            socket_addr: socket_addr,
        }
    }
    pub fn run(&self) {
        // 利用TcpListener使得localhost:3000被server监听
        let listener = TcpListener::bind(self.socket_addr).unwrap();
        println!("Running on {}...", self.socket_addr);
        // 利用for加incoming持续得到listener的信息
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            println!("Connection established!");
            // 建立缓冲
            let mut read_buffer = [0; 1024];
            // 从读取的stream中读信息到buffer中
            let _ = stream.read(&mut read_buffer);
            let request: HttpRequest = String::from_utf8(read_buffer.to_vec()).unwrap().into();
            crate::router::Router::route(request, &mut stream);
        }
    }
}
