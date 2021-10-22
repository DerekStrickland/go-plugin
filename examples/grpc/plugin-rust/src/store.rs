use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tonic::{Request, Response, Status};

use crate::proto::proto::kv_server::{Kv, KvServer};
use crate::proto::proto::{Empty, GetRequest, GetResponse, PutRequest};

pub struct Store {
    store: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl core::default::Default for Store {
    fn default() -> Self {
        Store {
            store: Arc::new(Mutex::new(Store::default_store())),
        }
    }
}

impl Store {
    fn default_store() -> HashMap<String, Vec<u8>> {
        let store: HashMap<String, Vec<u8>> = HashMap::new();
        store
    }
}

#[tonic::async_trait]
impl Kv for Store {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let key = request.get_ref().clone().key;
        if key.is_empty() {
            return Err(Status::invalid_argument("key not specified"));
        }

        let store_clone = Arc::clone(&self.store);
        let store = store_clone.lock().unwrap();

        match store.get(&key) {
            Some(value) => Ok(tonic::Response::new(GetResponse {
                value: value.clone().to_vec(),
            })),
            None => Err(Status::invalid_argument("key not found")),
        }
    }

    async fn put(&self, request: Request<PutRequest>) -> Result<Response<Empty>, Status> {
        let request_ref = request.get_ref().clone();
        if request_ref.key.is_empty() {
            return Err(Status::invalid_argument("key not specified"));
        }

        let store_clone = Arc::clone(&self.store);
        let mut store = store_clone.lock().unwrap();

        store.insert(request_ref.key, request_ref.value);

        Ok(Response::new(Empty {}))
    }
}
