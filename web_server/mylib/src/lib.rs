use std::sync::mpsc; // 导入用于消息传递的多生产者单消费者通道
use std::sync::Arc; // 导入原子引用计数智能指针
use std::sync::Mutex; // 导入互斥锁
use std::thread; // 导入线程库

// 定义工作线程结构体
struct Worker {
    id: usize, // 线程编号
    thread: Option<thread::JoinHandle<()>>, // 可选的线程句柄，用于线程的生命周期管理
}

// 为 Worker 实现方法
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        // 创建并启动一个新线程
        let thread = thread::spawn(move || loop {
            // 获取锁并接收消息
            let message = match receiver.lock() {
                Ok(receiver) => match receiver.recv() {
                    Ok(message) => message, // 成功接收消息
                    Err(e) => {
                        eprintln!("Worker {} 接收消息失败: {:?}", id, e); // 接收消息失败
                        break;
                    }
                },
                Err(e) => {
                    eprintln!("Worker {} 获取锁失败: {:?}", id, e); // 获取锁失败
                    break;
                }
            };

            // 处理接收到的消息
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} 接收到一个任务; 正在执行。", id);
                    job(); // 执行任务
                },
                Message::Terminate => {
                    println!("Worker {} 收到终止指令。", id);
                    break; // 退出循环，终止线程
                },
            }
        });

        Worker { 
            id, // 初始化线程编号
            thread: Some(thread), // 初始化线程句柄
        }
    }
}

// 定义线程池结构体
pub struct ThreadPool {
    workers: Vec<Worker>, // 工作线程的向量
    sender: mpsc::Sender<Message>, // 发送消息的通道发送端
}

// 定义任务类型，使用动态分发的闭包
type Job = Box<dyn FnOnce() + Send + 'static>;

// 定义消息枚举类型
enum Message {
    NewJob(Job), // 新任务消息
    Terminate, // 终止消息
}

// 为 ThreadPool 实现方法
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0); // 确保线程池大小大于 0

        let (sender, receiver) = mpsc::channel(); // 创建消息通道
        let receiver = Arc::new(Mutex::new(receiver)); // 使用 Arc 和 Mutex 包装接收端，以实现线程安全

        let mut workers = Vec::with_capacity(size); // 创建具有预分配容量的工作线程向量

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver))); // 创建并启动工作线程
        }

        ThreadPool { workers, sender } // 返回初始化的线程池
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f); // 将任务封装为 Box<dyn FnOnce() + Send + 'static>
        if let Err(e) = self.sender.send(Message::NewJob(job)) {
            eprintln!("发送任务到线程池失败: {:?}", e); // 发送任务失败
        }
    }
}

// 为 ThreadPool 实现 Drop 特性
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("发送终止消息到所有工作线程。");

        for _ in &self.workers {
            if let Err(e) = self.sender.send(Message::Terminate) {
                eprintln!("发送终止消息失败: {:?}", e); // 发送终止消息失败
            }
        }

        println!("正在关闭所有工作线程。");

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                if let Err(e) = thread.join() {
                    eprintln!("等待工作线程终止失败: {:?}", e); // 等待线程终止失败
                }
            }
        }
    }
}
