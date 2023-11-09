use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env,Gas, near_bindgen, AccountId, Balance, PanicOnDefault};
use serde_derive::Serialize;
use near_sdk::collections::UnorderedMap;
use near_sdk::{ext_contract, Promise, PromiseError};

pub const TGAS: u64 = 1_000_000_000_000;

// Constants for the reward logic
const PLAYS_PER_REWARD: u64 = 10;
const REWARD_AMOUNT: Balance = 1; // 1 MWJ token

#[ext_contract(ext_ft)]
pub trait MawjatToken {
    fn mint_token(&mut self, account_id: AccountId, amount: u128);
    fn storage_deposit (&mut self, account_id: String);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MusicPlatform {
    nfts: UnorderedMap<String, MusicNFT>,
    mwj_balance: Balance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub struct MusicNFT {
    token_id: String,
    owner_id: AccountId,
    metadata: String,
    plays: u64,
    fractions: Vec<Fraction>, // Changed from Vector to Vec
    total_fractions: u64,
    fractions_remaining: u64,
    withdrawn_plays: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
pub struct Fraction {
    fraction_id: String,
    owner_id: AccountId,
    percentage: u8,
}

#[near_bindgen]
impl MusicPlatform {
    #[init]
    pub fn new() -> Self {
        Self {
            nfts: UnorderedMap::new(b"n".to_vec()),
            mwj_balance: 0, // Initial MWJ balance of the contract
        }
    }

    pub fn mint_music_nft(&mut self, token_id: String, metadata: String, total_fractions: u64) {
        let owner_id = env::predecessor_account_id();
        let new_nft = MusicNFT {
            token_id: token_id.clone(),
            owner_id: owner_id.clone(),
            metadata,
            plays: 0,
            fractions: Vec::new(), // Changed to Vec
            total_fractions,
            fractions_remaining: total_fractions,
            withdrawn_plays: 0,
        };
        self.nfts.insert(&token_id, &new_nft);
    }

    pub fn mint_fraction(&mut self, music_nft_id: String, fraction_id: String, percentage: u8) {
        let owner_id = env::predecessor_account_id();
        let mut music_nft = self.nfts.get(&music_nft_id).expect("Music NFT not found");

        assert!(music_nft.fractions_remaining > 0, "No more fractions available to mint.");

        let fraction = Fraction {
            fraction_id,
            owner_id: owner_id.clone(),
            percentage,
        };

        music_nft.fractions.push(fraction); // Changed to Vec push
        music_nft.fractions_remaining -= 1;
        self.nfts.insert(&music_nft_id, &music_nft);
    }

    // Method to increment the number of plays for a Music NFT
    // Only executable by the current contract
    pub fn increment_plays(&mut self, music_nft_id: String, increment: u64) {
        // Check if the caller is the current contract
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "This method can only be called by the current contract"
        );

        // Retrieve the Music NFT
        let mut music_nft = self.nfts.get(&music_nft_id).expect("Music NFT not found");

        // Increment the plays
        music_nft.plays = music_nft.plays.saturating_add(increment);

        // Update the Music NFT in the map
        self.nfts.insert(&music_nft_id, &music_nft);
    }

    pub fn redeem_rewards(&mut self, music_nft_id: String) {
        let mut music_nft = self.nfts.get(&music_nft_id).expect("Music NFT not found");
        let owner_id = env::predecessor_account_id();

        assert_eq!(owner_id, music_nft.owner_id, "Only the NFT owner can redeem rewards.");

        let new_plays = music_nft.plays - music_nft.withdrawn_plays;
        let rewards_due = (new_plays / PLAYS_PER_REWARD) as u128;

        // assert!(self.mwj_balance >= rewards_due * REWARD_AMOUNT, "Insufficient MWJ balance in contract for rewards.");

        // Placeholder for the transfer logic
        self.transfer_mwj_tokens(rewards_due * REWARD_AMOUNT );

        music_nft.withdrawn_plays += new_plays;
        self.nfts.insert(&music_nft_id, &music_nft);

        self.mwj_balance -= rewards_due * REWARD_AMOUNT; // Adjusted balance reduction
    }

    

    // Getter functions
    
    // Function to get all nfts
    pub fn get_all_music_nft(&self) -> Vec<MusicNFT> {
        self.nfts.values_as_vector().to_vec()
    }

    // Function to get details of a specific Music NFT
    pub fn get_music_nft(&self, token_id: String) -> Option<MusicNFT> {
        self.nfts.get(&token_id)
    }

    // Function to get details of a specific Music NFT
    pub fn get_user_music_nft(&self, owner_id: String) -> Vec<MusicNFT> {
        let mut res = Vec::new();
        for i in 0..self.nfts.values_as_vector().to_vec().len() {
            if self.nfts.values_as_vector().get(i.try_into().unwrap()).unwrap().owner_id.to_string() == owner_id {
                res.push(self.nfts.values_as_vector().get(i.try_into().unwrap()).unwrap());
            } 
        }
        res
    }
    
    // Function to get a specific fraction of a Music NFT
    pub fn get_fraction(&self, music_nft_id: String) -> Vec<Fraction> {
        self.get_music_nft(music_nft_id).unwrap().fractions
    }

    // Function to get the total number of fractions of a Music NFT
    pub fn get_total_fractions(&self, music_nft_id: String) -> u64 {
        self.nfts.get(&music_nft_id).map_or(0, |nft| nft.total_fractions)
    }

    // Function to get the number of remaining fractions available for minting for a Music NFT
    pub fn get_fractions_remaining(&self, music_nft_id: String) -> u64 {
        self.nfts.get(&music_nft_id).map_or(0, |nft| nft.fractions_remaining)
    }

    // Function to get the total plays of a Music NFT
    pub fn get_plays(&self, music_nft_id: String) -> u64 {
        self.nfts.get(&music_nft_id).map_or(0, |nft| nft.plays)
    }

    // Function to get the withdrawn plays (plays that have been rewarded) of a Music NFT
    pub fn get_withdrawn_plays(&self, music_nft_id: String) -> u64 {
        self.nfts.get(&music_nft_id).map_or(0, |nft| nft.withdrawn_plays)
    }

    // Function to get the owner of a Music NFT
    pub fn get_owner(&self, music_nft_id: String) -> Option<AccountId> {
        self.nfts.get(&music_nft_id).map(|nft| nft.owner_id)
    }

    // Backup Functions 

    // Function to mint MWJ 
    pub fn mint_lts (&mut self, amount:u128) -> Promise {
        let contract_account = "mawjat_token.testnet".to_string().try_into().unwrap();

        let promise=ext_ft::ext(contract_account)
            .with_static_gas(Gas(5_000_000_000_000))
            .mint_token(env::signer_account_id(), amount*100000000);

        return promise.then( // Create a promise to callback withdraw_callback
            Self::ext(env::current_account_id())
            .with_static_gas(Gas(3 * TGAS))
            .mint_lts_callback()
            )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn mint_lts_callback(&mut self, #[callback_result] call_result: Result<(), PromiseError> ) {
        // Check if the promise succeeded
        if call_result.is_err() {
        panic!("There was an error contacting the token contract");
        }
    }

    // Function to add the user in the storage of the MWJ token
    pub fn add_storage_deposit (&mut self) -> Promise{
        let contract_account = "mawjat_token.testnet".to_string().try_into().unwrap();

        let promise=ext_ft::ext(contract_account)
            .with_attached_deposit(1000000000000000000000000)
            .with_static_gas(Gas(5_000_000_000_000))
            .storage_deposit(env::signer_account_id().to_string());

            return promise.then( // Create a promise to callback withdraw_callback
                Self::ext(env::current_account_id())
                .with_static_gas(Gas(3 * TGAS))
                .add_storage_callback()
                )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn add_storage_callback(&mut self, #[callback_result] call_result: Result<(), PromiseError> ) {
        // Check if the promise succeeded
        if call_result.is_err() {
        panic!("There was an error contacting the token contract");
        }
    }

    // Placeholder function to transfer MWJ tokens
    // (to be implemented as per actual MWJ token contract logic)
    fn transfer_mwj_tokens(&mut self, amount: Balance) {
        // Transfer MWJ tokens logic
        self.add_storage_deposit();
        self.mint_lts(amount);
    }
}

