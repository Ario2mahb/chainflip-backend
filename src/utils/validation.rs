use crate::common::{ethereum, LokiPaymentId, LokiWalletAddress};
use chainflip_common::types::coin::Coin;
use std::str::FromStr;

/// Validate an address from the given `coin`
pub fn validate_address(coin: Coin, address: &str) -> Result<(), String> {
    match coin {
        Coin::LOKI => LokiWalletAddress::from_str(address).map(|_| ()),
        Coin::ETH => ethereum::Address::from_str(address)
            .map(|_| ())
            .map_err(|str| str.to_owned()),
        Coin::BTC => bitcoin::Address::from_str(address)
            .map(|_| ())
            .map_err(|err| err.to_string()),
    }
}

/// Validate an address id for the given coin
pub fn validate_address_id(coin: Coin, address_id: &str) -> Result<(), String> {
    match coin {
        Coin::BTC | Coin::ETH => match address_id.parse::<u32>() {
            // Index 0 is used for the main wallet and 1-4 are reserved for future use
            Ok(id) => {
                if id < 5 {
                    Err("Address id must be greater than 5".to_owned())
                } else {
                    Ok(())
                }
            }
            Err(_) => Err("Address id must be an signed integer".to_owned()),
        },
        Coin::LOKI => LokiPaymentId::from_str(address_id).map(|_| ()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn validates_address() {
        let invalid = "hello";
        let loki_address = "T6SMsepawgrKXeFmQroAbuTQMqLWyMxiVUgZ6APCRFgxQAUQ1AkEtHxAgDMZJJG9HMJeTeDsqWiuCMsNahScC7ZS2StC9kHhY";
        let eth_address = "0x70e7db0678460c5e53f1ffc9221d1c692111dcc5";

        // loki
        assert!(validate_address(Coin::LOKI, loki_address).is_ok());
        assert!(validate_address(Coin::LOKI, invalid).is_err());

        // eth
        assert!(validate_address(Coin::ETH, eth_address).is_ok());
        assert!(validate_address(Coin::ETH, invalid).is_err());

        // BTC - p2pkh
        assert!(&validate_address(Coin::BTC, "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2").is_ok());
        assert!(&validate_address(Coin::BTC, "1certainlyaninvalidaddress").is_err());

        // BTC - p2sh
        assert!(&validate_address(Coin::BTC, "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy").is_ok());
        assert!(&validate_address(Coin::BTC, "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLb").is_err());

        // BTC - bech32
        assert!(&validate_address(Coin::BTC, "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq").is_ok());
        assert!(
            &validate_address(Coin::BTC, "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5md2").is_err()
        );
    }

    #[test]
    pub fn validates_eth_address_id() {
        assert!(validate_address_id(Coin::ETH, "5").is_ok());
        assert_eq!(
            &validate_address_id(Coin::ETH, "4").unwrap_err(),
            "Address id must be greater than 5"
        );
        assert_eq!(
            validate_address_id(Coin::ETH, "id").unwrap_err(),
            "Address id must be an signed integer"
        );
        assert_eq!(
            validate_address_id(Coin::ETH, "-5").unwrap_err(),
            "Address id must be an signed integer"
        );
    }

    #[test]
    pub fn validates_btc_address_id() {
        assert!(validate_address_id(Coin::BTC, "5").is_ok());
        assert_eq!(
            &validate_address_id(Coin::BTC, "4").unwrap_err(),
            "Address id must be greater than 5"
        );
        assert_eq!(
            validate_address_id(Coin::BTC, "id").unwrap_err(),
            "Address id must be an signed integer"
        );
        assert_eq!(
            validate_address_id(Coin::BTC, "-5").unwrap_err(),
            "Address id must be an signed integer"
        );
    }

    #[test]
    pub fn validates_loki_address_id() {
        assert!(validate_address_id(Coin::LOKI, "60900e5603bf96e3").is_ok());
        assert!(validate_address_id(
            Coin::LOKI,
            "60900e5603bf96e3000000000000000000000000000000000000000000000000"
        )
        .is_ok());

        assert!(validate_address_id(Coin::LOKI, "5").is_err());
        assert!(validate_address_id(Coin::LOKI, "invalid").is_err());
        assert!(validate_address_id(Coin::LOKI, "60900e5603bf96H").is_err());
    }
}
