use lru_cache::LruCache;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;

use aion_types::{H128, U128, H256, U256, Address};
use bytes::{Bytes, ToPretty};
use generic::{Account, Filth};
use traits::CommonAccount;
use blake2b::{BLAKE2B_EMPTY, BLAKE2B_NULL_RLP, blake2b};
use rlp::*;
use trie;
use trie::{Trie, SecTrieDB, TrieFactory};

use kvdb::{DBValue, HashStore};

/// Basic account type.
#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct BasicAccount {
    /// Nonce of the account.
    pub nonce: U256,
    /// Balance of the account.
    pub balance: U256,
    /// Storage root of the account.
    pub storage_root: H256,
    /// Code hash of the account.
    pub code_hash: H256,
}

const STORAGE_CACHE_ITEMS: usize = 8192;

type FVMCache = (RefCell<LruCache<H128, H128>>, RefCell<LruCache<H128, H256>>);
type FVMStorageChange = (HashMap<H128, H128>, HashMap<H128, H256>);
pub type FVMAccount = Account<FVMCache, FVMStorageChange>;

impl FVMAccount {
    fn empty_storage_cache() -> FVMCache {
        (RefCell::new(LruCache::new(STORAGE_CACHE_ITEMS)), RefCell::new(LruCache::new(STORAGE_CACHE_ITEMS)))
    }

    fn empty_storage_change() -> FVMStorageChange {
        (HashMap::new(), HashMap::new())
    }
}

impl From<BasicAccount> for FVMAccount {
    fn from(basic: BasicAccount) -> Self {
        Account {
            balance: basic.balance,
            nonce: basic.nonce,
            storage_root: basic.storage_root,
            storage_cache: Self::empty_storage_cache(),
            storage_changes: (HashMap::new(), HashMap::new()),
            code_hash: basic.code_hash,
            code_size: None,
            code_cache: Arc::new(vec![]),
            code_filth: Filth::Clean,
            address_hash: Cell::new(None),
            empty_but_commit: false,
        }
    }
}

impl FVMAccount {
    pub fn new_contract(balance: U256, nonce: U256) -> Self {
        Self {
            balance: balance,
            nonce: nonce,
            storage_root: BLAKE2B_NULL_RLP,
            storage_cache: Self::empty_storage_cache(),
            storage_changes: Self::empty_storage_change(),
            code_hash: BLAKE2B_EMPTY,
            code_cache: Arc::new(vec![]),
            code_size: None,
            code_filth: Filth::Clean,
            address_hash: Cell::new(None),
            empty_but_commit: false,
        }
    }

    pub fn new_basic(balance: U256, nonce: U256) -> Self {
        Self {
            balance: balance,
            nonce: nonce,
            storage_root: BLAKE2B_NULL_RLP,
            storage_cache: Self::empty_storage_cache(),
            storage_changes: Self::empty_storage_change(),
            code_hash: BLAKE2B_EMPTY,
            code_cache: Arc::new(vec![]),
            code_size: Some(0),
            code_filth: Filth::Clean,
            address_hash: Cell::new(None),
            empty_but_commit: false,
        }
    }

    fn storage_is_clean(&self) -> bool {
        self.storage_changes.0.is_empty() && self.storage_changes.1.is_empty()
    }

    /// Commit the `storage_changes` to the backing DB and update `storage_root`.
    fn commit_storage(
        &mut self,
        trie_factory: &TrieFactory,
        db: &mut HashStore,
    ) -> trie::Result<()>
    {
        let mut t = trie_factory.from_existing(db, &mut self.storage_root)?;
        for (k, v) in self.storage_changes.0.drain() {
            // cast key and value to trait type,
            // so we can call overloaded `to_bytes` method
            match v.is_zero() {
                true => t.remove(&k)?,
                false => t.insert(&k, &encode(&U128::from(&*v)))?,
            };

            self.storage_cache.0.borrow_mut().insert(k, v);
        }

        for (k, v) in self.storage_changes.1.drain() {
            // cast key and value to trait type,
            // so we can call overloaded `to_bytes` method
            match v.is_zero() {
                true => t.remove(&k)?,
                false => t.insert(&k, &encode(&v))?,
            };

            self.storage_cache.1.borrow_mut().insert(k, v);
        }

        Ok(())
    }

