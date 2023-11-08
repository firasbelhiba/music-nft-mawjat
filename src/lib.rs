use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Balance};
use near_sdk::json_types::U128;

near_sdk::setup_alloc!();

// Constants for the reward logic
const PLAYS_PER_REWARD: u64 = 10;
const REWARD_AMOUNT: Balance = 1_000_000_000_000_000_000_000_000; // 1 MWJ token, assuming 24 decimal places

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MusicPlatform {
    nfts: UnorderedMap<String, MusicNFT>,
    // Assuming there is a field for the contract's MWJ token balance
    mwj_balance: Balance,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MusicNFT {
    token_id: String,
    owner_id: AccountId,
    metadata: String,
    plays: u64,
    fractions: Vector<Fraction>,
    total_fractions: u64,
    fractions_remaining: u64,
    withdrawn_plays: u64, // Tracks the number of plays that have been rewarded
}

#[derive(BorshDeserialize, BorshSerialize)]
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
            fractions: Vector::new(token_id.as_bytes()),
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
        assert_eq!(owner_id, music_nft.owner_id, "Only the NFT owner can mint fractions.");

        let fraction = Fraction {
            fraction_id,
            owner_id: owner_id.clone(),
            percentage,
        };

        music_nft.fractions.push(&fraction);
        music_nft.fractions_remaining -= 1;
        self.nfts.insert(&music_nft_id, &music_nft);
    }

    pub fn redeem_rewards(&mut self, music_nft_id: String) {
        let mut music_nft = self.nfts.get(&music_nft_id).expect("Music NFT not found");
        let owner_id = env::predecessor_account_id();

        assert_eq!(owner_id, music_nft.owner_id, "Only the NFT owner can redeem rewards.");

        let new_plays = music_nft.plays - music_nft.withdrawn_plays;
        let rewards_due = (new_plays / PLAYS_PER_REWARD) as u128;

        assert!(self.mwj_balance >= rewards_due * REWARD_AMOUNT, "Insufficient MWJ balance in contract for rewards.");

        // Placeholder for the transfer logic
        // transfer_mwj_tokens(music_nft.owner_id, rewards_due * REWARD_AMOUNT);

        music_nft.withdrawn_plays  += (rewards_due * REWARD_AMOUNT) as u64;
        self.nfts.insert(&music_nft_id, &music_nft);

        self.mwj_balance = (rewards_due) * REWARD_AMOUNT;
    }

    // Getter functions
    pub fn get_music_nft(&self, token_id: String) -> Option<MusicNFT> {
        self.nfts.get(&token_id)
    }

    pub fn get_fraction(&self, music_nft_id: String, index: u64) -> Option<Fraction> {
        self.nfts.get(&music_nft_id).and_then(|nft| nft.fractions.get(index))
    }

    pub fn get_total_fractions(&self, music_nft_id: String) -> u64 {
        self.nfts.get(&music_nft_id).map_or(0, |nft| nft.total_fractions)
    }

    pub fn get_fractions_remaining(&self, music_nft_id: String) -> u64 {
        self.nfts.get(&music_nft_id).map_or(0, |nft| nft.fractions_remaining)
    }

    // Additional functions...
}

// The following are placeholder functions and should be replaced with actual logic
// for interacting with the MWJ token contract.

// Placeholder function to transfer MWJ tokens to the artist
// This function would need to be implemented according to the actual MWJ token contract.
fn transfer_mwj_tokens(to: AccountId, amount: Balance) {
    // Logic to transfer MWJ tokens
    // This would typically involve calling a function on the MWJ token contract.
}