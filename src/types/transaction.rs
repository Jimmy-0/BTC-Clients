#![allow(unused)]
use serde::{Serialize,Deserialize};
use ring::signature::{self, Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use rand::{Rng, distributions::Alphanumeric};
use crate::types::address::Address;
use crate::types::hash::{H256, Hashable};
use crate::types::key_pair;
use std::collections::VecDeque;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]

pub struct TransactionMempool{
	tx_hash_queue: VecDeque<H256>,
	tx_map: HashMap<H256, SignedTransaction>,
}
  
impl TransactionMempool{
	/// Initialise a new mempool
	pub fn new() -> Self{
		TransactionMempool{
			tx_hash_queue: VecDeque::new(), 
			tx_map: HashMap::new()
		}
	}

	/// Insert a transaction in mempool
	pub fn insert(&mut self, trx: &SignedTransaction, push_in_queue: bool){
		if trx.verify(){
			let hash = trx.hash();
			if push_in_queue { // in case the transaction needs to be mined in the block
				self.tx_hash_queue.push_back(hash);
			}
			self.tx_map.insert(hash, trx.clone());
		} else {
			println!("recieved invalid transaction: {:#?}", trx);
		}
	}

	/// Dequeue a transaction from mempool queue
	pub fn dequeue(&mut self) -> Result<SignedTransaction, &'static str> {
		if self.tx_hash_queue.is_empty() {
			return Err("empty queue in mempool");
		} else {
			let hash = self.tx_hash_queue.pop_front().unwrap();
			let trx = &self.tx_map[ &hash ];
			return Ok(trx.clone());
		}
	}

    /// Check if a transaction hash is present
    pub fn is_hash_present(&self, hash: H256) -> bool {
        return self.tx_map.contains_key(&hash)
    }

    /// Retrieve a transaction corresponding to a hash
    pub fn get_transaction(&self, hash: H256) -> Result<SignedTransaction, &'static str> {
        if !self.tx_map.contains_key(&hash) {
            return Err("invalid transaction hash");
        } else {
            let trx = &self.tx_map[ &hash ];
            return Ok(trx.clone());
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
	pub receiver: Address,
	pub value: u32,
	pub account_nonce: u32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
	transaction: Transaction,
	sign: Vec<u8>,
	public_key: Vec<u8>,
}

impl SignedTransaction {
	pub fn new(trx: Transaction, key: &Ed25519KeyPair) -> Self {
		let signature: Signature = sign(&trx, &key);
		let signed_trx = SignedTransaction{
			transaction: trx,
			sign: signature.as_ref().to_vec(),
			public_key: key.public_key().as_ref().to_vec()
		};
		return signed_trx;
	}

	pub fn verify(&self) -> bool {
		return verify(&self.transaction, &self.public_key, &self.sign);
	}

	pub fn sender_address(&self) -> Address {
		return Address::from_public_key_bytes(self.public_key.as_ref());
	}

	pub fn receiver_address(&self) -> Address {
		return self.transaction.receiver;
	}

	pub fn value(&self) -> u32 {
		return self.transaction.value;
	}

	pub fn account_nonce(&self) -> u32 {
		return self.transaction.account_nonce;
	}
}

impl Hashable for SignedTransaction{
    fn hash(&self) -> H256{
        let encoded: Vec <u8> = bincode:: serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &encoded[..]).into()
    }
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
	let serialized = serde_json::to_string(&t).unwrap();
	//println!("serialized: {}", serialized);

	return key.sign(serialized.as_bytes());
	//return Signature::new();
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
	let serialized = serde_json::to_string(&t).unwrap();
	let unparsed_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, &public_key[..]);

	let result = unparsed_public_key.verify( serialized.as_bytes(), &signature[..]).is_ok();
	//println!("serialized: {}", serialized);
	//println!("sign: {:?}", signature);
	//println!("result: {}", result);

	return result;
	//return false;
}

pub fn generate_random_string(len: usize) -> String {
	let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect();
	
	return s;
}

//pub fn generate_random_transaction() -> Transaction {
//    let input = vec![UtxoInput{tx_hash: H256::generate_random_hash(), idx: 0}];
//    let output = vec![UtxoOutput{receipient_addr: Address::generate_random_address(), value: 0}];
//    
//    Transaction{sender: input, receiver: output}
//}


//#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
	let r = generate_random_string(30);
	let rb = r.as_bytes();

	let mut rng = rand::thread_rng();
	let val: u32 = rng.gen();
	let ac_nonce: u32 = rng.gen(); // maintain last account_nonce for each pk, set ac_nonce = last_ac_nonce + 1

	//let s = hex!("00");
	//let r = hex!("01");
	let t = Transaction{
		receiver: Address::from_public_key_bytes(&rb),
		value: val,
		account_nonce: ac_nonce
	};

	return t;
}

//#[cfg(any(test, test_utilities))]
//pub fn generate_random_signed_transaction(key: &Ed25519KeyPair) -> SignedTransaction {
//	let trx: Transaction = generate_random_transaction();
//	let signature: Signature = sign(&trx, &key);
//	let signed_trx = SignedTransaction{
//		transaction: trx,
//		sign: signature.as_ref().to_vec(),
//		public_key: key.public_key().as_ref().to_vec()
//	};
//
//	return signed_trx;
//}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;


    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
	}
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
    }
	#[test]
	fn sign_verify_signed_trx() {
		let trx = generate_random_transaction();
		let key = key_pair::random();
		let signed_trx = SignedTransaction::new(trx, &key);
		assert!(signed_trx.verify());
	}
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