    fn discard_storage_changes(&mut self) {
        self.storage_changes.0.clear();
        self.storage_changes.1.clear();
    }

    /// Clone basic account data
    fn clone_basic(&self) -> Self {
        Self {
            balance: self.balance.clone(),
            nonce: self.nonce.clone(),
            storage_root: self.storage_root.clone(),
            storage_cache: Self::empty_storage_cache(),
            storage_changes: Self::empty_storage_change(),
            code_hash: self.code_hash.clone(),
            code_size: self.code_size.clone(),
            code_cache: self.code_cache.clone(),
            code_filth: self.code_filth,
            address_hash: self.address_hash.clone(),
            empty_but_commit: self.empty_but_commit.clone(),
        }
    }

    /// Clone account data, dirty storage keys and cached storage keys.
    fn clone_all(&self) -> Self {
        let mut account = self.clone_dirty();
        account.storage_cache = self.storage_cache.clone();
        account
    }

    /// Replace self with the data from other account merging storage cache.
    /// Basic account data and all modifications are overwritten
    /// with new values.
    fn overwrite_with(&mut self, other: Self) {
        self.balance = other.balance;
        self.nonce = other.nonce;
        self.storage_root = other.storage_root;
        self.code_hash = other.code_hash;
        self.code_filth = other.code_filth;
        self.code_cache = other.code_cache;
        self.code_size = other.code_size;
        self.address_hash = other.address_hash;

        let mut cache = self.storage_cache.0.borrow_mut();
        for (k, v) in other.storage_cache.0.into_inner() {
            cache.insert(k.clone(), v.clone()); //TODO: cloning should not be required here
        }

        let mut cache = self.storage_cache.1.borrow_mut();
        for (k, v) in other.storage_cache.1.into_inner() {
            cache.insert(k.clone(), v.clone()); //TODO: cloning should not be required here
        }

        self.storage_changes = other.storage_changes;
    }
}

impl AVMAccount {
     /// Replace self with the data from other account merging storage cache.
    /// Basic account data and all modifications are overwritten
    /// with new values.
    fn overwrite_with(&mut self, other: Self) {
        self.balance = other.balance;
        self.nonce = other.nonce;
        self.storage_root = other.storage_root;
        self.code_hash = other.code_hash;
        self.code_filth = other.code_filth;
        self.code_cache = other.code_cache;
        self.code_size = other.code_size;
        self.address_hash = other.address_hash;

        let mut cache = self.storage_cache.borrow_mut();
        for (k, v) in other.storage_cache.into_inner() {
            cache.insert(k.clone(), v.clone()); //TODO: cloning should not be required here
        }
        self.storage_changes = other.storage_changes;
    }
}

impl From<BasicAccount> for AVMAccount {
    fn from(basic: BasicAccount) -> Self {
        Account {
            balance: basic.balance,
            nonce: basic.nonce,
            storage_root: basic.storage_root,
            storage_cache: Self::empty_storage_cache(),
            storage_changes: Self::empty_storage_change(),
            code_hash: basic.code_hash,
            code_size: None,
            code_cache: Arc::new(vec![]),
            code_filth: Filth::Clean,
            address_hash: Cell::new(None),
            empty_but_commit: false,
        }
    }
}

type AVMCache = RefCell<LruCache<Bytes, Bytes>>;
type AVMStorageChange = HashMap<Bytes, Bytes>;
pub type AVMAccount = Account<AVMCache, AVMStorageChange>;

impl AVMAccount {
    fn empty_storage_cache() -> AVMCache {
        RefCell::new(LruCache::new(STORAGE_CACHE_ITEMS))
    }

    fn empty_storage_change() -> AVMStorageChange {
        HashMap::new()
    }

    fn new_basic(balance: U256, nonce: U256) -> Self {
        Self {
            balance: balance,
            nonce: nonce,
            storage_root: BLAKE2B_NULL_RLP,
            storage_cache: Self::empty_storage_cache(),
            storage_changes: HashMap::new(),
            code_hash: BLAKE2B_EMPTY,
            code_cache: Arc::new(vec![]),
            code_size: Some(0),
            code_filth: Filth::Clean,
            address_hash: Cell::new(None),
            empty_but_commit: false,
        }
    }

