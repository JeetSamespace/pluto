use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use pingora::http::RequestHeader;
use once_cell::sync::Lazy;
use crate::gateway::config::read_gateway_config;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use wasmtime::*;

// Define a generic Claims struct
#[derive(Debug, Serialize, Deserialize)]
struct Claims<T> {  // expiration time
    #[serde(flatten)]
    custom: T,    // custom claims
}

pub struct Blacklist {
    blacklisted_users: Arc<RwLock<HashSet<i32>>>,
    id_key: String, // The key to use for identifying users in the JWT
    token_secret: String,
}

impl Blacklist {
    pub fn new(id_key: String, token_secret: String) -> Self {
        let mut initial_blacklist = HashSet::new();
        initial_blacklist.insert(456);  // Hard-code user123 into the blacklist
        Blacklist {
            blacklisted_users: Arc::new(RwLock::new(initial_blacklist)),
            id_key,
            token_secret,
        }
    }

    pub async fn add_to_blacklist(&self, user_id: i32) {
        let mut blacklist = self.blacklisted_users.write().await;
        blacklist.insert(user_id);
        println!("Added user to blacklist: {}", user_id);
    }

    pub async fn check_blacklist(&self, req_header: &RequestHeader) -> bool {
        // Extract the JWT from the Authorization header
        let auth_header = match req_header.headers.get("Authorization") {
            Some(header) => header,
            None => {
                println!("No Authorization header found");
                return false; // No Authorization header, let the request proceed
            }
        };

        let token = match auth_header.to_str() {
            Ok(t) => t.trim_start_matches("Bearer "),
            Err(_) => {
                println!("Invalid Authorization header value");
                return false; // Invalid header value, let the request proceed
            }
        };

        println!("Extracted token: {}", token);

        // Decode and validate the JWT
        let token_data = match decode::<Claims<serde_json::Value>>(
            token,
            &DecodingKey::from_secret(self.token_secret.as_ref()),  // Replace with your actual secret
            &Validation::new(Algorithm::HS256)
        ) {
            Ok(data) => data,
            Err(e) => {
                println!("Failed to decode token: {:?}", e);
                return false; // Invalid token, let the request proceed
            }
        };
        println!("token_data: {:?}", token_data);
        println!("id_key: {}", self.id_key);

        // Extract the user_id from the JWT
        let user_id = token_data.claims.custom[&self.id_key].as_i64().unwrap_or(0) as i32;
        println!("Decoded user_id: {}", user_id);

        // Get the current blacklist
        let blacklist = self.blacklisted_users.read().await;
        let blacklisted_ids: Vec<i32> = blacklist.iter().cloned().collect();

        // Check if the user_id is blacklisted using WASM
        let is_blacklisted = check_wasm_blacklist(user_id, &blacklisted_ids).unwrap_or(false);
        println!("User {} is {}blacklisted", user_id, if is_blacklisted { "" } else { "not " });
        is_blacklisted
    }
}

// Update the function signature to take the user_id and blacklisted_ids
fn check_wasm_blacklist(user_id: i32, blacklisted_ids: &[i32]) -> Result<bool, Box<dyn std::error::Error>> {
    // Load the WebAssembly module
    let wasm_path = Path::new("blacklist.wasm");
    let mut file = File::open(wasm_path)?;
    let mut wasm_bytes = Vec::new();
    file.read_to_end(&mut wasm_bytes)?;

    // Create a WebAssembly engine and store
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());

    // Compile the WebAssembly module
    let module = Module::new(&engine, wasm_bytes)?;

    // Create a memory instance with initial size of 4 pages (256KB)
    let memory = Memory::new(&mut store, MemoryType::new(4, None))?;

    // Create a linker and add the imports
    let mut linker = Linker::new(&engine);
    linker.define(&mut store, "env", "memory", memory)?;

    // Instantiate the WebAssembly module
    let instance = linker.instantiate(&mut store, &module)?;

    // Get the exported function from WASM
    let is_blacklisted = instance.get_typed_func::<(i32, i32, i32), i32>(&mut store, "is_blacklisted")?;

    // Allocate memory in WASM for the blacklisted IDs
    let alloc_size = (blacklisted_ids.len() * std::mem::size_of::<i32>()) as i32;
    let memory_data = memory.data_mut(&mut store);
    let memory_size = memory_data.len() as i32;

    let blacklist_ptr = 0; // start at address 0
    if alloc_size > memory_size {
        return Err("Not enough memory for the blacklist".into());
    }
    let blacklist_slice = &mut memory_data[blacklist_ptr as usize..(blacklist_ptr + alloc_size) as usize];
    for (i, &id) in blacklisted_ids.iter().enumerate() {
        let bytes = id.to_le_bytes();
        blacklist_slice[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }

    // Check the user_id against the blacklist in WASM
    let is_blacklisted_result = is_blacklisted.call(&mut store, (user_id, blacklist_ptr, blacklisted_ids.len() as i32))?;

    Ok(is_blacklisted_result == 1)
}

pub async fn handle_blacklist(req_header: &RequestHeader) -> bool {
    // Create a static Blacklist instance with the configuration
    static BLACKLIST: Lazy<Blacklist> = Lazy::new(|| {
        let config = read_gateway_config().expect("Failed to read gateway config");
        Blacklist::new(config.gateway.blacklist.id_key, config.gateway.blacklist.token_secret)
    });

    println!("Checking blacklist for request");
    // Check if the user is blacklisted
    let result = BLACKLIST.check_blacklist(req_header).await;
    println!("Blacklist check result: {}", result);
    result
}
