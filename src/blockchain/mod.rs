#![allow(unused)]
use serde::{Serialize, Deserialize};
use crate::types::block::{Block, generate_genesis_block};
use crate::types::hash::{H256, Hashable};
use std::collections::HashMap;
use crate::types::transaction::{Transaction, SignedTransaction, generate_random_transaction};
use crate::types::address::Address;
use crate::types::key_pair;
use rand::Rng;
use ring::signature::{KeyPair, Ed25519KeyPair};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct State{
	ledger: HashMap<Address, (u32, u32)>, // Address corresponding to public_key -> (account_nonce, balance)
}

impl State {
    /// Create a new state containing balances defined as per Initial Coin Offering
    pub fn new(ico: Vec<(Address, (u32, u32))>) -> Self {
		let mut ledger: HashMap< Address, (u32, u32)> = HashMap::new();
		for entry in ico.iter() {
			let (addr, tuple) = entry;
			ledger.insert(*addr, *tuple);
		}

		let state = Self {
			ledger: ledger,
		};

		return state;
	}

	/// Return an updated state after applying a vector of transactions to it
	pub fn update(&mut self, signed_trx: &Vec<SignedTransaction>) -> Result<State, &'static str> {
		let mut state: State = self.clone();
		let mut tuple: (u32, u32);
		let mut val: u32;
		let mut sender_addr: Address;
		let mut receiver_addr: Address;

		for strx in signed_trx.iter() {
			sender_addr = strx.sender_address();
			if !state.ledger.contains_key(&sender_addr) {
				return Err("Invalid Transaction: sender does not exist");
			}
			tuple = state.ledger[&sender_addr];
			let (nonce, mut bal) = tuple;
			val = strx.value();
			if bal<val {
				return Err("Invalid Transaction: insufficient balance at sender");
			} else if nonce+1!=strx.account_nonce() {
				return Err("Invalid Transaction: invalid account nonce");
			}
			state.ledger.insert(sender_addr, (nonce+1, bal-val));

			receiver_addr = strx.receiver_address();
			if !state.ledger.contains_key(&receiver_addr) {
				state.ledger.insert(receiver_addr, (0, val));
			} else {
				tuple = state.ledger[ &receiver_addr ];
				let (nonce, bal) = tuple;
				state.ledger.insert(receiver_addr, (nonce, bal+val));
			}
		}

		return Ok(state);
	}

	/// Returns a random address from the ledger
	pub fn get_random_addr(&self) -> Option<&Address> {
		let len = self.ledger.len();
		if len==0 {
			return None;
		} else {
			let index = rand::thread_rng().gen_range(0..len);
			return self.ledger.keys().skip(index).next();
		}
	}

	/// Fetch the details corresponding to an address
	pub fn get_balance(&self, addr: Address) -> Result<(u32, u32), &'static str> {
		if self.ledger.contains_key(&addr) {
			return Ok(self.ledger[ &addr ]);
		} else {
			return Err("Address does not exist in state");
		}
	}

	/// Return the account details as a vector of tuples
	pub fn get_account_details(&self) -> Vec<(String, String, String)> {
		let mut accounts: Vec<(String, String, String)> = vec![];
		let mut account_tuple;
		for (key,value) in &self.ledger {
			if value.1>0 {
				account_tuple = (key.to_string(), value.0.to_string(), value.1.to_string());
				accounts.push(account_tuple);
			}
		}
		return accounts;
	}
}


// TODO - convert serialisation function to bincode
pub struct Blockchain {
	hashmap: HashMap< H256, (Block, u32)>, 	// storage of blocks, HashMap: Hash -> (Block, height)
	longest_chain_len: u32, 				// length of longest chain
	tip: H256, 								// hash of last block in longest chain
	block_state_map: HashMap<H256, State>,	// storage of states, HashMap: Hash -> State
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
		//let parent: H256 = hex!("0000000000000000000000000000000000000000000000000000000000000000").into();
		//let genesis: Block = generate_random_block(&parent);
		let genesis: Block = generate_genesis_block();
		let hash: H256 = genesis.hash();

		let mut hashmap = HashMap::new();
		hashmap.insert(hash, (genesis, 0));

		let mut ico: Vec<(Address, (u32, u32))> = vec![];
		let mut key: Ed25519KeyPair;
		let mut bal: u32;
		let mut addr: Address;

