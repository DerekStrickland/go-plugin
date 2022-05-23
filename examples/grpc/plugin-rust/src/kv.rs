use std::fs;

use tonic::{Request, Response, Status};

use crate::proto::proto::kv_server::Kv;
use crate::proto::proto::{Empty, GetRequest, GetResponse, PutRequest};
// use futures_util::AsyncWriteExt;
use tokio::io;
use tokio::io::AsyncWriteExt;

pub struct KV {}

impl core::default::Default for KV {
    fn default() -> Self {
        KV {}
    }
}

#[tonic::async_trait]
impl Kv for KV {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let key = request.get_ref().clone().key;
        io::stdout().write_all(b"getting key").await?;
        if key.is_empty() {
            return Err(Status::invalid_argument("key not specified"));
        }

        let mut file_name: String = "kv_".to_owned();
        file_name.push_str(key.as_str());

        let value = fs::read(file_name).expect("Unable to read file");
        Ok(tonic::Response::new(GetResponse {
            value: value.clone().to_vec(),
        }))
    }

    async fn put(&self, request: Request<PutRequest>) -> Result<Response<Empty>, Status> {
        let request_ref = request.get_ref().clone();
        if request_ref.key.is_empty() {
            return Err(Status::invalid_argument("key not specified"));
        }

        let mut file_name: String = "kv_".to_owned();
        file_name.push_str(request_ref.key.as_str());

        fs::write(file_name, request_ref.value).expect("Unable to write file");
        Ok(Response::new(Empty {}))
    }
}
