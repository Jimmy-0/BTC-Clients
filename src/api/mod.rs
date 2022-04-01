use serde::Serialize;
use crate::blockchain::Blockchain;
use crate::miner::Handle as MinerHandle;
use crate::generator::Handle as GeneratorHandle;
use crate::network::server::Handle as NetworkServerHandle;
use crate::network::message::Message;

use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::Header;
use tiny_http::Response;
use tiny_http::Server as HTTPServer;
use url::Url;

pub struct Server {
    handle: HTTPServer,
    miner: MinerHandle,
	generator: GeneratorHandle,
    network: NetworkServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

macro_rules! respond_result {
    ( $req:expr, $success:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let payload = ApiResponse {
            success: $success,
            message: $message.to_string(),
        };
        let resp = Response::from_string(serde_json::to_string_pretty(&payload).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}
macro_rules! respond_json {
    ( $req:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let resp = Response::from_string(serde_json::to_string(&$message).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}

impl Server {
    pub fn start(
        addr: std::net::SocketAddr,
        miner: &MinerHandle,
        generator: &GeneratorHandle,
        network: &NetworkServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
    ) {
        let handle = HTTPServer::http(&addr).unwrap();
        let server = Self {
            handle,
            miner: miner.clone(),
            generator: generator.clone(),
            network: network.clone(),
            blockchain: Arc::clone(blockchain),
        };
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                let miner = server.miner.clone();
                let generator = server.generator.clone();
                let network = server.network.clone();
                let blockchain = Arc::clone(&server.blockchain);
                thread::spawn(move || {
                    // a valid url requires a base
                    let base_url = Url::parse(&format!("http://{}/", &addr)).unwrap();
                    let url = match base_url.join(req.url()) {
                        Ok(u) => u,
                        Err(e) => {
                            respond_result!(req, false, format!("error parsing url: {}", e));
                            return;
                        }
                    };
                    match url.path() {
                        "/miner/start" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let lambda = match params.get("lambda") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing lambda");
                                    return;
                                }
                            };
                            let lambda = match lambda.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing lambda: {}", e)
                                    );
                                    return;
                                }
                            };
                            miner.start(lambda);
                            respond_result!(req, true, "ok");
                        }
                        "/tx-generator/start" => {
                            // respond_result!(req, false, "unimplemented!");
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let theta = match params.get("theta") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing theta");
                                    return;
                                }
                            };
                            let theta = match theta.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing theta: {}", e)
                                    );
                                    return;
                                }
                            };
                            generator.start(theta);
                            respond_result!(req, true, "ok");
                        }
                        "/network/ping" => {
                            network.broadcast(Message::Ping(String::from("Test ping")));
                            respond_result!(req, true, "ok");
                        }
                        "/blockchain/longest-chain" => {
                            let blockchain = blockchain.lock().unwrap();
                            let v = blockchain.all_blocks_in_longest_chain();
                            let v_string: Vec<String> = v.into_iter().map(|h|h.to_string()).collect();
                            respond_json!(req, v_string);
                        }
                        "/blockchain/longest-chain-tx" => {
                            //respond_result!(req, false, "unimplemented!");
                            let blockchain = blockchain.lock().unwrap();
                            let v = blockchain.all_transactions_in_longest_chain();
                            let mut v_string: Vec<Vec<String>> = Vec::new();

                            for trxs_vector in v.iter() {
                                let mut tmp: Vec<String> = Vec::new();
                                for trxs in trxs_vector.iter() {
                                    tmp.push(trxs.to_string());
                                }
                                v_string.push(tmp.clone());
                            }
                            respond_json!(req, v_string);
                        }
                        "/blockchain/longest-chain-tx-count" => {
                            //respond_result!(req, false, "unimplemented!");
                            let blockchain = blockchain.lock().unwrap();
                            let v = blockchain.count_transactions_in_longest_chain();
                            let v_string = v.to_string();
                            respond_json!(req, v_string);
						}
                        "/blockchain/state" =>{
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let block_id = match params.get("block") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing block");
                                    return;
                                }
                            };
                            let block_id = match block_id.parse::<u32>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing block: {}", e)
                                    );
                                    return;
                                }
                            };
                            let blockchain = blockchain.lock().unwrap();
                            let accounts: Vec<(String, String, String)> = blockchain.get_block_state(block_id);

							let v_string: Vec<String> = accounts.into_iter().map(|h| format!("{},{},{}", h.0, h.1, h.2)).collect();
                            respond_json!(req, v_string);
                        }
                        _ => {
                            let content_type =
                                "Content-Type: application/json".parse::<Header>().unwrap();
                            let payload = ApiResponse {
                                success: false,
                                message: "endpoint not found".to_string(),
                            };
                            let resp = Response::from_string(
                                serde_json::to_string_pretty(&payload).unwrap(),
                            )
                            .with_header(content_type)
                            .with_status_code(404);
                            req.respond(resp).unwrap();
                        }
                    }
                });
            }
        });
        info!("API server listening at {}", &addr);
    }
}
