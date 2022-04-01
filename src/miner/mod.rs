pub mod worker;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;

use std::thread;

use crate::types::block::{Block, Header, Content};
use crate::types::hash::{H256, Hashable};
use crate::blockchain::{State, Blockchain};
use crate::types::transaction::{SignedTransaction, TransactionMempool};
use crate::types::merkle::MerkleTree;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_TRX_PER_BLOCK: u32 = 25;

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Update, // update the block in mining, it may due to new blockchain tip or new transaction
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    finished_block_chan: Sender<Block>,
	blockchain: Arc<Mutex<Blockchain>>,
	mempool: Arc<Mutex<TransactionMempool>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(blockchain: &Arc<Mutex<Blockchain>>, mempool: &Arc<Mutex<TransactionMempool>>) -> (Context, Handle, Receiver<Block>) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_block_sender, finished_block_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_block_chan: finished_block_sender,
		blockchain: Arc::clone(blockchain),
		mempool: Arc::clone(mempool),
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_block_receiver)
}

#[cfg(any(test,test_utilities))]
fn test_new() -> (Context, Handle, Receiver<Block>) {
    let blockchain = Blockchain::new();
    let blockchain = Arc::new(Mutex::new(blockchain));
	let mempool = TransactionMempool::new();
	let mempool = Arc::new(Mutex::new(mempool));
    new(&blockchain, &mempool)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn miner_loop(&mut self) {
        // main mining loop
		let mut parent_hash: H256;
		let mut difficulty: H256 = [255u8; 32].into();
		let mut timestamp: u128;
		let mut rng = rand::thread_rng();
		let mut nonce: u32;
		let mut state: State;

		let blockchain = self.blockchain.lock().unwrap();
		parent_hash = blockchain.tip();
		state = blockchain.get_state(parent_hash).unwrap();
		//difficulty = blockchain.get_block(parent_hash).unwrap().get_difficulty();
		match blockchain.get_block(parent_hash) {
			Ok(b) => difficulty = b.get_difficulty(),
			Err(e) => println!("error getting block: {:?}", e),
		}
		drop(blockchain); // to release mutex lock

		loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Miner shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Miner starting in continuous mode with lambda {}", i);
                            self.operating_state = OperatingState::Run(i);
                        }
                        ControlSignal::Update => {
                            // in paused state, don't need to update
                        }
                    };
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        match signal {
                            ControlSignal::Exit => {
                                info!("Miner shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Miner starting in continuous mode with lambda {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                //unimplemented!()
								let blockchain = self.blockchain.lock().unwrap();
								parent_hash = blockchain.tip();
								state = blockchain.get_state(parent_hash).unwrap();
								match blockchain.get_block(parent_hash) {
									Ok(b) => difficulty = b.get_difficulty(),
									Err(e) => println!("error getting block: {:?}", e),
								}
							}
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // TODO for student: actual mining, create a block
			let mut data: Vec<SignedTransaction> = vec![];
			let mut i: u32 = 0;
			let mut mempool = self.mempool.lock().unwrap(); // to acquire mutex lock
			loop {
				match mempool.dequeue() {
					Ok(trx) => {
						match state.update(&vec![trx.clone()]) {
							Ok(s) => {state = s; data.push(trx)},
							Err(_e) => {
								//println!("received strx, hash:{},error:{}", trx.hash(), _e);
								break;},
						};
					},
					Err(_) => break,
				}

				i = i + 1;
				if i >= MAX_TRX_PER_BLOCK { break; }
			}
			drop(mempool); // to release mutex lock

			if data.len()>0 { // mine a block only if there is atleast 1 trx
				//let root: H256 = hex!("4b3947f87e40c184f6394d4f0916a43b1395d51855e39b4ffe400b2be3797d98").into();
				let merkle_tree: MerkleTree = MerkleTree::new(&data);
				let root: H256 = merkle_tree.root();
				let content: Content = Content{data: data};
				let mut head: Header;
				let mut block: Block;

				loop {
					timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
					nonce = rng.gen();
					head = Header{
						parent_hash: parent_hash,
						nonce: nonce,
						difficulty: difficulty,
						timestamp: timestamp,
						merkle_root: root
					};
					block = Block{header: head.clone(), content: content.clone()};

					if block.hash() <= difficulty { break; }
				}
				//println!("parent_hash: {}, block_hash: {}", parent_hash, block.hash());

				// TODO for student: if block mining finished, you can have something like: self.finished_block_chan.send(block.clone()).expect("Send finished block error");
				self.finished_block_chan.send(block.clone()).expect("Send finished block error");
				parent_hash = block.hash();
			}

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::hash::Hashable;

    #[test]
    #[timeout(60000)]
    fn miner_three_block() {
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        for _ in 0..2 {
            let block_next = finished_block_chan.recv().unwrap();
            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }

	#[test]
	#[timeout(60000)]
    fn miner_twenty_block() {
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        for _ in 0..19 {
			println!("iteration start");
            let block_next = finished_block_chan.recv().unwrap();
			println!("{:#?}", block_next);
            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
