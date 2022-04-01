#![allow(unused)]
use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::blockchain::{Blockchain, State};
use crate::types::block::Block;
use crate::types::hash::{H256, Hashable};
use crate::types::transaction::{SignedTransaction, TransactionMempool};

use log::{debug, warn, error};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};

use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::{HashSet, HashMap};

#[cfg(any(test,test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test,test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    tx_mempool: Arc<Mutex<TransactionMempool>>,
}


impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
		tx_mempool: &Arc<Mutex<TransactionMempool>>
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(blockchain),
            tx_mempool: Arc::clone(tx_mempool),
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            let mut locked_blockchain = self.blockchain.lock().unwrap();
            let mut locked_mempool = self.tx_mempool.lock().unwrap();
			let mut buffer: HashMap<H256, Block> = HashMap::new();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }

                Message::NewBlockHashes(hashes) => {
					// hashes: Vec<H256>
                    let mut missing_hashes:Vec<H256> = vec![];
                    debug!("Received New Block Hash");

                    for hash in hashes{
						if !locked_blockchain.is_hash_present(hash){
							debug!("Block hash {} does not exist", hash);
							missing_hashes.push(hash);
						}
                    }
                    if missing_hashes.len()!=0{
                        debug!("Sending getBlocks ...");
                        peer.write(Message::GetBlocks(missing_hashes));
                    }
				}

                Message::GetBlocks(hashes) => {
					// hashes: Vec<H256>
                    let mut available_blocks:Vec<Block> = vec![];
					debug!("Receive GetBlocks");
                    for hash in hashes {
						if locked_blockchain.is_hash_present(hash) {
							debug!("Adding blockhash {} to available_blocks", hash);
							match locked_blockchain.get_block(hash) {
								Ok(block) => available_blocks.push(block),
								Err(e) => debug!("error getting block: {:?}", e),
							}
						} else if buffer.contains_key(&hash) {
							available_blocks.push(buffer[&hash].clone());
						}
                    }
                    if available_blocks.len() !=0{
                        debug!("Sending Block Message");
                        peer.write(Message::Blocks(available_blocks));
                    }
                }

                Message::Blocks(vec_blocks)=>{
                    debug!("Received Blocks Message");
					let mut get_block_hash: Vec<H256> = vec![];
					let mut new_block_hashes: Vec<H256> = vec![];
					let (mut hash, mut difficulty, mut parent_hash): (H256, H256, H256);
					let mut is_parent_present: bool;
					let mut parent_state: State;
					for block in vec_blocks{
                        let mut block_is_valid: bool = true;
                        for tx in &block.content.data{ // Transaction signature check
                            if !tx.verify(){
                                block_is_valid = false;
                                debug!("Invalid transaction in the block!");
                                break;
                            }
                        }
                        if block_is_valid{
                            hash = block.hash();
                            difficulty = block.get_difficulty();
                            if !(hash<=difficulty) || locked_blockchain.is_hash_present(hash) {
                                debug!("received block is either invalid or already present");
                                continue;
                            }
                            parent_hash = block.get_parent();
                            is_parent_present = locked_blockchain.is_hash_present(parent_hash);
                            if is_parent_present && difficulty==locked_blockchain.get_block(parent_hash).unwrap().get_difficulty() {
								parent_state = locked_blockchain.get_state(parent_hash).unwrap();
								match parent_state.update(&block.content.data) {
									Ok(state) => {locked_blockchain.insert(&block); new_block_hashes.push(hash)},
									Err(e) => continue,
								}
                                //locked_blockchain.insert(&block);
                                //new_block_hashes.push(hash); // Send Message::NewBlockHashes to broadcast new block
                            } else if !is_parent_present && !buffer.contains_key(&hash) {
                                get_block_hash.push(parent_hash); // Send Message::GetBlocks for parent of orphan block
                                buffer.insert(hash, block.clone());
                                new_block_hashes.push(hash); // Send Message::NewBlockHashes to broadcast new block
                            }
                            for signed_tx in &block.content.data{
                                let signed_tx_hash = signed_tx.hash();
								if !locked_mempool.is_hash_present(signed_tx_hash) {
									locked_mempool.insert(signed_tx, false);
								}
                            }
                        }
                    }
					if get_block_hash.len()>0 {
						// deduplicate the vector of parent hashes
						let set: HashSet<H256> = get_block_hash.drain(..).collect();
						get_block_hash.extend(set.into_iter());
						debug!("sending getblocks msg from Blocks msg handler to get {} hashes", get_block_hash.len());
						self.server.broadcast(Message::GetBlocks(get_block_hash));
						//peer.write(Message::GetBlocks(get_block_hash));
					}
					if new_block_hashes.len()>0 {
						self.server.broadcast(Message::NewBlockHashes(new_block_hashes));
						//peer.write(Message::NewBlockHashes(new_block_hashes));
					}
					let mut inserted_hashes: Vec<H256> = vec![];
					let mut something_inserted: bool;
					while buffer.len()>0 {
						something_inserted = false;
						for (hash, block) in buffer.iter() {
							parent_hash = block.get_parent();
							if locked_blockchain.is_hash_present(parent_hash) && block.get_difficulty()==locked_blockchain.get_block(parent_hash).unwrap().get_difficulty() {
								locked_blockchain.insert(&block);
								inserted_hashes.push(*hash);
								something_inserted = true;
							}
						}
						if !something_inserted {
							break
						} else {
							for hash in inserted_hashes.iter() {
								buffer.remove(hash);
							}
						}
					}
				}
                /*
                If a block's parent is missing, put this block into a buffer and send Getblocks message. The buffer stores the blocks whose parent is not seen yet. When the parent is received, that block can be popped out from buffer and inserted into blockchain.
                */

                //_ => unimplemented!(),
				Message::NewTransactionHashes(tx_hashes) => {
					let mut required_txs: Vec<H256> = vec![];
                    debug!("Recieved a new transaction hash");

                    for hash in tx_hashes{
						if locked_mempool.is_hash_present(hash) {
							debug!(" tx hash {} is already in mempool", hash)
                        } else {
							required_txs.push(hash.clone());
						}
                    }

                    if required_txs.len() != 0{
                        debug!("Sending getTransactions...");
                        peer.write(Message::GetTransactions(required_txs));
                    }
				}

				Message::GetTransactions(tx_hashes) => {
					let mut send_trx:Vec<SignedTransaction> = vec![];
					debug!("Received GetTransactions");

                    for hash in tx_hashes{
						if locked_mempool.is_hash_present(hash) {
							let signed_trx = locked_mempool.get_transaction(hash).unwrap();
							send_trx.push(signed_trx);
						} else {
							debug!(" tx hash {} is not in mempool", hash);
						}
                    }

                    if send_trx.len() != 0{
                        debug!("Sending Transactions msg ...");
                        peer.write(Message::Transactions(send_trx));
                    }
				}

				Message::Transactions(vec_signed_transaction) => {
					debug!("Recieved Transactions");
                    let mut tx_to_broadcast:Vec<H256> = vec![];
                    for signed_trx in vec_signed_transaction{
                        if signed_trx.verify(){
                            let hash = signed_trx.hash();
							if locked_mempool.is_hash_present(hash) {
								debug!("tx {} already in the memepool", hash);
							} else {
								locked_mempool.insert(&signed_trx, true);
								tx_to_broadcast.push(hash);
							}
                        }
                    }
                    if tx_to_broadcast.len() != 0{
                        self.server.broadcast(Message::NewTransactionHashes(tx_to_broadcast));
                    }
				}
            }
        }
    }
}

