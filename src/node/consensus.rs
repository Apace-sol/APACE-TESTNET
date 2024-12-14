use solana_sdk::{
    hash::Hash,
    signature::Signature,
};
use std::time::{Duration, Instant};

pub struct ConsensusManager {
    last_block_hash: Hash,
    validators: Vec<Validator>,
    consensus_timeout: Duration,
    last_consensus: Instant,
}

impl ConsensusManager {
    pub fn new(timeout: Duration) -> Self {
        ConsensusManager {
            last_block_hash: Hash::default(),
            validators: Vec::new(),
            consensus_timeout: timeout,
            last_consensus: Instant::now(),
        }
    }

    pub async fn validate_transaction(&self, transaction: &Transaction) -> bool {
       
        if !self.verify_signature(transaction) {
            return false;
        }

        
        if !self.verify_timestamp(transaction) {
            return false;
        }

       
        let confirmations = self.get_validator_confirmations(transaction).await;
        
        
        confirmations > (self.validators.len() * 2 / 3)
    }

    fn verify_signature(&self, transaction: &Transaction) -> bool {
        transaction.verify_signature()
    }

    fn verify_timestamp(&self, transaction: &Transaction) -> bool {
       
        let now = Instant::now();
        let transaction_age = now.duration_since(transaction.timestamp);
        transaction_age < self.consensus_timeout
    }

    async fn get_validator_confirmations(&self, transaction: &Transaction) -> usize {
        let mut confirmations = 0;
        for validator in &self.validators {
            if validator.verify_transaction(transaction).await {
                confirmations += 1;
            }
        }
        confirmations
    }
}
