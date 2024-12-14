use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{sleep, Duration, timeout};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use parking_lot::RwLock;
use log::{info, error, warn, debug};

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const MAX_RECONNECT_ATTEMPTS: u32 = 3;

#[derive(Debug)]
pub struct Node {
    config: Arc<NodeConfig>,
    keypair: Keypair,
    rpc_client: RpcClient,
    peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
    tx: broadcast::Sender<Message>,
    shutdown: mpsc::Sender<()>,
}

impl Node {
    pub async fn new(config: NodeConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let keypair = Keypair::new();
        let rpc_client = RpcClient::new_with_commitment(
            "https://api.mainnet-beta.solana.com".to_string(),
            CommitmentConfig::confirmed(),
        );

        let (tx, _) = broadcast::channel(100);
        let (shutdown_tx, _) = mpsc::channel(1);
        
        Ok(Node {
            config: Arc::new(config),
            keypair,
            rpc_client,
            peers: Arc::new(RwLock::new(HashMap::new())),
            tx,
            shutdown: shutdown_tx,
        })
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
       
        self.verify_stake().await?;

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| {
                error!("Failed to bind to {}: {}", addr, e);
                e
            })?;
        
        info!("Node listening on {}", addr);

        
        let peers = Arc::clone(&self.peers);
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(60)).await;
                Self::cleanup_disconnected_peers(Arc::clone(&peers)).await;
            }
        });

       
        self.connect_to_bootstrap_nodes().await?;

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((socket, addr)) => {
                            let tx = self.tx.clone();
                            let peers = Arc::clone(&self.peers);
                            
                            debug!("New connection from {}", addr);
                            
                            tokio::spawn(async move {
                                match timeout(CONNECTION_TIMEOUT, Self::handle_connection(socket, addr, tx, peers)).await {
                                    Ok(result) => {
                                        if let Err(e) = result {
                                            error!("Error handling connection from {}: {}", addr, e);
                                        }
                                    }
                                    Err(_) => {
                                        error!("Connection handling timeout for {}", addr);
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            error!("Error accepting connection: {}", e);
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
        }
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Shutting down node...");
        let _ = self.shutdown.send(()).await;
        Ok(())
    }

    async fn verify_stake(&self) -> Result<(), Box<dyn std::error::Error>> {
        let balance = self.rpc_client
            .get_balance(&self.keypair.pubkey())
            .await?;

        if balance < self.config.min_stake {
            return Err("Insufficient stake amount".into());
        }

        Ok(())
    }

    async fn connect_to_bootstrap_nodes(&self) -> Result<(), Box<dyn std::error::Error>> {
        for node in &self.config.bootstrap_nodes {
            let mut attempts = 0;
            while attempts < MAX_RECONNECT_ATTEMPTS {
                match TcpStream::connect(node).await {
                    Ok(stream) => {
                        info!("Connected to bootstrap node: {}", node);
                        let peers = Arc::clone(&self.peers);
                        if let Err(e) = self.handle_outbound_connection(stream, peers).await {
                            error!("Error handling connection to {}: {}", node, e);
                            attempts += 1;
                            sleep(RECONNECT_DELAY).await;
                            continue;
                        }
                        break;
                    }
                    Err(e) => {
                        warn!("Failed to connect to bootstrap node {}: {}", node, e);
                        attempts += 1;
                        if attempts < MAX_RECONNECT_ATTEMPTS {
                            sleep(RECONNECT_DELAY).await;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_connection(
        socket: TcpStream,
        addr: SocketAddr,
        tx: broadcast::Sender<Message>,
        peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        socket.set_nodelay(true)?;
        
        
        let keepalive = socket2::TcpKeepalive::new()
            .with_time(Duration::from_secs(60))
            .with_interval(Duration::from_secs(10));
        
        let socket2 = socket2::SockRef::from(&socket);
        socket2.set_tcp_keepalive(&keepalive)?;
        
        
        peers.write().insert(addr, PeerInfo::new(addr));
        
        
        Ok(())
    }

    async fn handle_outbound_connection(
        &self,
        stream: TcpStream,
        peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let addr = stream.peer_addr()?;
        stream.set_nodelay(true)?;
        
        
        let keepalive = socket2::TcpKeepalive::new()
            .with_time(Duration::from_secs(60))
            .with_interval(Duration::from_secs(10));
        
        let socket2 = socket2::SockRef::from(&stream);
        socket2.set_tcp_keepalive(&keepalive)?;
        
        
        peers.write().insert(addr, PeerInfo::new(addr));
        
        
        Ok(())
    }

    async fn cleanup_disconnected_peers(peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>) {
        let mut peers = peers.write();
        peers.retain(|addr, peer| {
            if !peer.is_connected() {
                warn!("Removing disconnected peer: {}", addr);
                false
            } else {
                true
            }
        });
    }
}
