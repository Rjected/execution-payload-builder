use clap::Parser;
use reth::primitives::{SealedBlock, SealedHeader, TransactionSigned, Withdrawal};
use reth::rpc::{
    compat::engine::payload::try_block_to_payload,
    types::{Block, BlockTransactions},
};

/// Parses the given json file, creating an execution payload from it.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the json file to parse
    #[arg(short, long)]
    path: String,
}

fn main() {
    let args = Args::parse();

    // parse the file
    let file = std::fs::read_to_string(args.path).unwrap();
    let block: Block = serde_json::from_str(&file).unwrap();

    // print the block
    println!("Extracted block: {:#?}", block);

    // extract the parent beacon block root
    let parent_beacon_block_root = block.header.parent_beacon_block_root;

    // convert transactions into primitive txs
    // TODO: upstream into rpc compat
    let txs = match block.transactions {
        // this would be an error in upstream
        BlockTransactions::Hashes(hashes) => {
            panic!("send the eth_getBlockByHash request with full: `true`")
        }
        BlockTransactions::Full(txs) => txs,
        // this would be an error in upstream
        BlockTransactions::Uncle => panic!("this should not be run on uncle blocks"),
    };

    // convert transactions into primitive transactions
    let body: Vec<TransactionSigned> = todo!("convert txs into primitive txs");

    // convert header into a primitive header
    // TODO: ustream into rpc compat
    let header: SealedHeader = todo!("convert header into a primitive header");

    // TODO: blob versioned hashes from txs

    // convert withdrawals into primitive withdrawals
    let withdrawals: Option<Vec<Withdrawal>> =
        todo!("convert withdrawals into primitive withdrawals");

    // convert into an execution payload
    // TODO: this will fail if transactions are not full.
    // TODO: upstream into rpc compat
    let sealed = SealedBlock {
        header,
        ommers: Vec::new(),
        body,
        withdrawals,
    };

    // convert to execution payload
    let execution_payload = try_block_to_payload(sealed);

    // convert into something that can be sent to the engine, ie `cast rpc` or something
    // this needs to be combined with the parent beacon block root, and blob versioned hashes
}
