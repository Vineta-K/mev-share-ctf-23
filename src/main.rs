pub mod abi;
pub mod ctf;

use std::sync::Arc;

use dotenvy::{dotenv, var};
use ethers::abi::AbiEncode;
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use eyre::Result;
use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;
use jsonrpsee::http_client::{transport::Error as HttpError, HttpClientBuilder};
use mev_share::rpc::{
    BundleItem, FlashbotsSignerLayer, Inclusion, MevApiClient, SendBundleRequest,
};
use mev_share::sse::EventClient;
use tower::ServiceBuilder;
use tracing::{info, warn};
use tracing_subscriber::{filter::EnvFilter, fmt::Subscriber};

use crate::ctf::Flag;
use abi::mev_share_ctf_simple::MevShareCTFSimpleCalls;
use abi::mev_share_ctf_triple::MevShareCTFTripleCalls;
use abi::mev_share_magic_number_v3::MevShareMagicNumberCalls;
use abi::mev_share_new_contract::MevShareNewContractCalls;

#[tokio::main]
async fn main() -> Result<()> {
    //load env
    dotenv()?;
    let fb_signer = var("FlashbotKey")?.parse::<LocalWallet>()?;
    let tx_signer = var("BotKey")?.parse::<LocalWallet>()?;
    let goerli_endpoint = var("EthereumApi")?;

    //set up tracing
    Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    //mev-share-sse client
    let goerli_sse = "https://mev-share-goerli.flashbots.net";
    let event_client = EventClient::default();
    let mut mev_share_stream = event_client.events(goerli_sse).await.unwrap();
    info!("Subscribed to {}", mev_share_stream.endpoint());

    //mev-share-bundle-client
    let signing_middleware = FlashbotsSignerLayer::new(fb_signer);
    let service_builder = ServiceBuilder::new()
        // map signer errors to http errors
        .map_err(|e| HttpError::Http(e))
        .layer(signing_middleware);
    let url = "https://relay-goerli.flashbots.net:443";
    let bundle_client = HttpClientBuilder::default()
        .set_middleware(service_builder)
        .build(url)
        .expect("Failed to create http client");
    let bundle_client = Arc::new(bundle_client);

    //goerli-client
    let ws = Ws::connect(goerli_endpoint).await?;
    let provider = Provider::new(ws);
    let client = Arc::new(provider);

    //map of address -> contract flag type
    let contracts = ctf::contracts();

    while let Some(event) = mev_share_stream.next().await {
        let event = event?;
        info!("{:?}", event);

        //get contract address jank
        let contract_address = get_contract_address(&event);
        if let Err(e) = contract_address {
            warn!("{:?}", e);
            continue;
        }
        let contract_address = contract_address.unwrap();

        let client = client.clone();
        let bundle_client = bundle_client.clone();
        let tx_signer = tx_signer.clone();
        match contracts.get(&contract_address) {
            //this is fucking horrific lmao but i cba fighting with futures to refactor
            Some(Flag::CTFSimple(false)) => {
                tokio::spawn(async move {
                    let res =
                        solve_ctf_simple(event, contract_address, client, bundle_client, tx_signer)
                            .await;
                    match res {
                        Ok(_) => (),
                        Err(e) => warn!("{:?}", e),
                    }
                });
            }
            Some(Flag::CTFTriple(false)) => {
                tokio::spawn(async move {
                    let res =
                        solve_ctf_triple(event, contract_address, client, bundle_client, tx_signer)
                            .await;
                    match res {
                        Ok(_) => (),
                        Err(e) => warn!("{:?}", e),
                    }
                });
            }
            Some(Flag::MagicNumberV1(false)) => {
                tokio::spawn(async move {
                    let res = solve_magic_number(
                        event,
                        contract_address,
                        client,
                        bundle_client,
                        tx_signer,
                        Flag::MagicNumberV1(false),
                    )
                    .await;
                    match res {
                        Ok(_) => (),
                        Err(e) => warn!("{:?}", e),
                    }
                });
            }
            Some(Flag::MagicNumberV2(false)) => {
                tokio::spawn(async move {
                    let res = solve_magic_number(
                        event,
                        contract_address,
                        client,
                        bundle_client,
                        tx_signer,
                        Flag::MagicNumberV2(false),
                    )
                    .await;
                    match res {
                        Ok(_) => (),
                        Err(e) => warn!("{:?}", e),
                    }
                });
            }
            Some(Flag::MagicNumberV3(false)) => {
                tokio::spawn(async move {
                    let res = solve_magic_number(
                        event,
                        contract_address,
                        client,
                        bundle_client,
                        tx_signer,
                        Flag::MagicNumberV3(false),
                    )
                    .await;
                    match res {
                        Ok(_) => (),
                        Err(e) => warn!("{:?}", e),
                    }
                });
            }
            Some(Flag::NewContracts(false)) => {
                tokio::spawn(async move {
                    let res = solve_new_contracts(
                        event,
                        contract_address,
                        client,
                        bundle_client,
                        tx_signer,
                    )
                    .await;
                    match res {
                        Ok(_) => (),
                        Err(e) => warn!("{:?}", e),
                    }
                });
            }
            _ => (),
        }
    }
    Ok(())
}

