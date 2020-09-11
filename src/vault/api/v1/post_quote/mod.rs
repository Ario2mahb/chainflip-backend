use crate::{
    common::{api::ResponseError, Coin, Timestamp, WalletAddress},
    transactions::QuoteTx,
    vault::{processor::utils::get_swap_expire_timestamp, transactions::TransactionProvider},
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod validation;

/// Params for the v1/quote endpoint
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteParams {
    input_coin: Coin,
    input_return_address: Option<String>,
    input_address_id: String,
    input_amount: String, // Amounts are strings,
    output_coin: Coin,
    output_address: String,
    slippage_limit: f32,
}

/// Response for the v1/quote endpoint
#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Serialize)]
pub struct QuoteResponse {
    id: Uuid,         // unique id
    created_at: u128, // milliseconds from epoch
    expires_at: u128, // milliseconds from epoch
    input_coin: Coin,
    input_address: String,                // Generated on the server,
    input_return_address: Option<String>, // User specified address,
    input_amount: String,
    output_coin: Coin,
    output_address: String,
    estimated_output_amount: String, // Generated on the server. Quoted amount.
    slippage_limit: f32,
}

fn bad_request(message: &str) -> ResponseError {
    ResponseError::new(StatusCode::BAD_REQUEST, message)
}

fn internal_server_error() -> ResponseError {
    ResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
}

/// Request a swap quote
pub async fn post_quote<T: TransactionProvider>(
    params: QuoteParams,
    provider: Arc<Mutex<T>>,
) -> Result<QuoteResponse, ResponseError> {
    let original_params = params.clone();

    if let Err(err) = validation::validate_params(&params) {
        return Err(bad_request(err));
    }

    // Validation of these should have been handled above
    let input_coin = params.input_coin;
    let output_coin = params.output_coin;
    let input_amount = params
        .input_amount
        .parse::<u128>()
        .map_err(|_| internal_server_error())?;

    let mut provider = provider.lock().unwrap();
    provider.sync();

    // Ensure we don't have a quote with the address
    if let Some(_) = provider.get_quote_txs().iter().find(|quote_info| {
        let quote = &quote_info.inner;
        quote.input == input_coin && quote.input_address_id == params.input_address_id
    }) {
        return Err(bad_request("Quote already exists for input address id"));
    }

    // Calculate the output amount
    let estimated_output_amount = provider
        .get_output_amount(input_coin, input_amount, output_coin)
        .map(|calculation| {
            let detail = calculation.second.unwrap_or(calculation.first);
            detail.output_amount
        })
        .unwrap_or(0);
    if estimated_output_amount == 0 {
        return Err(bad_request("Not enough liquidity"));
    }

    let effective_price = input_amount as f64 / estimated_output_amount as f64;

    // Generate addresses
    let input_address = match input_coin {
        Coin::ETH => {
            // TODO: Derive address from input_address_id
            "0x70e7db0678460c5e53f1ffc9221d1c692111dcc5"
        }
        Coin::LOKI => {
            // TODO: Generate integrated address here
            "T6SMsepawgrKXeFmQroAbuTQMqLWyMxiVUgZ6APCRFgxQAUQ1AkEtHxAgDMZJJG9HMJeTeDsqWiuCMsNahScC7ZS2StC9kHhY"
        }
        _ => {
            warn!(
                "Input address generation not implemented for {}",
                input_coin
            );
            return Err(internal_server_error());
        }
    };

    let quote = QuoteTx::new(
        Timestamp::now(),
        input_coin,
        WalletAddress::new(input_address),
        params.input_address_id,
        params.input_return_address.clone().map(WalletAddress),
        output_coin,
        WalletAddress::new(&params.output_address),
        effective_price,
        params.slippage_limit,
    )
    .map_err(|err| {
        error!(
            "Failed to create quote tx for params: {:?} due to error {}",
            original_params,
            err.clone()
        );
        bad_request(err)
    })?;

    provider
        .add_transactions(vec![quote.clone().into()])
        .map_err(|err| {
            error!("Failed to add quote transaction: {}", err);
            internal_server_error()
        })?;

    Ok(QuoteResponse {
        id: quote.id,
        created_at: quote.timestamp.0,
        expires_at: get_swap_expire_timestamp(&quote.timestamp).0,
        input_coin,
        input_address: input_address.to_string(),
        input_return_address: params.input_return_address,
        input_amount: params.input_amount,
        output_coin,
        output_address: params.output_address,
        estimated_output_amount: estimated_output_amount.to_string(),
        slippage_limit: params.slippage_limit,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        common::coins::PoolCoin, transactions::PoolChangeTx,
        utils::test_utils::get_transactions_provider,
    };

    fn params() -> QuoteParams {
        QuoteParams {
            input_coin: Coin::LOKI,
            input_return_address: Some("T6SMsepawgrKXeFmQroAbuTQMqLWyMxiVUgZ6APCRFgxQAUQ1AkEtHxAgDMZJJG9HMJeTeDsqWiuCMsNahScC7ZS2StC9kHhY".to_string()),
            input_address_id: "60900e5603bf96e3".to_owned(),
            input_amount: "1000000000".to_string(),
            output_coin: Coin::ETH,
            output_address: "0x70e7db0678460c5e53f1ffc9221d1c692111dcc5".to_string(),
            slippage_limit: 0.0,
        }
    }

    #[tokio::test]
    async fn returns_error_if_quote_exists() {
        let quote_params = params();

        let mut provider = get_transactions_provider();
        let quote = QuoteTx {
            id: Uuid::new_v4(),
            timestamp: Timestamp::now(),
            input: quote_params.input_coin,
            input_address: WalletAddress::new("T6SMsepawgrKXeFmQroAbuTQMqLWyMxiVUgZ6APCRFgxQAUQ1AkEtHxAgDMZJJG9HMJeTeDsqWiuCMsNahScC7ZS2StC9kHhY"),
            input_address_id: quote_params.input_address_id,
            return_address: quote_params.input_return_address.clone().map(WalletAddress),
            output: quote_params.output_coin,
            slippage_limit: quote_params.slippage_limit,
            output_address: WalletAddress::new(&quote_params.output_address),
            effective_price: 1.0
        };
        provider.add_transactions(vec![quote.into()]).unwrap();

        let provider = Arc::new(Mutex::new(provider));

        let result = post_quote(params(), provider)
            .await
            .expect_err("Expected post_quote to return error");

        assert_eq!(&result.message, "Quote already exists for input address id");
    }

    #[tokio::test]
    async fn returns_error_if_no_liquidity() {
        let provider = get_transactions_provider();
        let provider = Arc::new(Mutex::new(provider));

        // No pools yet
        let result = post_quote(params(), provider.clone())
            .await
            .expect_err("Expected post_quote to return error");

        assert_eq!(&result.message, "Not enough liquidity");

        // Pool with no liquidity
        {
            let tx = PoolChangeTx::new(PoolCoin::ETH, 0, 0);

            let mut provider = provider.lock().unwrap();
            provider.add_transactions(vec![tx.into()]).unwrap();
        }

        let result = post_quote(params(), provider.clone())
            .await
            .expect_err("Expected post_quote to return error");

        assert_eq!(&result.message, "Not enough liquidity");
    }

    #[tokio::test]
    async fn returns_response_if_successful() {
        let mut provider = get_transactions_provider();
        let tx = PoolChangeTx::new(PoolCoin::ETH, 10_000_000_000, 50_000_000_000);
        provider.add_transactions(vec![tx.into()]).unwrap();

        let provider = Arc::new(Mutex::new(provider));

        post_quote(params(), provider.clone())
            .await
            .expect("Expected to get a quote response");

        assert_eq!(provider.lock().unwrap().get_quote_txs().len(), 1);
    }
}
