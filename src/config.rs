use serde::{Deserialize,Serialize};

/// This is a struct to hold configuration information. Right now the only things in here are
/// database connection params, but I'll also need to store API keys and AI login stuff when
/// the time comes.
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct ServerConfig {
    /// Just the host name or IP address for the database connection
    pub database_file: String,

    /// The name of the session storage table
    #[serde(default="ServerConfig::default_session_table")]
    pub session_table_name:String,

    /// The number of seconds to wait between scans to delete expired sessions
    #[serde(default="ServerConfig::default_session_cleanup_interval_seconds")]
    pub session_cleanup_interval_seconds:i64,
    
    /// The amount of inactive time before a session expires
    #[serde(default="ServerConfig::default_session_timeout_seconds")]
    pub session_timeout_seconds: i64,
}

impl ServerConfig {
    fn default_session_timeout_seconds() -> i64 {60*60*24*5}
    fn default_session_cleanup_interval_seconds() -> i64 {5}
    fn default_session_table() -> String { "sessions".into() }
}

