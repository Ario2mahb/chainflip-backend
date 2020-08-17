use super::*;

use crate::side_chain::FakeSideChain;
use crate::utils::test_utils::make_valid_quote_request;

/// Populate the chain with 2 blocks, request all 2
#[tokio::test]
async fn get_all_two_blocks() {
    let params = BlocksQueryParams::new(0, 2);

    let mut side_chain = FakeSideChain::new();

    side_chain.add_block(vec![]).unwrap();
    side_chain.add_block(vec![]).unwrap();

    let side_chain = Arc::new(Mutex::new(side_chain));

    let res = APIServer::get_blocks_inner(side_chain, params).await;

    assert_eq!(res.blocks.len(), 2);
    assert_eq!(res.total_blocks, 2);
}

#[tokio::test]
async fn get_two_blocks_out_of_three() {
    use crate::utils::test_utils;

    let params = BlocksQueryParams::new(0, 2);

    let mut side_chain = FakeSideChain::new();

    side_chain.add_block(vec![]).unwrap();

    let tx = test_utils::create_fake_quote_tx();

    side_chain.add_block(vec![tx.clone().into()]).unwrap();
    side_chain.add_block(vec![]).unwrap();

    let side_chain = Arc::new(Mutex::new(side_chain));

    let res = APIServer::get_blocks_inner(side_chain, params).await;

    assert_eq!(res.blocks.len(), 2);
    assert_eq!(res.blocks[1].transactions.len(), 1);
    assert_eq!(res.total_blocks, 3);
}

#[tokio::test]
async fn cap_too_big_limit() {
    let params = BlocksQueryParams::new(1, 100);

    let mut side_chain = FakeSideChain::new();

    side_chain.add_block(vec![]).unwrap();
    side_chain.add_block(vec![]).unwrap();

    let side_chain = Arc::new(Mutex::new(side_chain));

    let res = APIServer::get_blocks_inner(side_chain, params).await;

    assert_eq!(res.blocks.len(), 1);
    assert_eq!(res.total_blocks, 2);
}

#[tokio::test]
async fn zero_limit() {
    let params = BlocksQueryParams::new(1, 0);
    let mut side_chain = FakeSideChain::new();

    side_chain.add_block(vec![]).unwrap();
    side_chain.add_block(vec![]).unwrap();

    let side_chain = Arc::new(Mutex::new(side_chain));

    let res = APIServer::get_blocks_inner(side_chain, params).await;

    assert_eq!(res.blocks.len(), 0);
    assert_eq!(res.total_blocks, 2);
}

#[tokio::test]
async fn blocks_do_not_exist() {
    let params = BlocksQueryParams::new(100, 2);

    let mut side_chain = FakeSideChain::new();

    side_chain.add_block(vec![]).unwrap();
    side_chain.add_block(vec![]).unwrap();

    let side_chain = Arc::new(Mutex::new(side_chain));

    let res = APIServer::get_blocks_inner(side_chain, params).await;

    assert_eq!(res.blocks.len(), 0);
    assert_eq!(res.total_blocks, 2);
}

#[test]
fn post_quote() {
    let params = QuoteQueryRequest {
        input_coin: Coin::LOKI,
        input_return_address: String::from("Some address"),
        input_address_id: "0".to_owned(),
        input_amount: String::from("100000"),
        output_coin: Coin::BTC,
        output_address: String::from("Some other Address"),
        slippage_limit: 1.0,
    };

    let side_chain = FakeSideChain::new();
    let side_chain = Arc::new(Mutex::new(side_chain));

    let _res = APIServer::post_quote_inner(side_chain, params);
}

#[test]
fn valid_quote_request() {
    let req = make_valid_quote_request();

    assert!(parse_quote_request(req).is_ok());
}

use crate::common::coins::COIN_PARSING_ERROR;

#[test]
fn quote_request_invalid_coin() {
    let valid_req = make_valid_quote_request();

    let mut req = valid_req.clone();
    req["inputCoin"] = serde_json::to_value("LOKKI").unwrap();
    let res = parse_quote_request(req);
    let res = res.unwrap_err();
    assert_eq!(res, COIN_PARSING_ERROR);
}
