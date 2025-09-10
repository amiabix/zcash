//! Example RPC server for ZisK-Zcash validator

use zisk_zcash_validator::rpc::http_server::HttpServer;
use zisk_zcash_validator::bridge::node_bridge::NodeConfig;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Get configuration from environment variables
    let rpc_url = env::var("ZCASH_RPC_URL").unwrap_or_else(|_| "http://localhost:8232".to_string());
    let rpc_user = env::var("ZCASH_RPC_USER").unwrap_or_else(|_| "user".to_string());
    let rpc_password = env::var("ZCASH_RPC_PASSWORD").unwrap_or_else(|_| "pass".to_string());
    let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    
    // Create node configuration
    let node_config = NodeConfig {
        rpc_url,
        rpc_user,
        rpc_password,
        cookie_file: None,
        timeout: 30,
        max_retries: 3,
        verify_tls: false,
        proxy_url: None,
        pool_size: 10,
    };
    
    // Create HTTP server
    let server = HttpServer::new(node_config)?;
    
    // Start server
    server.start(&server_addr).await?;
    
    Ok(())
}
