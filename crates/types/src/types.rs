use chrono::{DateTime, Local};
use crossbeam_channel::{Receiver, Sender};
use litesvm::types::TransactionMetadata;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{
    blake3::Hash,
    clock::Clock,
    epoch_info::EpochInfo,
    pubkey::Pubkey,
    transaction::{TransactionError, VersionedTransaction},
};
use solana_transaction_status::TransactionConfirmationStatus;
use std::{collections::HashMap, path::PathBuf};
use txtx_addon_network_svm::codec::subgraph::{PluginConfig, SubgraphRequest};
use txtx_core::kit::types::types::Value;
use uuid::Uuid;

pub const DEFAULT_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum RunloopTriggerMode {
    #[default]
    Clock,
    Manual,
    Transaction,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub uuid: Uuid,
    pub name: String,
    pub entries: Vec<SubgraphDataEntry>,
}

#[derive(Debug, Clone)]
pub struct SubgraphDataEntry {
    // The UUID of the entry
    pub uuid: Uuid,
    // A map of field names and their values
    pub values: HashMap<String, Value>,
}

impl SubgraphDataEntry {
    pub fn new(values: HashMap<String, Value>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            values,
        }
    }
}

#[derive(Debug)]
pub enum SubgraphEvent {
    EndpointReady,
    InfoLog(DateTime<Local>, String),
    ErrorLog(DateTime<Local>, String),
    WarnLog(DateTime<Local>, String),
    DebugLog(DateTime<Local>, String),
    Shutdown,
}

impl SubgraphEvent {
    pub fn info<S>(msg: S) -> Self
    where
        S: Into<String>,
    {
        Self::InfoLog(Local::now(), msg.into())
    }

    pub fn warn<S>(msg: S) -> Self
    where
        S: Into<String>,
    {
        Self::WarnLog(Local::now(), msg.into())
    }

    pub fn error<S>(msg: S) -> Self
    where
        S: Into<String>,
    {
        Self::ErrorLog(Local::now(), msg.into())
    }

    pub fn debug<S>(msg: S) -> Self
    where
        S: Into<String>,
    {
        Self::DebugLog(Local::now(), msg.into())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SchemaDataSourcingEvent {
    Rountrip(Uuid),
    ApplyEntry(Uuid, Vec<u8>), //, SubgraphRequest u64),
}

#[derive(Debug, Clone)]
pub enum SubgraphCommand {
    CreateSubgraph(Uuid, SubgraphRequest, Sender<String>),
    ObserveSubgraph(Receiver<SchemaDataSourcingEvent>),
    Shutdown,
}

#[derive(Debug)]
pub enum SimnetEvent {
    Ready,
    Aborted(String),
    Shutdown,
    ClockUpdate(Clock),
    EpochInfoUpdate(EpochInfo),
    BlockHashExpired,
    InfoLog(DateTime<Local>, String),
    ErrorLog(DateTime<Local>, String),
    WarnLog(DateTime<Local>, String),
    DebugLog(DateTime<Local>, String),
    PluginLoaded(String),
    TransactionReceived(DateTime<Local>, VersionedTransaction),
    TransactionProcessed(
        DateTime<Local>,
        TransactionMetadata,
        Option<TransactionError>,
    ),
    AccountUpdate(DateTime<Local>, Pubkey),
}

impl SimnetEvent {
    pub fn info<S>(msg: S) -> Self
    where
        S: Into<String>,
    {
        Self::InfoLog(Local::now(), msg.into())
    }

    pub fn warn<S>(msg: S) -> Self
    where
        S: Into<String>,
    {
        Self::WarnLog(Local::now(), msg.into())
    }

    pub fn error<S>(msg: S) -> Self
    where
        S: Into<String>,
    {
        Self::ErrorLog(Local::now(), msg.into())
    }

    pub fn debug<S>(msg: S) -> Self
    where
        S: Into<String>,
    {
        Self::DebugLog(Local::now(), msg.into())
    }

    pub fn transaction_processed(meta: TransactionMetadata, err: Option<TransactionError>) -> Self {
        Self::TransactionProcessed(Local::now(), meta, err)
    }

    pub fn transaction_received(tx: VersionedTransaction) -> Self {
        Self::TransactionReceived(Local::now(), tx)
    }

    pub fn account_update(pubkey: Pubkey) -> Self {
        Self::AccountUpdate(Local::now(), pubkey)
    }
}

pub enum TransactionStatusEvent {
    Success(TransactionConfirmationStatus),
    SimulationFailure((TransactionError, TransactionMetadata)),
    ExecutionFailure((TransactionError, TransactionMetadata)),
}

pub enum SimnetCommand {
    SlotForward,
    SlotBackward,
    UpdateClock(ClockCommand),
    UpdateRunloopMode(RunloopTriggerMode),
    TransactionReceived(
        Hash,
        VersionedTransaction,
        Sender<TransactionStatusEvent>,
        RpcSendTransactionConfig,
    ),
}

#[derive(Debug)]
pub enum PluginManagerCommand {
    LoadConfig(Uuid, PluginConfig, Sender<String>),
}

pub enum ClockCommand {
    Pause,
    Resume,
    Toggle,
    UpdateSlotInterval(u64),
}

pub enum ClockEvent {
    Tick,
    ExpireBlockHash,
}

#[derive(Clone, Debug, Default)]
pub struct SurfpoolConfig {
    pub simnet: SimnetConfig,
    pub rpc: RpcConfig,
    pub subgraph: SubgraphConfig,
    pub plugin_config_path: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct SimnetConfig {
    pub remote_rpc_url: String,
    pub slot_time: u64,
    pub runloop_trigger_mode: RunloopTriggerMode,
    pub airdrop_addresses: Vec<Pubkey>,
    pub airdrop_token_amount: u64,
}

impl Default for SimnetConfig {
    fn default() -> Self {
        Self {
            remote_rpc_url: DEFAULT_RPC_URL.to_string(),
            slot_time: 0,
            runloop_trigger_mode: RunloopTriggerMode::Clock,
            airdrop_addresses: vec![],
            airdrop_token_amount: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphConfig {}

#[derive(Clone, Debug)]
pub struct RpcConfig {
    pub bind_host: String,
    pub bind_port: u16,
    pub remote_rpc_url: String,
}

impl RpcConfig {
    pub fn get_socket_address(&self) -> String {
        format!("{}:{}", self.bind_host, self.bind_port)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubgraphPluginConfig {
    pub uuid: Uuid,
    pub ipc_token: String,
    pub subgraph_request: SubgraphRequest,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            remote_rpc_url: DEFAULT_RPC_URL.to_string(),
            bind_host: "127.0.0.1".to_string(),
            bind_port: 8899,
        }
    }
}
