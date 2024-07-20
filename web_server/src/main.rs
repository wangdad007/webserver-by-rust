use mylib::ThreadPool; // 导入自定义的线程池库
use std::env; // 导入环境变量库
use std::fs; // 导入文件系统库
use std::io::{self, Read, Write}; // 导入 IO 库
use std::net::{TcpListener, TcpStream}; // 导入网络库
use std::{thread, time}; // 导入线程和时间库

// 处理客户端请求的函数
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512]; // 创建一个 512 字节的缓冲区
    if let Err(e) = stream.read(&mut buffer) {
        eprintln!("从流中读取数据失败: {}", e); // 读取失败
        return;
    }

    let get = b"GET / HTTP/1.1\r\n"; // 定义 GET 请求的字节序列
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "main.html") // 成功的 HTTP 响应和文件名
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html") // 失败的 HTTP 响应和文件名
    };

    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents, // 读取文件内容成功
        Err(e) => {
            eprintln!("读取文件 {} 失败: {}", filename, e); // 读取文件内容失败
            return;
        }
    };

    let response = format!("{}{}", status_line, contents); // 格式化 HTTP 响应

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("向流中写入数据失败: {}", e); // 写入数据失败
    }

    if let Err(e) = stream.flush() {
        eprintln!("刷新流失败: {}", e); // 刷新流失败
    }

    let te = time::Duration::from_millis(10000); // 设置处理时间为 10 秒
    thread::sleep(te); // 模拟长时间处理
}

// 主函数
fn main() -> io::Result<()> {
    env::set_var("RUST_BACKTRACE", "1"); // 设置环境变量以启用回溯

    let listener = TcpListener::bind("127.0.0.1:8080")?; // 绑定 TCP 监听器到指定地址和端口
    let pool = ThreadPool::new(5); // 创建一个包含 5 个工作线程的线程池

    for stream in listener.incoming().take(5) {
        let stream = stream?; // 接受传入的连接
        pool.execute(|| handle_client(stream)); // 将处理请求的任务交给线程池执行
    }

    Ok(())
}
