#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use game_proxy::ProxyTrait as _;

mod game_proxy {
    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait Game {
        #[payable("*")]
        #[endpoint(depositEcity)]
        fn deposit_ecity(&self);
    }
}

/// Elrond City's router smart contract. Distributes the newly minted $ECITY to the reward contracts.
#[elrond_wasm::contract]
pub trait Router {
    #[init]
    fn init(&self) {}

    #[proxy]
    fn game_proxy(&self, sc_address: ManagedAddress) -> game_proxy::Proxy<Self::Api>;

    // Maps the reward contract addresses to the reward percentages they will receive.
    // The percentages are stored as u64, but they are actually percentages with 2 decimals (e.g. 10000 = 100%).
    // Percentages MUST add up to 100%.
    #[view(distribution)]
    #[storage_mapper("distribution")]
    fn distribution(&self) -> MapMapper<ManagedAddress, u64>;

    #[view(ecityTokenId)]
    #[storage_mapper("ecity_token_id")]
    fn ecity_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(gameAddress)]
    #[storage_mapper("game_address")]
    fn game_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(minterAddress)]
    #[storage_mapper("minter_address")]
    fn minter_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("locked")] // Once locked, the distribution cannot be changed.
    fn locked(&self) -> SingleValueMapper<bool>;

    #[only_owner]
    #[endpoint(addDistribution)]
    fn add_distribution(&self, address: ManagedAddress, percentage: u64) {
        require!(!self.locked().get(), "Distribution is locked");
        require!(percentage > 0, "Percentage must be greater than 0");
        require!(percentage <= 10000, "Percentage must be less than or equal to 10000");

        let mut total_percentage = 0;
        for (_address, perc) in self.distribution().iter() {
            total_percentage += perc;
        }

        require!(total_percentage + percentage <= 10000, "Total percentage must be less than or equal to 10000");

        self.distribution().insert(address, percentage);

    }

    #[only_owner]
    #[endpoint(setGameAddress)]
    fn set_game_address(&self, address: ManagedAddress) {
        self.game_address().set(&address);
    }

    #[only_owner]
    #[endpoint(setMinterAddress)]
    fn set_minter_address(&self, address: ManagedAddress) {
        self.minter_address().set(&address);
    }

    #[only_owner]
    #[endpoint(removeDistribution)]
    fn remove_distribution(&self, address: ManagedAddress) {
        require!(!self.locked().get(), "Distribution is locked");
        self.distribution().remove(&address);
    }

    #[only_owner]
    #[endpoint(lockDistribution)]
    fn lock_distribution(&self) {
        self.locked().set(&true);
    }

    #[only_owner]
    #[endpoint(addToken)]
    fn set_token(&self, token_id: TokenIdentifier) {
        self.ecity_token_id().set(&token_id);
    }

    #[payable("*")]
    #[endpoint(distribute)]
    fn distribute(&self) {
        require!(self.blockchain().get_caller() == self.minter_address().get(), "Only the minter can call this function");

        let mut total_percentage = 0;
        for (_address, percentage) in self.distribution().iter() {
            total_percentage += percentage;
        }

        require!(total_percentage == 10000, "Total percentage must be 10000");

        let (payment_token, _payment_value) = self.call_value().single_fungible_esdt();

        require!(payment_token == self.ecity_token_id().get(), "Invalid token");
        
        let wrapped_id = EgldOrEsdtTokenIdentifier::esdt(self.ecity_token_id().get());
        let ecity_amount = self.blockchain().get_sc_balance(&wrapped_id, 0); 

        for (address, percentage) in self.distribution().iter() {
            let amount = ecity_amount.clone() * percentage / BigUint::from(10000u64);

            if &address == &self.game_address().get() {
                self.game_proxy(self.game_address().get())
                    .deposit_ecity()
                    .add_esdt_token_transfer(self.ecity_token_id().get(), 0, amount)
                    .transfer_execute();
            } else {
                self.send().direct_esdt(&address, &self.ecity_token_id().get(), 0, &amount);
            }
        }
    }
}
