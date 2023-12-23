#![cfg_attr(not(feature = "std"), no_std, no_main)]


mod filemap;
mod scoob;

#[ink::contract]
mod scooby {

    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;
    use ink::prelude::string::ToString;
    use secp256k1::hashes::sha256;
    use secp256k1::Message;
    use secp256k1::rand::rngs::OsRng;

    #[ink(storage)]
    pub struct Scooby{
        pub balances: Mapping<AccountId, Balance>,
        pub rawdata: Vec<u8>,
        pub datahash: Vec<u8>
    }

    impl Default for Scooby {
        fn default() -> Self {
            Scooby{
                balances: Mapping::default(),
                rawdata: Vec::new(),
                datahash: Vec::new()
            }
        }
    }

    impl Scooby{
        #[ink(constructor)]
        pub fn new(rawdata:Vec<u8>) -> Self {
            let secp = secp256k1::Secp256k1::new();
            let (sk, _) = secp.generate_keypair(&mut OsRng);
            let msg_info: &[u8] = rawdata.as_slice();
            let secp_msg = Message::from_hashed_data::<sha256::Hash>(msg_info);
            let secp_encode = secp.sign_ecdsa(&secp_msg, &sk);
            let balances = Mapping::default();
            let datahash = secp_encode.to_string().as_bytes().to_vec();
            let scooby_resp = Scooby{balances, rawdata, datahash};
            scooby_resp
        }

        #[ink(message, payable)]
        pub fn validate_data_and_transfer(&mut self, ext_data:Vec<u8>) {
            let caller = self.env().caller();
            let balance = self.balances.get(caller).unwrap_or(0);
            let val_data = self.datahash == ext_data;
            let tx_val = self.env().transferred_value();
            if balance > tx_val && val_data {
                self.balances.insert(caller, &(balance + tx_val));
            }
        }

        #[ink(message)]
        pub fn get(&self) {
            Scooby::default();
        }

        #[ink(message)]
        pub fn scooby_withdraw(&mut self, ) {
            let caller = self.env().account_id();
            let balances = self.balances.get(caller).unwrap();
            self.balances.remove(caller);
            self.env().transfer(caller, balances).unwrap();
        }

        #[ink(message)]
        pub fn scooby_balance(&self) -> u128 {
            let scooby = self.env().caller();
            let sbalance = self.balances.get(scooby).unwrap_or(0);
            sbalance
        }

        #[ink(message)]
        pub fn get_valid_datahash(&self, rawdata:Vec<u8>) -> Option<Vec<u8>> {
            let scooby_constructor = Scooby::new(rawdata);
            let hash_from_constructor = scooby_constructor.datahash;
            Some(hash_from_constructor)
        }

        pub fn match_valid_scooby(&self, rawdata:Vec<u8>) -> Option<Self> {
            let scooby_new_from_constructor = Self::new(rawdata);
            Some(scooby_new_from_constructor)
        }

    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let scooby = Scooby::default();
            assert_eq!(scooby.scooby_balance(), 0);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let scp_data = "data".to_string().as_bytes().to_vec();
            let scooby = Scooby::new(scp_data.clone());
            assert_eq!(scooby.get_valid_datahash(scp_data), Some(scooby.datahash));
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = ScoobyRef::default();

            // When
            let contract_account_id = client
                .instantiate("scooby", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<ScoobyRef>(contract_account_id.clone())
                .call(|scooby| scooby.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = ScoobyRef::new(false);
            let contract_account_id = client
                .instantiate("scooby", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<ScoobyRef>(contract_account_id.clone())
                .call(|scooby| scooby.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<ScoobyRef>(contract_account_id.clone())
                .call(|scooby| scooby.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<ScoobyRef>(contract_account_id.clone())
                .call(|scooby| scooby.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
