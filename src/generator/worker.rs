#![allow(unused)]
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use crate::network::server::Handle as ServerHandle;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::network::message::Message;
use crate::types::hash::Hashable;
use crate::types::transaction::{SignedTransaction, TransactionMempool};

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    generated_trx_chan: Receiver<SignedTransaction>,
	mempool: Arc<Mutex<TransactionMempool>>,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        generated_trx_chan: Receiver<SignedTransaction>,
		mempool: &Arc<Mutex<TransactionMempool>>,
    ) -> Self {
        Self {
            server: server.clone(),
            generated_trx_chan,
			mempool: Arc::clone(mempool),
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("generator-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Generator initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let _signed_trx = self.generated_trx_chan.recv().expect("Receive finished signed_trx error");
            // TODO for student: insert this finished signed_trx to mempool, and broadcast this signed_trx hash
			let mut mempool = self.mempool.lock().unwrap();
			mempool.insert(&_signed_trx, true);
			// broadcast the signed_trx hash
			self.server.broadcast(Message::NewTransactionHashes(vec![_signed_trx.hash()]));
        }
    }
}
