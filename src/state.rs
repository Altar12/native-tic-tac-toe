use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::{IsInitialized, Sealed};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Game {
    pub players: [Pubkey; 2],
    pub board: [[Option<Symbol>; 3]; 3],
    pub state: GameState,
    pub turns: u8,
    pub is_initialized: bool,
}

impl IsInitialized for Game {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Sealed for Game {}
impl Game {
    pub const LEN: usize = 32 * 2 + 9 * 2 + 1 + 32 + 1 + 1;
}

#[derive(Copy, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum Symbol {
    X,
    O,
}

#[derive(PartialEq, BorshSerialize, BorshDeserialize)]
pub enum GameState {
    Unaccepted,
    Ongoing,
    Over { winner: Pubkey },
    Draw,
}

impl Default for GameState {
    fn default() -> Self {
        Self::Unaccepted
    }
}
