use clap::Parser;
use reth::rpc::{
    compat::engine::payload::try_block_to_payload,
    types::{Block, BlockTransactions, ExecutionPayload, Header, Transaction, Withdrawal},
};
use reth::{
    primitives::{
        transaction::{TxEip1559, TxEip2930, TxEip4844, TxLegacy},
        AccessList, AccessListItem, Header as PrimitiveHeader, SealedBlock, Signature,
        Transaction as PrimitiveTransaction, TransactionKind, TransactionSigned,
        Withdrawal as PrimitiveWithdrawal, U64,
    },
    rpc::types::Parity,
};

/// Parses the given json file, creating an execution payload from it.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the json file to parse
    #[arg(short, long)]
    path: String,

    /// The engine rpc url to use
    #[arg(short, long)]
    rpc_url: Option<String>,

    /// The jwt secret to use
    #[arg(short, long)]
    jwt_secret: Option<String>,
}

fn main() {
    let args = Args::parse();

    // parse the file
    let file = std::fs::read_to_string(args.path).unwrap();
    let block: Block = serde_json::from_str(&file).unwrap();

    // extract the parent beacon block root
    let parent_beacon_block_root = block.header.parent_beacon_block_root;

    // convert transactions into primitive txs
    // TODO: upstream into rpc compat
    let txs = match block.transactions {
        // this would be an error in upstream
        BlockTransactions::Hashes(_hashes) => {
            panic!("send the eth_getBlockByHash request with full: `true`")
        }
        BlockTransactions::Full(txs) => txs,
        // this would be an error in upstream
        BlockTransactions::Uncle => panic!("this should not be run on uncle blocks"),
    };

    // convert transactions into primitive transactions
    let body: Vec<TransactionSigned> = txs
        .into_iter()
        .map(rpc_transaction_to_primitive_transaction)
        .collect();

    // convert header into a primitive header
    let header = rpc_header_to_primitive_header(block.header).seal_slow();

    // extract blob versioned hashes from txs
    let mut blob_versioned_hashes = Vec::new();
    for tx in &body {
        if let PrimitiveTransaction::Eip4844(tx) = &tx.transaction {
            blob_versioned_hashes.extend(tx.blob_versioned_hashes.clone());
        }
    }

    // convert withdrawals into primitive withdrawals
    let withdrawals: Option<Vec<PrimitiveWithdrawal>> = block.withdrawals.map(|withdrawals| {
        withdrawals
            .into_iter()
            .map(rpc_withdrawal_to_primitive_withdrawal)
            .collect()
    });

    // convert into an execution payload
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
    let json_payload = match execution_payload {
        ExecutionPayload::V1(payload) => serde_json::to_string(&payload).unwrap(),
        ExecutionPayload::V2(payload) => serde_json::to_string(&payload).unwrap(),
        ExecutionPayload::V3(payload) => serde_json::to_string(&payload).unwrap(),
    };

    // print blob versioned hashes and parent beacon block root
    // let json_versioned_hashes = serde_json::to_string(&blob_versioned_hashes.into_iter().map(|versioned_hash| format!("{versioned_hash}")).collect::<Vec<String>>()).unwrap();
    let json_versioned_hashes = serde_json::to_string(&blob_versioned_hashes).unwrap();
    let json_parent_beacon_block_root = serde_json::to_string(&parent_beacon_block_root).unwrap();

    // craft the request to pass into `cast rpc --raw`
    let json_request = "'[".to_string()
        + &[
            json_payload,
            json_versioned_hashes,
            json_parent_beacon_block_root,
        ]
        .join(",")
        + "]'";

    // construct the cast rpc command
    let mut prefix = "cast rpc".to_string();
    let suffix = "engine_newPayloadV3 --raw ".to_string() + &json_request;

    if let Some(rpc_url) = args.rpc_url {
        prefix += &format!(" --rpc-url {}", rpc_url);
    }

    if let Some(secret) = args.jwt_secret {
        prefix += &format!(" --jwt-secret {}", secret);
    }

    // add the suffix and request
    prefix += &format!(" {}", suffix);

    // print the payload
    println!("{prefix}");
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

// convert a rpc transaction to a primitive transaction
fn rpc_transaction_to_primitive_transaction(transaction: Transaction) -> TransactionSigned {
    let nonce = transaction.nonce.to();
    let to = match transaction.to {
        Some(addr) => TransactionKind::Call(addr),
        None => TransactionKind::Create,
    };
    let value = transaction.value.into();
    let chain_id = transaction.chain_id.unwrap().to();
    let input = transaction.input;
    let access_list = AccessList(
        transaction
            .access_list
            .unwrap_or_default()
            .into_iter()
            .map(|item| AccessListItem {
                address: item.address,
                storage_keys: item.storage_keys,
            })
            .collect(),
    );
    let gas_limit = transaction.gas.to();

    // this is definitely a signed tx
    let rpc_signature = transaction.signature.unwrap();

    // massive chain ids can be ignored here
    let v: u64 = rpc_signature.v.to();

    // if y parity is defined use that
    // TODO: ugh eip155 v math
    let odd_y_parity = if let Some(Parity(parity)) = rpc_signature.y_parity {
        parity
    } else if v >= 35 {
        // EIP-155: v = {0, 1} + CHAIN_ID * 2 + 35
        ((v - 35) % 2) != 0
    } else if v == 0 || v == 1 {
        v == 1
    } else {
        // non-EIP-155 legacy scheme, v = 27 for even y-parity, v = 28 for odd y-parity
        if v != 27 && v != 28 {
            panic!("non-eip-155 legacy v value")
        }
        v == 28
    };

    // convert the signature
    let signature = Signature {
        r: rpc_signature.r,
        s: rpc_signature.s,
        odd_y_parity,
    };

    // just condition on tx type
    let tx = if transaction.transaction_type == Some(U64::from(3)) {
        PrimitiveTransaction::Eip4844(TxEip4844 {
            chain_id,
            nonce,
            gas_limit,
            max_fee_per_gas: transaction.max_fee_per_gas.unwrap().to(),
            max_priority_fee_per_gas: transaction.max_priority_fee_per_gas.unwrap().to(),
            to,
            value,
            access_list,
            blob_versioned_hashes: transaction.blob_versioned_hashes,
            max_fee_per_blob_gas: transaction.max_fee_per_blob_gas.unwrap().to(),
            input,
        })
    } else if transaction.transaction_type == Some(U64::from(2)) {
        PrimitiveTransaction::Eip1559(TxEip1559 {
            chain_id,
            nonce,
            gas_limit,
            max_fee_per_gas: transaction.max_fee_per_gas.unwrap().to(),
            max_priority_fee_per_gas: transaction.max_priority_fee_per_gas.unwrap().to(),
            to,
            value,
            access_list,
            input,
        })
    } else if transaction.transaction_type == Some(U64::from(1)) {
        PrimitiveTransaction::Eip2930(TxEip2930 {
            chain_id,
            nonce,
            gas_price: transaction.gas_price.unwrap().to(),
            gas_limit,
            to,
            value,
            access_list,
            input,
        })
    } else {
        // otherwise legacy
        PrimitiveTransaction::Legacy(TxLegacy {
            chain_id: Some(chain_id),
            nonce,
            gas_price: transaction.gas_price.unwrap().to(),
            gas_limit,
            to,
            value,
            input,
        })
    };

    TransactionSigned::from_transaction_and_signature(tx, signature)
}
