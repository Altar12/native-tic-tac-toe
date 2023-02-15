use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("You can not accept the provided game")]
    UnauthorizedToAccept,
    #[error("Can not accept a game twice")]
    AlreadyAccepted,
    #[error("Game has not been accepted yet")]
    UnacceptedGame,
    #[error("Game is already over")]
    GameAlreayOver,
    #[error("The tile position specified is invalid")]
    InvalidTileSelected,
    #[error("Selected tile is already occupied")]
    TileOccupied,
}

impl From<Error> for ProgramError {
    fn from(e: Error) -> Self {
        Self::Custom(e as u32)
    }
}
