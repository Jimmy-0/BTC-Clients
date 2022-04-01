#![allow(unused)]
pub mod worker;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;

use std::thread;

use crate::types::hash::{H256, Hashable};
use crate::types::transaction::{Transaction, SignedTransaction, generate_random_transaction};
use crate::blockchain::{State, Blockchain};
use crate::types::address::Address;
use crate::types::key_pair;
use ring::signature::{KeyPair, Ed25519KeyPair};
use rand::Rng;
use std::sync::{Arc, Mutex};

enum ControlSignal {
    Start(u64), // the number controls the theta of interval between transaction generation
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
    generated_trx_chan: Sender<SignedTransaction>,
	blockchain: Arc<Mutex<Blockchain>>,
	controlled_keys: Vec<Ed25519KeyPair>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the generator thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(blockchain: &Arc<Mutex<Blockchain>>, keys: Vec<Ed25519KeyPair>) -> (Context, Handle, Receiver<SignedTransaction>) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (generated_trx_sender, generated_trx_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        generated_trx_chan: generated_trx_sender,
		blockchain: Arc::clone(blockchain),
		controlled_keys: keys,
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, generated_trx_receiver)
}

#[cfg(any(test,test_utilities))]
fn test_new() -> (Context, Handle, Receiver<SignedTransaction>) {
    let blockchain = Blockchain::new();
    let blockchain = Arc::new(Mutex::new(blockchain));
	let mut keys: Vec<Ed25519KeyPair> = vec![];
	let key = key_pair::from_seed(0);
	keys.push(key);
    new(&blockchain, keys)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, theta: u64) {
        self.control_chan
            .send(ControlSignal::Start(theta))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("generator".to_string())
            .spawn(move || {
                self.generator_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn generator_loop(&mut self) {
		// main generation loop

		let mut trx: Transaction;
		let mut key: &Ed25519KeyPair;
		let mut signed_trx: SignedTransaction;
		let mut num_keys: usize;
		let mut start_idx: usize;
		let mut i: usize;
		let mut recv_addr: Address;
		let mut send_addr: Address;
		let mut tuple: (u32, u32);
		let mut val: u32;
		let mut init_new_key: bool;

		loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Generator shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Generator starting in continuous mode with theta {}", i);
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
                                info!("Generator shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Generator starting in continuous mode with theta {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                //unimplemented!()
								continue;
							}
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Generator control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // TODO for student: actual generation, create a SignedTransaction
			//trx = generate_random_transaction();
			//key = key_pair::random();
			//signed_trx = SignedTransaction::new(trx, &key);

			// fetch state of tip of blockchain
			let blockchain = self.blockchain.lock().unwrap();
			let state: State = blockchain.get_state(blockchain.tip()).unwrap();
			drop(blockchain);


			num_keys = self.controlled_keys.len();
			start_idx = rand::thread_rng().gen_range(0..num_keys);
			i = 0;
			init_new_key = false;
			let mut new_key: Ed25519KeyPair = key_pair::random();
			while i<num_keys {
				key = &self.controlled_keys[ (start_idx+i)%num_keys ];
				send_addr = Address::from_public_key_bytes(key.public_key().as_ref());
				let (nonce, bal) = match state.get_balance(send_addr) {
					Ok(tuple) => tuple,
					Err(e) => {i=i+1; continue;}, // when a new key is added in a previously generated trx but that trx is not yet mined, the state does not contain its corresponding address
				};
				if bal > 0 {
					recv_addr = *state.get_random_addr().unwrap();
					if recv_addr == send_addr { // to introduce new key with a small probability
						new_key = key_pair::random();
						recv_addr = Address::from_public_key_bytes(new_key.public_key().as_ref());
						//self.controlled_keys.push(new_key);
						init_new_key = true;
					}
					val = rand::thread_rng().gen_range(1..bal);
					trx = Transaction{
						receiver: recv_addr,
						value: val,
						account_nonce: nonce+1
					};
					signed_trx = SignedTransaction::new(trx, key);
					//println!("generated strx, hash:{}, send:{},bal:{},recv:{},nonce:{},val:{}", signed_trx.hash(), Address::from_public_key_bytes(key.public_key().as_ref()), bal, recv_addr, nonce+1, val);

					if init_new_key {
						self.controlled_keys.push(new_key);
					}
					// TODO for student: if transaction generation is finished, you can have something like: self.generated_trx_chan.send(block.clone()).expect("Send finished block error");
					self.generated_trx_chan.send(signed_trx.clone()).expect("Send finished signed_trx error");

					break;
				}
				i=i+1;
			}

			//if num_keys>0 && i==num_keys {
			//	println!("all senders are zero-balance.");
			//}

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros((i as f64 * 0.5 * 1e4) as u64);
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
	use crate::types::transaction::SignedTransaction;

    #[test]
    #[timeout(60000)]
    fn generate_three_transactions() {
        let (generator_ctx, generator_handle, generated_trx_chan) = super::test_new();
        generator_ctx.start();
        generator_handle.start(0);
        let mut trx: SignedTransaction;
        for i in 0..2 {
			println!("transaction id: {}", i);
			trx = generated_trx_chan.recv().unwrap();
			println!("generated trx: {:#?}", trx);
			assert!(trx.verify())
        }
		//assert!(false);
    }

    #[test]
    #[timeout(60000)]
    fn generate_hundred_transactions() {
        let (generator_ctx, generator_handle, generated_trx_chan) = super::test_new();
        generator_ctx.start();
        generator_handle.start(0);
        let mut trx: SignedTransaction;
        for _ in 0..99 {
			trx = generated_trx_chan.recv().unwrap();
			assert!(trx.verify())
        }
		//assert!(false);
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
