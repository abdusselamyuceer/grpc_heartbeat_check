
use std::error::Error;
use std::time::Duration;
use std::thread::sleep;

use proto::heart_beat_client::{HeartBeatClient, };
pub mod proto {
    tonic::include_proto!("heartbeat");
}

async fn create_client(node_id:String, timestamp: i64) ->Result<(), Box<dyn Error>> {
    let url = "http://[::1]:50051";
    let mut client = HeartBeatClient::connect(url).await?;
    loop {
        let req = proto::HeartBeatRequest { node_id: node_id.clone() };
        let request = tonic::Request::new(req);

        let response = client.send_heartbeat(request).await?;

        println!("Response from {}: {:?}",node_id,  response.get_ref().status, );
        sleep(Duration::from_secs(5));
    }
Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let task1 = tokio::spawn(async {
        println!("Task is running...");
        create_client("1".to_string(), 5).await;
        println!("Task completed!");
    });

    let task2 = tokio::spawn(async {
        println!("Task is running...");
        create_client("2".to_string(), 7).await;
        println!("Task completed!");
    });
    let _ = tokio::join!(task1, task2);
    Ok(())
}