#[cfg(any(test,test_utilities))]
struct TestMsgSender {
    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>
}
#[cfg(any(test,test_utilities))]
impl TestMsgSender {
    fn new() -> (TestMsgSender, smol::channel::Receiver<(Vec<u8>, peer::Handle)>) {
        let (s,r) = smol::channel::unbounded();
        (TestMsgSender {s}, r)
    }

    fn send(&self, msg: Message) -> PeerTestReceiver {
        let bytes = bincode::serialize(&msg).unwrap();
        let (handle, r) = peer::Handle::test_handle();
        smol::block_on(self.s.send((bytes, handle))).unwrap();
        r
    }
}
#[cfg(any(test,test_utilities))]
/// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
    let (server, server_receiver) = ServerHandle::new_for_test();
    let (test_msg_sender, msg_chan) = TestMsgSender::new();
    let blockchain = Blockchain::new();
    let longest_chain_hashes: Vec<H256> = blockchain.all_blocks_in_longest_chain();
    let blockchain = Arc::new(Mutex::new(blockchain));
	let tx_mempool = TransactionMempool::new();
	let tx_mempool = Arc::new(Mutex::new(tx_mempool));
	let worker = Worker::new(1, msg_chan, &server, &blockchain, &tx_mempool);
    worker.start(); 
    (test_msg_sender, server_receiver, longest_chain_hashes)
}


// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    use super::super::message::Message;
    use super::generate_test_worker_and_start;

    #[test]
    #[timeout(60000)]
    fn reply_new_block_hashes() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut peer_receiver = test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
        let reply = peer_receiver.recv();
        if let Message::GetBlocks(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_get_blocks() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let h = v.last().unwrap().clone();
        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
        let reply = peer_receiver.recv();
        if let Message::Blocks(v) = reply {
            assert_eq!(1, v.len());
            assert_eq!(h, v[0].hash())
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
	fn reply_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
		let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
		let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