		key = key_pair::from_seed(0);
		bal = 1e6 as u32;
		addr = Address::from_public_key_bytes(key.public_key().as_ref());
		//println!("ico public key: {:#?}", key.public_key().as_ref());
		//println!("ico addr: {}", addr);
		ico.push((addr, (0, bal)));

		key = key_pair::from_seed(1);
		bal = 0;
		addr = Address::from_public_key_bytes(key.public_key().as_ref());
		ico.push((addr, (0, bal)));
		key = key_pair::from_seed(2);
		bal = 0;
		addr = Address::from_public_key_bytes(key.public_key().as_ref());
		ico.push((addr, (0, bal)));

		let state = State::new(ico);
		let mut block_state_map = HashMap::new();
		block_state_map.insert(hash, state);

		let chain = Self {
			hashmap: hashmap,
			longest_chain_len: 0,
			tip: hash,
			block_state_map: block_state_map,
		};
		return chain;
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
		let hash: H256 = block.hash();
		let parent_hash: H256 = block.get_parent();
		if hash > block.get_difficulty() {
			println!("received invalid block, ignoring it. block: {:#?}", block);
		} else if !self.hashmap.contains_key(&parent_hash) {
			println!("received orphan block");
			// add block to orphan buffer
		} else { // block is valid
			let parent_tuple = &self.hashmap[ &parent_hash ];
			let (_parent, parent_height) = parent_tuple;
			let height = parent_height + 1;
			self.hashmap.insert(hash, (block.clone(), height));
			if height > self.longest_chain_len {
				self.longest_chain_len = height;
				self.tip = hash;
			}
			// Process and save the state of current block
			let mut parent_state: State = self.block_state_map[ &parent_hash ].clone();
			let state: State = parent_state.update(&block.content.data).unwrap();
			self.block_state_map.insert(hash, state);
		}
	}

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
		return self.tip;
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
		let mut longest_chain: Vec<H256> = Vec::new();
		let mut hash: H256 = self.tip;
		let (mut block, mut height) = self.hashmap[ &hash ].clone();
		let mut tuple: &(Block, u32);
		while height>0 {
			longest_chain.push(hash);
			hash = block.get_parent();
			tuple = &self.hashmap[ &hash ];
			block = tuple.0.clone();
			height = tuple.1;
		}
		longest_chain.push(hash);

		longest_chain.reverse();
		return longest_chain;
    }

	/// Check if a block hash is present
	pub fn is_hash_present(&self, hash: H256) -> bool {
		return self.hashmap.contains_key(&hash)
	}

	/// Retrieve a block corresponding to a hash
	pub fn get_block(&self, hash: H256) -> Result<Block, &'static str> {
		if !self.hashmap.contains_key(&hash) {
			return Err("invalid block hash");
		} else {
			let tuple = &self.hashmap[ &hash ];
			let (block, _height) = tuple;
			return Ok(block.clone());
		}
	}

	/// Retrieve the state corresponding to a block hash
	pub fn get_state(&self, hash: H256) -> Result<State, &'static str> {
		if !self.block_state_map.contains_key(&hash) {
			return Err("invalid block hash");
		} else {
			let state: State = self.block_state_map[ &hash ].clone();
			return Ok(state.clone());
		}
	}

    /// Get all transactions' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_transactions_in_longest_chain(&self) -> Vec<Vec<H256>> {
		let mut longest_chain: Vec<Vec<H256>> = Vec::new();
		let mut hash: H256 = self.tip;
		let (mut block, mut height) = self.hashmap[ &hash ].clone();
		let mut tuple: &(Block, u32);
		while height>0 {
			longest_chain.push(block.get_transaction_hashes());
			hash = block.get_parent();
			tuple = &self.hashmap[ &hash ];
			block = tuple.0.clone();
			height = tuple.1;
		}
		longest_chain.push(block.get_transaction_hashes());

		longest_chain.reverse();
		return longest_chain;
    }

    /// Count the number of transactions in the longest chain
    pub fn count_transactions_in_longest_chain(&self) -> usize {
		let mut longest_chain_count: usize = 0;
		let mut hash: H256 = self.tip;
		let (mut block, mut height) = self.hashmap[ &hash ].clone();
		let mut tuple: &(Block, u32);
		while height>0 {
			longest_chain_count += block.content.data.len();
			hash = block.get_parent();
			tuple = &self.hashmap[ &hash ];
			block = tuple.0.clone();
			height = tuple.1;
		}

		return longest_chain_count;
    }

	/// Returns vector of accounts
	/// TODO convert Address, nonce, balance into string
	pub fn get_block_state(&self, id: u32) -> Vec<(String, String, String)> {
		let mut hash: H256 = self.tip;
		let (mut block, mut height) = self.hashmap[ &hash ].clone();
		let mut accounts: Vec<(String, String, String)> = vec![];
		let mut tuple: &(Block, u32);

		if height<id {
			return accounts;
		}
		while height>id{
			hash = block.get_parent();
			tuple = &self.hashmap[ &hash ];
			block = tuple.0.clone();
			height = tuple.1;
		}

		let state: State = self.block_state_map[ &hash ].clone();
		let mut accounts: Vec<(String, String, String)> = state.get_account_details();
		accounts.sort();
		return accounts;
	}
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());
    }
	#[test]
	fn forked_chains_equal_length() {
		/*
				G
			  / |
			 6	1
				| \
				2  4
				|  |
				3  5

		Note: block index represents the order of addition of the blocks in the blockchain
		*/
		let mut blockchain = Blockchain::new();
		let genesis_hash = blockchain.tip();
		let b1 = generate_random_block(&genesis_hash);
		blockchain.insert(&b1);
		let b2 = generate_random_block(&b1.hash());
		blockchain.insert(&b2);
		let b3 = generate_random_block(&b2.hash());
		blockchain.insert(&b3);
		let b4 = generate_random_block(&b1.hash());
		blockchain.insert(&b4);
		let b5 = generate_random_block(&b4.hash());
		blockchain.insert(&b5);
		let b6 = generate_random_block(&genesis_hash);
		blockchain.insert(&b6);

		assert_eq!(blockchain.tip(), b3.hash());
    }
	#[test]
	fn forked_chain_new_longest_chain() {
		/*
				G
			  / |
			 6	1
				| \
				2  4
				|  |
				3  5
				   |
				   7

		note: block index represents the order of addition of the blocks in the blockchain
		*/
		let mut blockchain = Blockchain::new();
		let genesis_hash = blockchain.tip();
		let b1 = generate_random_block(&genesis_hash);
		blockchain.insert(&b1);
		let b2 = generate_random_block(&b1.hash());
		blockchain.insert(&b2);
		let b3 = generate_random_block(&b2.hash());
		blockchain.insert(&b3);
		let b4 = generate_random_block(&b1.hash());
		blockchain.insert(&b4);
		let b5 = generate_random_block(&b4.hash());
		blockchain.insert(&b5);
		let b6 = generate_random_block(&genesis_hash);
		blockchain.insert(&b6);
		let b7 = generate_random_block(&b5.hash());
		blockchain.insert(&b7);

		assert_eq!(blockchain.tip(), b7.hash());
    }
	#[test]
	fn forked_chain_get_longest_chain() {
		/*
				G
			  / |
			 6	1
				| \
				2  4
				|  |
				3  5
				   |
				   7

		note: block index represents the order of addition of the blocks in the blockchain
		*/
		let mut blockchain = Blockchain::new();
		let genesis_hash = blockchain.tip();
		let b1 = generate_random_block(&genesis_hash);
		blockchain.insert(&b1);
		let b2 = generate_random_block(&b1.hash());
		blockchain.insert(&b2);
		let b3 = generate_random_block(&b2.hash());
		blockchain.insert(&b3);
		let b4 = generate_random_block(&b1.hash());
		blockchain.insert(&b4);
		let b5 = generate_random_block(&b4.hash());
		blockchain.insert(&b5);
		let b6 = generate_random_block(&genesis_hash);
		blockchain.insert(&b6);
		let b7 = generate_random_block(&b5.hash());
		blockchain.insert(&b7);

		let longest_chain: Vec<H256> = blockchain.all_blocks_in_longest_chain();

		assert_eq!(longest_chain, vec![
				genesis_hash,
				b1.hash(),
				b4.hash(),
				b5.hash(),
				b7.hash()
			]
		);
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
