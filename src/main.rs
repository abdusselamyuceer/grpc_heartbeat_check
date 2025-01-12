mod client;

pub mod proto{
    tonic::include_proto!("heartbeat");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("heartbeat_descriptor");
}
use chrono::{NaiveDate};
use chrono::{Utc, DateTime, TimeZone};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{self, Duration};

use tonic::{Request, Response, Status};
use tonic::transport::Server;
use proto::heart_beat_server::HeartBeatServer;

use proto::heart_beat_server::HeartBeat;
use crate::proto::{HeartBeatRequest, HeartBeatResponse};

use serde::Deserialize;
use std::fs;
use prost_types::Timestamp;
use serde::de::Unexpected::Option;

#[derive(Debug, Deserialize)]
struct Config {
    heartbeat_timeout: u64, // Timeout in seconds
}

fn read_yaml(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let config: Config = serde_yaml::from_str(&content)?;
    Ok(config)
}

#[derive(Debug, Default)]
pub struct HeartBeatService {
    active_nodes: Arc<Mutex<HashMap<String, DateTime<Utc> >>>,
}

#[tonic::async_trait]
impl HeartBeat for HeartBeatService {


 async fn send_heartbeat( &self,
                          request: tonic::Request<proto::HeartBeatRequest>,
 ) -> Result<tonic::Response<proto::HeartBeatResponse>, tonic::Status>  {
     let req = request.into_inner();
     let mut active_nodes = self.active_nodes.lock().unwrap();

     // Convert to UTC datetime at midnight (00:00:00)
     let last_seen_datetime = chrono::Utc::now();;
     active_nodes.insert(req.node_id.clone(), last_seen_datetime);

     println!("Received heartbeat from node: {}, last seen: {:?}", req.node_id, last_seen_datetime);

     Ok(Response::new(HeartBeatResponse {
         status: true
 }))
    }
}
async fn monitor_nodes(active_nodes: Arc<Mutex<HashMap<String,DateTime<Utc> >>>) {
    let config = read_yaml("config/conf.yaml");

    let timeout = Duration::from_secs(config.unwrap().heartbeat_timeout);
    loop {
        time::sleep(Duration::from_secs(5)).await;
        let now = chrono::Utc::now().timestamp();

        let mut active_nodes = active_nodes.lock().unwrap();
        active_nodes.retain(|node_id, &mut last_seen| {
            if now - last_seen.timestamp() > timeout.as_secs() as i64 {
                println!("Node {} timed out {:?}  {:?}  {:?} ", node_id, now, last_seen,timeout.as_secs());
                false
            } else {
                true
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let heartbeat_service = HeartBeatService::default();

    let active_nodes = heartbeat_service.active_nodes.clone();

    tokio::spawn(async move {
        monitor_nodes(active_nodes).await;
    });

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(HeartBeatServer::new(heartbeat_service))
        .serve(addr)
        .await?;

    Ok(())
}
