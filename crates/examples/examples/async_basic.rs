use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("开始异步任务...");
    sleep(Duration::from_millis(100)).await;
    println!("异步任务完成!");
}
