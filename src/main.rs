use tokio::net::TcpStream;
use tokio::sync::{Semaphore};
use tokio::time::{Duration, timeout};
use std::sync::Arc;
use std::net::IpAddr;

const MAX_CONCURRENT_CHECKS: usize = 5000;
const SCAN_TIMEOUT : Duration = Duration::from_millis(50);

async fn scan_port(ipAddress: IpAddr, port: i32, semaphore: Arc<Semaphore>){
    let _permit = semaphore.acquire().await.expect("Semaphore error");

    let address = format!("{}:{}", ipAddress, port);


    match timeout(SCAN_TIMEOUT, TcpStream::connect(&address)).await {
        Ok(Ok(_)) => {
            println!("[+] Port {} is OPEN", port);
        }
        _ => {  
        }
    }
}

#[tokio::main]
async fn main(){
    let ipAddress : IpAddr = "127.0.0.1".parse().expect("Invalid IP Address");
    let semaphore  = Arc::new(Semaphore::new(MAX_CONCURRENT_CHECKS));

    let mut tasks = Vec::new();

    for port in 0..65536 {

        let semaphoreClone = Arc::clone(&semaphore);
        let task = tokio::spawn(scan_port(ipAddress, port, semaphoreClone));
        tasks.push(task);
    }

    futures::future::join_all(tasks).await;
    println!("Scan complete.");

}



