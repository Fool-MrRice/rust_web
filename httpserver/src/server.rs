use http::httprequest::HttpRequest;
use std::io::Read;
use std::net::TcpListener;

pub struct Server<'a> {
    socket_addr: &'a str,
}

impl<'a> Server<'a> {
    ///new通过参数&str进行初始化Server的运行端口
    pub fn new(socket_addr: &'a str) -> Self {
        Server { socket_addr }
    }
    /// run ：Server调用自己的run开始运行
    pub fn run(&self) {
        // 利用TcpListener使得localhost:3000被server监听
        let listener = TcpListener::bind(self.socket_addr).unwrap();
        println!("Running on {}...", self.socket_addr);
        // 利用for加incoming持续得到listener的信息
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            //每次有一个新的界面都会打印一次
            println!("Connection established!");
            // 建立缓冲
            let mut read_buffer = [0; 1024];
            // 从读取的stream中读信息到buffer中
            let _ = stream.read(&mut read_buffer);
            // 从buffer中序列化出自定义的HttpRequest格式
            let request: HttpRequest = String::from_utf8(read_buffer.to_vec()).unwrap().into();
            // server的run中通过调用router::Router::route方法进一步的处理HttpRequest请求，传入request和可变引用TcpStream进行处理
            crate::router::Router::route(request, &mut stream);
        }
    }
}