    /// Determine whether there are any un-`commit()`-ed storage-setting operations.
    fn storage_is_clean(&self) -> bool {
        self.storage_changes.is_empty() 
    }

    /// Clone basic account data
    fn clone_basic(&self) -> Self {
        Self {
            balance: self.balance.clone(),
            nonce: self.nonce.clone(),
            storage_root: self.storage_root.clone(),
            storage_cache: Self::empty_storage_cache(),
            storage_changes: Self::empty_storage_change(),
            code_hash: self.code_hash.clone(),
            code_size: self.code_size.clone(),
            code_cache: self.code_cache.clone(),
            code_filth: self.code_filth,
            address_hash: self.address_hash.clone(),
            empty_but_commit: self.empty_but_commit,
        }
    }

    // commit avm storage changes to the Backing DB
    fn commit_storage(
        &mut self,
        trie_factory: &TrieFactory,
        db: &mut HashStore,
    ) -> trie::Result<()>
    {
        let mut t = trie_factory.from_existing(db, &mut self.storage_root)?;
        for (k, v) in self.storage_changes.drain() {
            // cast key and value to trait type,
            // so we can call overloaded `to_bytes` method
            let mut is_zero = true;
            for item in &v {
                if *item != 0x00_u8 {
                    is_zero = false;
                    break;
                }
            }
            match is_zero {
                true => t.remove(&k)?,
                false => t.insert(&k, &encode(&v))?,
            };

            self.storage_cache.borrow_mut().insert(k, v);
        }

        Ok(())
    }
}

