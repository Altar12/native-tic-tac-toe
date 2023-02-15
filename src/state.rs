use crate::error::Error;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::{IsInitialized, Sealed};
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Game {
    pub players: [Pubkey; 2],
    pub board: [[Option<Symbol>; 3]; 3],
    pub state: GameState,
    pub turns: u8,
    pub stake_mint: Pubkey,
    pub stake_amount: u64,
    pub is_initialized: bool,
}

impl IsInitialized for Game {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Sealed for Game {}
impl Game {
    pub const LEN: usize = 32 * 2 + 9 * 2 + 1 + 32 + 1 + 32 + 8 + 1;

    pub fn play(&mut self, row: usize, col: usize) -> ProgramResult {
        if self.state == GameState::Unaccepted {
            return Err(Error::UnacceptedGame.into());
        }
        if self.state != GameState::Ongoing {
            return Err(Error::GameAlreayOver.into());
        }
        if row > 3 || col > 3 {
            return Err(Error::InvalidTileSelected.into());
        }
        if let Some(_) = self.board[row][col] {
            return Err(Error::TileOccupied.into());
        }
        let symbol = if self.turns % 2 == 0 {
            Symbol::X
        } else {
            Symbol::O
        };
        self.board[row][col] = Some(symbol);
        self.turns += 1;
        self.update_state();

        Ok(())
    }
    fn update_state(&mut self) {
        let current_player = self.players[(self.turns % 2) as usize];
        for i in 0..3 {
            if let Some(_) = self.board[i][0] {
                if self.board[i][0] == self.board[i][1] && self.board[i][0] == self.board[i][2] {
                    self.state = GameState::Over {
                        winner: current_player,
                    };
                    return;
                }
            }
            if let Some(_) = self.board[0][i] {
                if self.board[0][i] == self.board[1][i] && self.board[0][i] == self.board[2][i] {
                    self.state = GameState::Over {
                        winner: current_player,
                    };
                    return;
                }
            }
        }
        if self.turns == 9 {
            self.state = GameState::Draw;
        }
    }
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
