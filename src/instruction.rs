use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

pub enum Instruction {
    CreateGame {
        player_two: Pubkey,
        stake_amount: u64,
    },
    /*
    player_one: signer, writable
    game: signer, writable,
    mint
    escrow // seeds = ["escrow", mint_pubkey]: writable
    token_account: writable
    token_program
    system_program
     */
    AcceptGame,
    /*
    player_two: signer
    game: writable
    escrow // seeds = ["escrow", mint_pubkey]: writable
    token_account: writable
    token_program
     */
    PlayGame {
        row: usize,
        col: usize,
    },
    /*
    player: signer,
    game: writable
     */
    CloseGame,
    /*
    player_one: signer,
    game: writable
     */
    CancelGame,
    /*
    player_one: signer,
    game: writable
     */
}

impl Instruction {
    pub fn unpack_from_slice(data: &[u8]) -> Result<Self, ProgramError> {
        let (&first, rest) = data.split_first().unwrap();
        let variant = match first {
            0 => {
                let player_two = Pubkey::deserialize(&mut &rest[..32])?;
                let stake_amount = u64::deserialize(&mut &rest[32..40])?;
                Self::CreateGame {
                    player_two,
                    stake_amount,
                }
            }
            1 => Self::AcceptGame,
            2 => {
                if rest.len() != 2 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                Self::PlayGame {
                    row: rest[0] as usize,
                    col: rest[1] as usize,
                }
            }
            3 => Self::CloseGame,
            4 => Self::CancelGame,
            _ => return Err(ProgramError::InvalidInstructionData),
        };
        Ok(variant)
    }
}
