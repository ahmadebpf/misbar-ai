use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::domain::ReceiptRecord;

#[derive(Debug, Clone)]
pub struct Store {
    receipts: Arc<RwLock<HashMap<Uuid, ReceiptRecord>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            receipts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn save_receipt(&self, receipt: ReceiptRecord) {
        let receipt_id = receipt.receipt_id;
        self.receipts.write().unwrap().insert(receipt_id, receipt);
    }

    pub fn get_receipt(&self, receipt_id: &Uuid) -> Option<ReceiptRecord> {
        self.receipts.read().unwrap().get(receipt_id).cloned()
    }
}
