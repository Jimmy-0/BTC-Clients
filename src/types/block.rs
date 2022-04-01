use serde::{Serialize, Deserialize};
use crate::types::hash::{H256, Hashable};
use crate::types::transaction::{Transaction, SignedTransaction};
use rand::Rng;
use ring::signature::KeyPair;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::types::key_pair;
use crate::types::merkle::MerkleTree;
use crate::types::address::Address;

//extern crate chrono;
//use chrono:: prelude::*;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Header{
    pub parent_hash: H256, 	// hash ptr to parent block
    pub nonce: u32,			// random integer for proof of work check
    pub difficulty: H256,	// threshold for proof of work
    pub timestamp: u128, 	// timestamp when the block is generated
    pub merkle_root: H256, 	// merkle_root of signed trx in content
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Content {
    pub data: Vec<SignedTransaction>, 	// transactions
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Block {
    pub header: Header,
    pub content: Content,
}

impl Hashable for Header{
    fn hash(&self) -> H256{
        let encodehead: Vec <u8> = bincode:: serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &encodehead[..]).into()
    }
}


impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Block {
    pub fn get_parent(&self) -> H256 {
        let parent_hash = self.header.parent_hash.clone();
        return parent_hash;
    } 

    pub fn get_difficulty(&self) -> H256 {
		return self.header.difficulty;
    }

	/// Get the hashes of all transactions in this block
	pub fn get_transaction_hashes(&self) -> Vec<H256> {
		let mut hashes: Vec<H256> = vec![];
		for trx in &self.content.data {
			hashes.push(trx.hash());
		}

		return hashes;
	}
}

//#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
	let mut rng = rand::thread_rng();

	let mut nonce: u32;
	let mut ts: u128;
	let mut head: Header;
	let mut block: Block;
	let root: H256 = hex!("4b3947f87e40c184f6394d4f0916a43b1395d51855e39b4ffe400b2be3797d98").into();
	//let difficulty = hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into();
	let difficulty = hex!("0000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into();

	let trx: Vec<SignedTransaction> = vec![];
	let content: Content = Content{data: trx};

	loop {
		nonce = rng.gen();
		ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
		head = Header{
			parent_hash: *parent,
			nonce: nonce,
			difficulty: difficulty,
			timestamp: ts,
			merkle_root: root
		};
		block = Block{header: head.clone(), content: content.clone()};

		if block.hash() <= difficulty { break; }
	}

	return block;
}

pub fn generate_genesis_block() -> Block {
	let parent: H256 = hex!("0000000000000000000000000000000000000000000000000000000000000000").into();
	let nonceval: u32 = 0;
	let ts: u128 = 0;

	let difficulty = hex!("0000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into();

	let mut vect: Vec<SignedTransaction> = vec![];
	// Initial Coin Offering (ICO) transactions
	let key = key_pair::from_seed(0);
	let trx = Transaction {
		receiver: Address::from_public_key_bytes(key.public_key().as_ref()),
		value: 1e6 as u32,
		account_nonce: 0
	};
	let signed_trx = SignedTransaction::new(trx, &key);
	vect.push(signed_trx);

	//let root: H256 = hex!("4b3947f87e40c184f6394d4f0916a43b1395d51855e39b4ffe400b2be3797d98").into();
	let merkle_tree: MerkleTree = MerkleTree::new(&vect);
	let root: H256 = merkle_tree.root();

	let head: Header = Header{
		parent_hash: parent,
		nonce: nonceval,
		difficulty: difficulty,
		timestamp: ts,
		merkle_root: root
	};
	let content: Content = Content{data: vect};
	let block: Block = Block{header: head, content: content};
	return block;
}
