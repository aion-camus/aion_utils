extern crate acore_bytes as bytes;
/// this is a standalone account model for Aion Blockchain
///
extern crate aion_types;
extern crate lru_cache;

#[macro_use]
extern crate log;
extern crate blake2b;
extern crate rlp;
#[macro_use]
extern crate rlp_derive;
extern crate db as kvdb;
extern crate patricia_trie as trie;
extern crate parking_lot;

mod accounts;
mod traits;
mod account_db;

pub use accounts::{FVMAccount, AVMAccount};
