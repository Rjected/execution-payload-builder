use clap::Parser;
use reth::primitives::{
    Header as PrimitiveHeader, SealedBlock, SealedHeader, TransactionSigned,
    Withdrawal as PrimitiveWithdrawal,
};
use reth::rpc::{
    compat::engine::payload::try_block_to_payload,
    types::{Block, BlockTransactions, Header, Withdrawal},
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
    let header = rpc_header_to_primitive_header(block.header).seal_slow();

    // TODO: blob versioned hashes from txs

    // convert withdrawals into primitive withdrawals
    let withdrawals: Option<Vec<PrimitiveWithdrawal>> = block.withdrawals.map(|withdrawals| {
        withdrawals
            .into_iter()
            .map(rpc_withdrawal_to_primitive_withdrawal)
            .collect()
    });

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

/// Converts a rpc header into primitive header
// TODO: upstream into rpc compat
fn rpc_header_to_primitive_header(header: Header) -> PrimitiveHeader {
    PrimitiveHeader {
        parent_hash: header.parent_hash,
        timestamp: header.timestamp.to(),
        ommers_hash: header.uncles_hash,
        beneficiary: header.miner,
        state_root: header.state_root,
        receipts_root: header.receipts_root,
        transactions_root: header.transactions_root,
        base_fee_per_gas: header.base_fee_per_gas.map(|x| x.to()),
        logs_bloom: header.logs_bloom,
        withdrawals_root: header.withdrawals_root,
        difficulty: header.difficulty.to(),
        number: header.number.unwrap().to(),
        gas_used: header.gas_used.to(),
        gas_limit: header.gas_limit.to(),
        mix_hash: header.mix_hash,
        nonce: header.nonce.unwrap().into(),
        extra_data: header.extra_data,
        blob_gas_used: header.blob_gas_used.map(|x| x.to()),
        excess_blob_gas: header.excess_blob_gas.map(|x| x.to()),
        parent_beacon_block_root: header.parent_beacon_block_root,
    }
}

// convert a rpc withdrawal into a primitive withdrawal
fn rpc_withdrawal_to_primitive_withdrawal(withdrawal: Withdrawal) -> PrimitiveWithdrawal {
    PrimitiveWithdrawal {
        index: withdrawal.index,
        amount: withdrawal.amount,
        validator_index: withdrawal.validator_index,
        address: withdrawal.address,
    }
}
