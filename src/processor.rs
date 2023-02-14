use crate::instruction::Instruction;
use crate::state::{Game, GameState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    system_instruction,
    system_program::ID,
    sysvar::{rent::Rent, Sysvar},
};
use std::convert::TryInto;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let instruction = Instruction::unpack_from_slice(data)?;
    match instruction {
        Instruction::CreateGame(player_two) => create_game(program_id, accounts, player_two),
        Instruction::AcceptGame => accept_game(program_id, accounts),
        Instruction::PlayGame { row, col } => play_game(program_id, accounts, row, col),
        Instruction::CloseGame => close_game(program_id, accounts),
        Instruction::CancelGame => cancel_game(program_id, accounts),
    }
}

fn create_game(program_id: &Pubkey, accounts: &[AccountInfo], player_two: Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let player = next_account_info(account_info_iter)?;
    let game_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !player.is_signer || !game_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if *system_program.key != ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let rent_amount = Rent::get()?.minimum_balance(Game::LEN);
    invoke(
        &system_instruction::create_account(
            player.key,
            game_account.key,
            rent_amount,
            Game::LEN.try_into().unwrap(),
            program_id,
        ),
        &[player.clone(), game_account.clone()],
    )?;
    let mut game = try_from_slice_unchecked::<Game>(&game_account.data.borrow()).unwrap();
    if game.is_initialized() {
        return Err(ProgramError::InvalidAccountData);
    }
    game.players = [*player.key, player_two];
    game.board = [[None; 3]; 3];
    game.state = GameState::Unaccepted;
    game.turns = 0;
    game.is_initialized = true;
    game.serialize(&mut &mut game_account.data.borrow_mut()[..])
        .unwrap();

    Ok(())
}

fn accept_game(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}

fn play_game(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    row: usize,
    col: usize,
) -> ProgramResult {
    Ok(())
}

fn close_game(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}

fn cancel_game(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}
