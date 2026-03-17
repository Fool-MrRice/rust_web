use std::net::TcpStream;
use std::io::{Read, Write,self};

fn main() {
    // client通过TcpStream连接到localhost:3000
    let mut stream = TcpStream::connect("127.0.0.1:3000").unwrap();
    println!("Client connected!");

    // 从std输入读取输入到buffer中
    let stdin = io::stdin();//stdin() 返回标准输入的句柄，可以反复使用
    loop {
        let mut input = String::new();
        print!("echo server等待输入(输入-1退出):");
        io::stdout().flush().expect("failed to flush stdout");
        stdin.read_line(&mut input).unwrap();
        if let Ok(-1)=input.trim().parse::<i32>() { break; }
        stream.write(&input.into_bytes()).unwrap();
        // buffer缓冲
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        println!("echo Server reply: {}", String::from_utf8_lossy(&buffer));
    }


}