fn get_contract_address(event: &mev_share::sse::Event) -> Result<Address> {
    //pretty questionable but works for this ctf...
    match event {
        e if event.transactions.len() == 1 => {
            if let Some(address) = e.transactions[0].to {
                Ok(address)
            } else {
                Ok("0x20a1A5857fDff817aa1BD8097027a841D4969AA5"
                    .parse::<Address>()
                    .unwrap())
            }
        }
        e if event.logs.len() == 1 => Ok(e.logs[0].address),
        _ => Ok("0x20a1A5857fDff817aa1BD8097027a841D4969AA5"
            .parse::<Address>()
            .unwrap()),
    }
}

async fn solve_ctf_simple(
    event: mev_share::sse::Event,
    contract_address: Address,
    client: Arc<Provider<Ws>>,
    bundle_client: Arc<impl MevApiClient>,
    tx_signer: LocalWallet,
) -> Result<()> {
    let data =
        MevShareCTFSimpleCalls::ClaimReward(abi::mev_share_ctf_simple::ClaimRewardCall).encode();
    let nonce = client
        .get_transaction_count(tx_signer.address(), None)
        .await?;
    let solution_bytes =
        populate_solution_tx(contract_address, data, &client, &tx_signer, nonce.as_u64()).await?;
    let block_number = client.get_block_number().await?;

    send_solution_backrun(
        event.hash,
        vec![solution_bytes],
        bundle_client,
        block_number,
        &Flag::CTFSimple(false),
    )
    .await?;
    Ok(())
}

async fn solve_ctf_triple(
    event: mev_share::sse::Event,
    contract_address: Address,
    client: Arc<Provider<Ws>>,
    bundle_client: Arc<impl MevApiClient>,
    tx_signer: LocalWallet,
) -> Result<()> {
    let data =
        MevShareCTFTripleCalls::ClaimReward(abi::mev_share_ctf_triple::ClaimRewardCall).encode();
    let nonce = client
        .get_transaction_count(tx_signer.address(), None)
        .await?
        .as_u64();
    let mut solution_bytes = Vec::new();
    for n in nonce..(nonce + 3) {
        solution_bytes.push(
            populate_solution_tx(contract_address, data.clone(), &client, &tx_signer, n).await?,
        );
    }
    let block_number = client.get_block_number().await?;

    send_solution_backrun(
        event.hash,
        solution_bytes,
        bundle_client,
        block_number,
        &Flag::CTFTriple(false),
    )
    .await?;
    Ok(())
}

async fn solve_magic_number(
    mut event: mev_share::sse::Event,
    contract_address: Address,
    client: Arc<Provider<Ws>>,
    bundle_client: Arc<impl MevApiClient>,
    tx_signer: LocalWallet,
    flag: Flag,
) -> Result<()> {
    let log = event.logs.pop().unwrap(); //hehe
    let log = Log {
        address: log.address,
        topics: log.topics,
        data: log.data,
        ..Default::default()
    };
    let parsed: abi::mev_share_magic_number_v3::ActivateFilter = ethers::contract::parse_log(log)?;
    let lower_bound = parsed.lower_bound.as_u64();
    let upper_bound = parsed.upper_bound.as_u64();

    let nonce = client
        .get_transaction_count(tx_signer.address(), None)
        .await?
        .as_u64();
    let block_number = client.get_block_number().await?;

    //doing this concurrently cause i query the rpc to fill transaction and its kinda slow otherwise
    let mut futs = FuturesUnordered::new();
    for m in lower_bound..upper_bound {
        let data = MevShareMagicNumberCalls::ClaimReward(
            abi::mev_share_magic_number_v3::ClaimRewardCall {
                magic_number: U256::from(m),
            },
        )
        .encode();
        futs.push(populate_solution_tx(
            contract_address,
            data,
            &client,
            &tx_signer,
            nonce,
        ));
    }
    let mut solutions = Vec::new();
    while let Some(bytes) = futs.next().await {
        solutions.push(bytes?);
    }

    //send backruns concurrently
    let mut futs = FuturesUnordered::new();
    for solution_bytes in solutions {
        futs.push(send_solution_backrun(
            event.hash,
            vec![solution_bytes],
            bundle_client.clone(),
            block_number,
            &flag,
        ))
    }
    //wait for response from all backruns
    while let Some(res) = futs.next().await {
        if let Err(e) = res {
            warn!("Error sending magicnumber spam backrun {:?}", e);
        }
    }

    Ok(())
}

