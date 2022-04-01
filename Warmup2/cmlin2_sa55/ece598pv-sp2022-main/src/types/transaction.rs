use serde::{Serialize,Deserialize};
use ring::signature::{self, Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use rand::{Rng, distributions::Alphanumeric};
use crate::types::address::Address;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
	sender: Address,
	receiver: Address,
	value: u32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
	transaction: Transaction,
	sign: Vec<u8>,
	public_key: Vec<u8>,
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    //unimplemented!()
	let serialized = serde_json::to_string(&t).unwrap();
	//println!("serialized: {}", serialized);

	return key.sign(serialized.as_bytes());
	//return Signature::new();
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    //unimplemented!()

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

#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    //unimplemented!()

	let s = generate_random_string(30);
	let sb = s.as_bytes();
	let r = generate_random_string(30);
	let rb = r.as_bytes();

	let mut rng = rand::thread_rng();
	let val: u32 = rng.gen();

	//let s = hex!("00");
	//let r = hex!("01");
	let t = Transaction{
		sender: Address::from_public_key_bytes(&sb),
		receiver: Address::from_public_key_bytes(&rb),
		value: val
	};

	return t;
}

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
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