macro_rules! impl_account {
    ($T: ty, $fixed_strg: expr) => {
        impl CommonAccount for $T {
            fn from_rlp(rlp: &[u8]) -> $T {
                let basic: BasicAccount = ::rlp::decode(rlp);
                basic.into()
            }

            fn init_code(&mut self, code: Bytes) {
                self.code_hash = blake2b(&code);
                self.code_cache = Arc::new(code);
                self.code_size = Some(self.code_cache.len());
                self.code_filth = Filth::Dirty;
            }

            fn reset_code(&mut self, code: Bytes) {
                self.init_code(code);
            }

            fn balance(&self) -> &U256 {&self.balance}

            fn nonce(&self) -> &U256 {&self.nonce}

            fn code_hash(&self) -> H256 {self.code_hash.clone()}

            fn address_hash(&self, address: &Address) -> H256 {
                let hash = self.address_hash.get();
                hash.unwrap_or_else(|| {
                    let hash = blake2b(address);
                    self.address_hash.set(Some(hash.clone()));
                    hash
                })
            }

            fn code(&self) -> Option<Arc<Bytes>> {
                if self.code_cache.is_empty() {
                    return None;
                }

                Some(self.code_cache.clone())
            }

            fn code_size(&self) -> Option<usize>{self.code_size.clone()}
            
            fn is_cached(&self) -> bool {
                !self.code_cache.is_empty()
                    || (self.code_cache.is_empty() && self.code_hash == BLAKE2B_EMPTY)
            }

            fn cache_code(&mut self, db: &HashStore) -> Option<Arc<Bytes>> {
                // TODO: fill out self.code_cache;
                trace!(
                    target: "account",
                    "Account::cache_code: ic={}; self.code_hash={:?}, self.code_cache={}",
                    self.is_cached(),
                    self.code_hash,
                    self.code_cache.pretty()
                );

                if self.is_cached() {
                    return Some(self.code_cache.clone());
                }

                match db.get(&self.code_hash) {
                    Some(x) => {
                        self.code_size = Some(x.len());
                        self.code_cache = Arc::new(x.into_vec());
                        Some(self.code_cache.clone())
                    }
                    _ => {
                        warn!(target: "account","Failed reverse get of {}", self.code_hash);
                        None
                    }
                }
            }

            fn cache_given_code(&mut self, code: Arc<Bytes>) {
                trace!(
                    target: "account",
                    "Account::cache_given_code: ic={}; self.code_hash={:?}, self.code_cache={}",
                    self.is_cached(),
                    self.code_hash,
                    self.code_cache.pretty()
                );

                self.code_size = Some(code.len());
                self.code_cache = code;
            }

            fn cache_code_size(&mut self, db: &HashStore) -> bool {
                // TODO: fill out self.code_cache;
                trace!(
                    target: "account",
                    "Account::cache_code_size: ic={}; self.code_hash={:?}, self.code_cache={}",
                    self.is_cached(),
                    self.code_hash,
                    self.code_cache.pretty()
                );
                self.code_size.is_some() || if self.code_hash != BLAKE2B_EMPTY {
                    match db.get(&self.code_hash) {
                        Some(x) => {
                            self.code_size = Some(x.len());
                            true
                        }
                        _ => {
                            warn!(target: "account","Failed reverse get of {}", self.code_hash);
                            false
                        }
                    }
                } else {
                    false
                }
            }

            fn is_empty(&self) -> bool {
                assert!(
                    self.storage_is_clean(),
                    "Account::is_empty() may only legally be called when storage is clean."
                );
                self.is_null() && self.storage_root == BLAKE2B_NULL_RLP
            }

            fn is_null(&self) -> bool {
                self.balance.is_zero() && self.nonce.is_zero() && self.code_hash == BLAKE2B_EMPTY
            }

            fn is_basic(&self) -> bool {
                self.code_hash == BLAKE2B_EMPTY
            }

            fn storage_root(&self) -> Option<&H256> {
                if self.storage_is_clean() {
                    Some(&self.storage_root)
                } else {
                    None
                }
            }

            fn inc_nonce(&mut self) {self.nonce = self.nonce + U256::from(1u8);}

            /// Increase account balance.
            fn add_balance(&mut self, x: &U256) {self.balance = self.balance + *x;}

            /// Decrease account balance.
            /// Panics if balance is less than `x`
            fn sub_balance(&mut self, x: &U256) {
                assert!(self.balance >= *x);
                self.balance = self.balance - *x;
            }

            /// Commit any unsaved code. `code_hash` will always return the hash of the `code_cache` after this.
            fn commit_code(&mut self, db: &mut HashStore) {
                trace!(
                    target: "account",
                    "Commiting code of {:?} - {:?}, {:?}",
                    self,
                    self.code_filth == Filth::Dirty,
                    self.code_cache.is_empty()
                );
                match (self.code_filth == Filth::Dirty, self.code_cache.is_empty()) {
                    (true, true) => {
                        self.code_size = Some(0);
                        self.code_filth = Filth::Clean;
                    }
                    (true, false) => {
                        db.emplace(
                            self.code_hash.clone(),
                            DBValue::from_slice(&*self.code_cache),
                        );
                        self.code_size = Some(self.code_cache.len());
                        self.code_filth = Filth::Clean;
                    }
                    (false, _) => {}
                }
            }

            /// Export to RLP.
            fn rlp(&self) -> Bytes {
                let mut stream = RlpStream::new_list(4);
                stream.append(&self.nonce);
                stream.append(&self.balance);
                stream.append(&self.storage_root);
                stream.append(&self.code_hash);
                stream.out()
            }

            /// Clone account data and dirty storage keys
            fn clone_dirty(&self) -> Self {
                let mut account = self.clone_basic();
                account.storage_changes = self.storage_changes.clone();
                account.code_cache = self.code_cache.clone();
                account
            }
        }
    };
}

impl_account!(FVMAccount, true);
impl_account!(AVMAccount, false);

#[derive(Debug)]
enum FVMKey {
    Normal(H128),
    Wide(H128),
}

#[derive(Debug, PartialEq)]
enum FVMValue {
    Normal(H128),
    Long(H256),
}

impl FVMAccount {
    fn storage_at(&self, db: &HashStore, key: &FVMKey) -> trie::Result<FVMValue> {
        if let Some(value) = self.cached_storage_at(key) {
            return Ok(value);
        }
        let db = SecTrieDB::new(db, &self.storage_root)?;

        match key {
            FVMKey::Normal(key) => {
                let item: U128 = db.get_with(key, ::rlp::decode)?.unwrap_or_else(U128::zero);
                let value: H128 = item.into();
                self.storage_cache
                    .0
                    .borrow_mut()
                    .insert(key.clone(), value.clone());
                Ok(FVMValue::Normal(value))
            },
            FVMKey::Wide(key) => {
                let item: U256 = db.get_with(key, ::rlp::decode)?.unwrap_or_else(U256::zero);
                let value: H256 = item.into();
                self.storage_cache
                    .1
                    .borrow_mut()
                    .insert(key.clone(), value.clone());
                Ok(FVMValue::Long(value))
            },
        }
    }