async fn solve_new_contracts(
    mut event: mev_share::sse::Event,
    contract_address: Address,
    client: Arc<Provider<Ws>>,
    bundle_client: Arc<impl MevApiClient>,
    tx_signer: LocalWallet,
) -> Result<()> {
    let log = event.logs.pop().unwrap(); //hehe
    let log = Log {
        address: log.address,
        topics: log.topics,
        data: log.data,
        ..Default::default()
    };

    //find new contract address
    let new_contract_address: Address;
    let topic0 = log.topics[0];
    if format!("{:?}", topic0) //format! for ethers quirks
        == String::from("0xf7e9fe69e1d05372bc855b295bc4c34a1a0a5882164dd2b26df30a26c1c8ba15")
    {
        let parsed: abi::mev_share_new_contracts::ActivateFilter =
            ethers::contract::parse_log(log)?;
        new_contract_address = parsed.newly_deployed_contract;
    } else {
        let parsed: abi::mev_share_new_contracts::ActivateBySaltFilter =
            ethers::contract::parse_log(log)?;
        let salt = parsed.salt;
        let bytecode = ethers::utils::hex::decode("60a060405233608052436000556080516101166100266000396000606f01526101166000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c806396b81609146037578063b88a802f146051575b600080fd5b603f60005481565b60405190815260200160405180910390f35b60576059565b005b4360005414606657600080fd5b600080819055507f00000000000000000000000000000000000000000000000000000000000000006001600160a01b031663720ecf456040518163ffffffff1660e01b8152600401600060405180830381600087803b15801560c757600080fd5b505af115801560da573d6000803e3d6000fd5b5050505056fea26469706673582212207a00db890eff47285ac0d9c9b8735727d476952aa87b45ee82fd6bb4f42c6fa764736f6c63430008130033").unwrap();
        new_contract_address = ethers::utils::get_create2_address(contract_address, salt, bytecode);
    }
    let data = MevShareNewContractCalls::ClaimReward(abi::mev_share_new_contract::ClaimRewardCall)
        .encode();
    let nonce = client
        .get_transaction_count(tx_signer.address(), None)
        .await?;
    let solution_bytes = populate_solution_tx(
        new_contract_address,
        data,
        &client,
        &tx_signer,
        nonce.as_u64(),
    )
    .await?;
    let block_number = client.get_block_number().await?;
    send_solution_backrun(
        event.hash,
        vec![solution_bytes],
        bundle_client,
        block_number,
        &Flag::NewContracts(false),
    )
    .await?;
    Ok(())
}

async fn send_solution_backrun(
    target_hash: TxHash,
    solutions: Vec<Bytes>,
    bundle_client: Arc<impl MevApiClient>,
    block_number: U64,
    flag: &Flag,
) -> Result<()> {
    let mut bundle_body = Vec::new();
    bundle_body.push(BundleItem::Hash { hash: target_hash });
    for solution in solutions {
        bundle_body.push(BundleItem::Tx {
            tx: solution,
            can_revert: false,
        });
    }

    let bundle = SendBundleRequest {
        bundle_body,
        inclusion: Inclusion {
            max_block: None,
            block: block_number + 1,
        },
        ..Default::default()
    };
    info!("Sending {:?} bundle: {:?}", flag, bundle);
    let resp = bundle_client.send_bundle(bundle).await;
    info!("Sent {:?} bundle: {:?}", flag, resp);
    // Simulate bundle
    Ok(())
}

async fn populate_solution_tx(
    contract_address: Address,
    data: Vec<u8>,
    client: &Arc<Provider<Ws>>,
    tx_signer: &LocalWallet,
    nonce: u64,
) -> Result<Bytes> {
    let mut solution_tx: TypedTransaction = Eip1559TransactionRequest::new()
        .from(tx_signer.address())
        .to(contract_address)
        .data(data)
        .gas(690_420)
        .nonce(nonce)
        .chain_id(5)
        .value(0)
        .into();
    client.fill_transaction(&mut solution_tx, None).await?; //this makes whole fn slow probably should have just hardcoded gas prices
    let signature = tx_signer
        .sign_transaction(&solution_tx.clone().into())
        .await?;
    let solution_bytes = solution_tx.rlp_signed(&signature);
    Ok(solution_bytes)
}
