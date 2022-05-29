use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::Map;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Balance};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    pub balance: Balance,
    pub allowances: Map<Vec<u8>, Balance>,
}

impl Account {
    pub fn new(account_hash: Vec<u8>) -> Self {
        Self {
            balance: 0,
            allowances: Map::new(account_hash),
        }
    }

    pub fn set_allowance(&mut self, escrow_account_id: &AccountId, allowance: Balance) {
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        if allowance > 0 {
            self.allowances.insert(&escrow_hash, &allowance);
        } else {
            self.allowances.remove(&escrow_hash);
        }
    }

    pub fn get_allowance(&self, escrow_account_id: &AccountId) -> Balance {
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        self.allowances.get(&escrow_hash).unwrap_or(0)
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    pub accounts: Map<Vec<u8>, Account>,

    pub total_supply: Balance,
}

impl Default for FungibleToken {
    fn default() -> Self {
        panic!("Fun token should be initialized before usage")
    }
}

#[near_bindgen]
impl FungibleToken {
    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128) -> Self {
        let total_supply = total_supply.into();
        assert!(env::state_read::<Self>().is_none(), "Already initialized");
        let mut ft = Self {
            accounts: Map::new(b"a".to_vec()),
            total_supply,
        };
        let mut account = ft.get_account(&owner_id);
        account.balance = total_supply;
        ft.set_account(&owner_id, &account);
        ft
    }

    pub fn set_allowance(&mut self, escrow_account_id: AccountId, allowance: U128) {
        let allowance = allowance.into();
        let owner_id = env::predecessor_account_id();
        if escrow_account_id == owner_id {
            env::panic(b"Can't set allowance for yourself");
        }
        let mut account = self.get_account(&owner_id);

        account.set_allowance(&escrow_account_id, allowance);
        self.set_account(&owner_id, &account);
    }

    pub fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, amount: U128) {
        let amount = amount.into();
        if amount == 0 {
            env::panic(b"Can't transfer 0 tokens");
        }
        let mut account = self.get_account(&owner_id);

        if account.balance < amount {
            env::panic(b"Not enough balance");
        }
        account.balance -= amount;

        let escrow_account_id = env::predecessor_account_id();
        if escrow_account_id != owner_id {
            let allowance = account.get_allowance(&escrow_account_id);
            if allowance < amount {
                env::panic(b"Not enough allowance");
            }
            account.set_allowance(&escrow_account_id, allowance - amount);
        }

        self.set_account(&owner_id, &account);

        let mut new_account = self.get_account(&new_owner_id);
        new_account.balance += amount;
        self.set_account(&new_owner_id, &new_account);
    }

    pub fn transfer(&mut self, new_owner_id: AccountId, amount: U128) {
        self.transfer_from(env::predecessor_account_id(), new_owner_id, amount);
    }

    pub fn get_total_supply(&self) -> U128 {
        self.total_supply.into()
    }

    pub fn get_balance(&self, owner_id: AccountId) -> U128 {
        self.get_account(&owner_id).balance.into()
    }

    pub fn get_allowance(&self, owner_id: AccountId, escrow_account_id: AccountId) -> U128 {
        self.get_account(&owner_id)
            .get_allowance(&escrow_account_id)
            .into()
    }
}

impl FungibleToken {
    fn get_account(&self, owner_id: &AccountId) -> Account {
        let account_hash = env::sha256(owner_id.as_bytes());
        self.accounts
            .get(&account_hash)
            .unwrap_or_else(|| Account::new(account_hash))
    }

    fn set_account(&mut self, owner_id: &AccountId, account: &Account) {
        let account_hash = env::sha256(owner_id.as_bytes());
        self.accounts.insert(&account_hash, &account);
    }
}