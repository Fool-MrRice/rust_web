use std::net::TcpListener;
use std::io::{Read, Write};

fn main() {
    // 利用TcpListener使得localhost:3000被server监听
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    println!("Running on 127.0.0.1:3000...");
    // 利用for加incoming持续得到listener的信息
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        println!("Connection established!");
        loop {
            // 建立缓冲
            let mut buffer= [0; 1024];
            // 从读取的stream中读信息到buffer中
            match stream.read(&mut buffer) {
                Ok(_) => {
                    // 把buffer中的内容原封不动的返回给客户端
                    stream.write(&buffer).unwrap();
                }
                Ok(0)=>break,
                Err(e)=>{println!("Error: {}", e);break;}

            }

        }
    }

}
