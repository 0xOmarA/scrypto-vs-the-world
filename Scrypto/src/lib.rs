use scrypto::prelude::*;

blueprint! {
    struct Sample {}

    impl Sample {
        pub fn new() -> Bucket {
            let token: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Sample Token")
                .metadata("symbol", "SAMPLE")
                .initial_supply(1_000);

            return token;
        }
    }
}