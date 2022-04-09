use astreuos_transaction::Transaction;
use astro_format::{decode, encode};
use fides::{merkle_root};
use opis::Int;
use std::collections::HashMap;
use std::error::Error;

#[derive(Clone, Debug)]
pub struct Account {
    pub balance: Int,
    pub counter: Int,
    pub storage: HashMap<[u8; 32], [u8; 32]>
}

impl Account {

    pub fn from_bytes(arg: &Vec<u8>) -> Result<Self, Box<dyn Error>> {

        let set = decode(arg);

        if set.len() == 3 {

            let storage: HashMap<[u8; 32], [u8; 32]> = {
            
                let key_val_set = decode(&set[2]);

                if key_val_set.is_empty() {

                    let keys_vals: Vec<Option<([u8; 32], [u8; 32])>> = key_val_set
                        .iter()
                        .map(|x| {

                            let key_val = decode(x);
                            
                            if key_val.len() == 2 {
                                Some(([0_u8; 32], [0_u8; 32]))
                            } else {
                                None
                            }
                            
                        })
                        .collect::<Vec<_>>();

                    if keys_vals.iter().any(|x| x.is_none()) {
                        Err("Key value error!")?
                    } else {

                        keys_vals
                            .iter()
                            .map(|x| x.clone().unwrap())
                            .collect::<HashMap<_, _>>()

                    }

                } else {
                    HashMap::new()
                }
                
            };

            Ok(Account{
                balance: Int::from_bytes(&set[0]),
                counter: Int::from_bytes(&set[1]),
                storage: storage
            })

        } else {
           Err("Account error!")? 
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        merkle_root(&vec![
            self.balance.to_bytes(),
            self.counter.to_bytes(),
            self.storage_hash().to_vec()
        ])
    }

    pub fn new() -> Self {
        Account {
            balance: Int::zero(),
            counter: Int::zero(),
            storage: HashMap::new()
        }
    }

    pub fn storage_hash(&self) -> [u8; 32] {

        let mut records = self.storage
            .iter()
            .map(|x| (x.0, x.1))
            .collect::<Vec<_>>();

        records.sort_by_key(|k| k.0);

        merkle_root(
            &records
                .iter()
                .map(|x| merkle_root(&vec![x.0.to_vec(), x.1.to_vec()]).to_vec())
                .collect::<Vec<_>>()
        )

    }

    pub fn to_bytes(&self) -> Vec<u8> {
        encode(&vec![
            self.balance.to_bytes(),
            self.counter.to_bytes(),
            encode(&self.storage
                .iter()
                .map(|x| {
                    encode(&vec![
                        x.0.to_vec(),
                        x.1.to_vec()
                    ])
                })
                .collect()
            )
        ])
    }

}

#[derive(Clone, Debug)]
pub struct Accounts {
    pub accounts: HashMap<[u8; 32], Account>
}

impl Accounts {

    pub fn apply_transaction(
        &self,
        transaction: &Transaction,
        solar_price: &Int
    ) -> Option<(HashMap<[u8; 32], Account>, Receipt)> {

        if &transaction.solar_price >= solar_price {

            let transaction_cost = Int::from_decimal("1000");

            if transaction.solar_limit >= transaction_cost {

                match self.accounts.get(&transaction.sender) {

                    Some(s) => {
    
                        if s.counter == transaction.counter {
    
                            let mut sender = s.clone();
    
                            let mut solar_used = Int::zero();
    
                            let transaction_fee =  &transaction_cost * solar_price;
    
                            sender.balance -= transaction_fee;
    
                            solar_used += transaction_cost;

                            if transaction.sender == transaction.recipient {

                                None

                            } else {
    
                                match self.accounts.get(&transaction.recipient) {
        
                                    Some(r) => {
        
                                        let mut recipient = r.clone();
        
                                        if sender.balance >= transaction.value {
        
                                            sender.balance -= transaction.value.clone();
        
                                            recipient.balance += transaction.value.clone();
        
                                            let receipt = Receipt { solar_used: solar_used, status: Status::Accepted };

                                            let changed = HashMap::from([
                                                (transaction.sender, sender),
                                                (transaction.recipient, recipient)
                                            ]);
        
                                            Some((changed, receipt))
                                            
                                        } else {
        
                                            let receipt = Receipt {
                                                solar_used: solar_used,
                                                status: Status::BalanceError
                                            };

                                            let changed = HashMap::from([(transaction.sender, sender)]);
        
                                            Some((changed, receipt))
        
                                        }
                                    },
        
                                    None => {
        
                                        let remaining_transaction_solar = &transaction.solar_limit - &solar_used;
        
                                        let account_creation_cost = Int::from_decimal("200000");
                                        
                                        if remaining_transaction_solar >= account_creation_cost {
        
                                            let account_creation_fee: Int = &account_creation_cost * solar_price;
                                            
                                            if sender.balance >= account_creation_fee {
        
                                                solar_used += account_creation_cost;
        
                                                sender.balance -= account_creation_fee;
                                            
                                                if sender.balance >= transaction.value {
                                                    
                                                    sender.balance -= transaction.value.clone();
        
                                                    let mut recipient = Account::new();

                                                    recipient.balance = transaction.value.clone();
        
                                                    let receipt = Receipt {
                                                        solar_used: solar_used,
                                                        status: Status::Accepted
                                                    };

                                                    let changed = HashMap::from([
                                                        (transaction.sender, sender),
                                                        (transaction.recipient, recipient)
                                                    ]);
        
                                                    Some((changed, receipt))
        
                                                } else {
                                                    
                                                    let receipt = Receipt {
                                                        solar_used: solar_used,
                                                        status: Status::BalanceError
                                                    };
        
                                                    let changed = HashMap::from([(transaction.sender, sender)]);
        
                                                    Some((changed, receipt))
        
                                                }
        
                                            } else {
        
                                                let receipt = Receipt {
                                                    solar_used: solar_used,
                                                    status: Status::BalanceError
                                                };
        
                                                let changed = HashMap::from([(transaction.sender, sender)]);
                                                
                                                Some((changed, receipt))
        
                                            }
        
                                        }  else {
        
                                            let receipt = Receipt {
                                                solar_used: solar_used,
                                                status: Status::SolarError
                                            };
        
                                            let changed = HashMap::from([(transaction.sender, sender)]);
        
                                            Some((changed, receipt))
        
                                        } 
                                    }
                                }
                            }
                        } else {
                            None
                        }
                    },
                    None => None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn hash(&self) -> [u8; 32] {

        let mut account_hashes = self.accounts
            .iter()
            .map(|x| (x.0, x.1.hash()))
            .collect::<Vec<_>>();

        account_hashes.sort_by_key(|k| k.0);

        merkle_root(
            &account_hashes
                .iter()
                .map(|x| merkle_root(&vec![x.0.to_vec(), x.1.to_vec()]).to_vec())
                .collect::<Vec<_>>()
        )
        
    }

    pub fn new() -> Self {
        Accounts {
            accounts: HashMap::new()
        }
    }

}

#[derive(Clone, Debug)]
pub struct Receipt {
    pub solar_used: Int,
    pub status: Status
}

impl Receipt {
    pub fn hash(&self) -> [u8; 32] {
        merkle_root(&vec![self.solar_used.to_bytes(), self.status.to_bytes()])
    }
}

#[derive(Clone, Debug)]
pub enum Status {
    Accepted,
    BalanceError,
    SolarError
}

impl Status {

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Status::Accepted => vec![1_u8],
            Status::BalanceError => vec![2_u8],
            Status::SolarError => vec![3_u8]
        }
    }

}