    fn cached_storage_at(&self, key: &FVMKey) -> Option<FVMValue> {
        match key {
            FVMKey::Normal(key) => {
                if let Some(value) = self.storage_changes.0.get(key) {
                    return Some(FVMValue::Normal(value.clone()));
                }
                if let Some(value) = self.storage_cache.0.borrow_mut().get_mut(key) {
                    return Some(FVMValue::Normal(value.clone()));
                }
                None
            },
            FVMKey::Wide(key) => {
                if let Some(value) = self.storage_changes.1.get(key) {
                    return Some(FVMValue::Long(value.clone()));
                }
                if let Some(value) = self.storage_cache.1.borrow_mut().get_mut(key) {
                    return Some(FVMValue::Long(value.clone()));
                }
                None
            },
        }
    }

    fn set_storage(&mut self, key: FVMKey, value: FVMValue) {
        match key {
            FVMKey::Normal(key) => {
                if let FVMValue::Normal(value) = value {
                    self.storage_changes.0.insert(key, value);
                } else {
                    panic!("unexpected key/value pair: maybe the value is too long");
                }
            },
            FVMKey::Wide(key) => {
                if let FVMValue::Long(value) = value {
                    self.storage_changes.1.insert(key, value);
                } else {
                    panic!("unexpected key/value pair: maybe the value is too short");
                }
            },
        }
    }
}

impl AVMAccount {
    fn storage_at(&self, db: &HashStore, key: &Bytes) -> trie::Result<Bytes> {
        println!("get storage: key = {:?}", key);
        if let Some(value) = self.cached_storage_at(key) {
            return Ok(value);
        }
        let db = SecTrieDB::new(db, &self.storage_root)?;

        let value: Vec<u8> = db.get_with(key, ::rlp::decode)?.unwrap_or_else(|| vec![]);
        self.storage_cache
            .borrow_mut()
            .insert(key.clone(), value.clone());
        println!("get storage value from db: key = {:?}, value = {:?}", key, value);
        Ok(value)
    }

    fn cached_storage_at(&self, key: &Bytes) -> Option<Bytes> {
        println!("search storage_changes: {:?}", self.storage_changes);
        if let Some(value) = self.storage_changes.get(key) {
            return Some(value.clone());
        }

        if let Some(value) = self.storage_cache.borrow_mut().get_mut(key) {
            return Some(value.clone());
        }
        None
    }

    fn set_storage(&mut self, key: Bytes, value: Bytes) {
        println!("pre storage_changes = {:?}", self.storage_changes);
        self.storage_changes.insert(key, value);
        let raw_changes: *mut HashMap<Vec<u8>, Vec<u8>> = unsafe {::std::mem::transmute(&self.storage_changes)};
        println!("storage_changes ptr = {:?}", raw_changes);
        println!("post storage_changes = {:?}", self.storage_changes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kvdb::MemoryDB;
    use account_db::*;

    #[test]
    fn storage_at() {
        let mut db = MemoryDB::new();
        let mut db = AccountDBMut::new(&mut db, &Address::new());
        let rlp = {
            let mut a = FVMAccount::new_contract(69.into(), 0.into());
            a.set_storage(FVMKey::Normal(H128::from(0x00u64)), FVMValue::Normal(H128::from(0x1234u64)));
            a.commit_storage(&Default::default(), &mut db).unwrap();
            a.init_code(vec![]);
            a.commit_code(&mut db);
            a.rlp()
        };

        let a = FVMAccount::from_rlp(&rlp);
        assert_eq!(
            *a.storage_root().unwrap(),
            "d2e59a50e7414e56da75917275d1542a13fd345bf88a657a4222a0d50ad58868".into()
        );
        let value = a.storage_at(&db.immutable(), &FVMKey::Normal(H128::from(0x00u64))).unwrap();
        assert_eq!(
            value,
            FVMValue::Normal(0x1234u64.into())
        );
        let value = a.storage_at(&db.immutable(), &FVMKey::Normal(0x01u64.into())).unwrap();
        assert_eq!(
            value,
            FVMValue::Normal(H128::default())
        );
    }
}

unsafe impl Sync for FVMAccount {}
unsafe impl Sync for AVMAccount {}