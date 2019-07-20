use serde::{Serialize, Deserialize};

pub mod process;
pub mod rt;
pub use self::process::Process;

/// A message that can be sent between threads, over IPC, or across the network.
pub trait Message: Serialize + for<'de> Deserialize<'de> + Send {}

impl<M> Message for M where M: Serialize + for<'de> Deserialize<'de> + Send {}
