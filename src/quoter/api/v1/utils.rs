use super::InputIdCache;
use crate::common::api::ResponseError;
use chainflip_common::types::coin::Coin;
use rand::Rng;
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

/// Generate a uniqe input address id and insert it into the cache
pub fn generate_unique_input_address_id<R: Rng>(
    input_coin: Coin,
    input_id_cache: Arc<Mutex<InputIdCache>>,
    rng: &mut R,
) -> Result<Vec<u8>, ResponseError> {
    let mut cache = input_id_cache.lock().unwrap();
    let used_ids = cache.entry(input_coin).or_insert(BTreeSet::new());

    // We can test this by passing a SeededRng
    let input_address_id = loop {
        let id = match input_coin {
            // BTC and ETH have u32 indexes which we can derive an address through hd wallets
            Coin::BTC | Coin::ETH => rng.gen_range(5, u32::MAX).to_be_bytes().to_vec(),
            // LOKI has 8 random bytes which represent a payment id
            Coin::LOKI => rng.gen::<[u8; 8]>().to_vec(),
        };

        if !used_ids.contains(&id) {
            break id;
        }
    };

    // Add the id in the cache
    used_ids.insert(input_address_id.clone());

    Ok(input_address_id)
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::{prelude::StdRng, SeedableRng};
    use std::collections::HashMap;

    fn cache() -> InputIdCache {
        let mut cache: InputIdCache = HashMap::new();
        cache.insert(Coin::LOKI, BTreeSet::new());
        cache.insert(Coin::ETH, BTreeSet::new());
        cache.insert(Coin::BTC, BTreeSet::new());
        cache
    }

    #[test]
    fn generates_id_and_inserts_to_cache() {
        let mut rng = StdRng::seed_from_u64(0);
        let cache = cache();
        let cache = Arc::new(Mutex::new(cache));

        for coin in vec![Coin::LOKI, Coin::ETH, Coin::BTC] {
            generate_unique_input_address_id(coin, cache.clone(), &mut rng)
                .expect(&format!("Expected to generate unique id for {}", coin));
            assert_eq!(cache.lock().unwrap().get(&coin).unwrap().len(), 1);
        }
    }

    #[test]
    fn generates_unique_ids() {
        let seed = 0;
        let cache = Arc::new(Mutex::new(cache()));

        // The expected first ids generated by the rng with the given seed
        let mut first_ids = HashMap::new();
        first_ids.insert(Coin::ETH, vec![129, 245, 247, 179]);
        first_ids.insert(Coin::BTC, vec![129, 245, 247, 179]);
        first_ids.insert(Coin::LOKI, vec![178, 214, 168, 126, 192, 105, 52, 255]);

        // generare first ids
        for coin in vec![Coin::LOKI, Coin::ETH, Coin::BTC] {
            let mut rng = StdRng::seed_from_u64(seed);
            generate_unique_input_address_id(coin, cache.clone(), &mut rng)
                .expect(&format!("Expected to generate unique id for {}", coin));

            let expected = first_ids.get(&coin).unwrap();
            {
                let coin_cache = cache.lock().unwrap();
                let set = coin_cache.get(&coin).unwrap();
                assert_eq!(set.len(), 1);
                assert!(
                    set.contains(expected),
                    "Set doesn't contain expected value for {}. Expected: {:?}. Got: {:?}",
                    coin,
                    expected,
                    set
                );
            }
        }

        // Generate other ids
        for coin in vec![Coin::LOKI, Coin::ETH, Coin::BTC] {
            let mut rng = StdRng::seed_from_u64(seed);
            generate_unique_input_address_id(coin, cache.clone(), &mut rng)
                .expect(&format!("Expected to generate unique id for {}", coin));
            assert_eq!(cache.lock().unwrap().get(&coin).unwrap().len(), 2);
        }
    }
}
