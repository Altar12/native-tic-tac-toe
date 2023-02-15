use crate::error::Error;
use crate::instruction::Instruction;
use crate::state::{Game, GameState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    system_instruction,
    system_program::ID as SYSTEM_PROGRAM_ID,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::{instruction, state::Account, ID as TOKEN_PROGRAM_ID};
use std::convert::TryInto;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let instruction = Instruction::unpack_from_slice(data)?;
    match instruction {
        Instruction::CreateGame {
            player_two,
            stake_amount,
        } => create_game(program_id, accounts, player_two, stake_amount),
        Instruction::AcceptGame => accept_game(program_id, accounts),
        Instruction::PlayGame { row, col } => play_game(program_id, accounts, row, col),
        Instruction::CloseGame => close_game(program_id, accounts),
        Instruction::CancelGame => cancel_game(program_id, accounts),
    }
}

fn create_game(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    player_two: Pubkey,
    stake_amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let player = next_account_info(account_info_iter)?;
    let game_account = next_account_info(account_info_iter)?;
    let mint = next_account_info(account_info_iter)?;
    let escrow = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // data and accounts validation
    if stake_amount == 0 || player_two == *player.key {
        return Err(ProgramError::InvalidArgument);
    }
    if !player.is_signer || !game_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if *system_program.key != SYSTEM_PROGRAM_ID || *token_program.key != TOKEN_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let (escrow_key, bump) = Pubkey::find_program_address(
        &["escrow".as_bytes().as_ref(), mint.key.as_ref()],
        program_id,
    );
    if *escrow.key != escrow_key {
        return Err(ProgramError::IncorrectProgramId);
    }
    if *mint.key != TOKEN_PROGRAM_ID || *token_account.key != TOKEN_PROGRAM_ID {
        return Err(ProgramError::IllegalOwner);
    }
    let send_account = Account::unpack(&token_account.data.borrow())?;
    if send_account.mint != *mint.key || send_account.owner != *player.key {
        return Err(ProgramError::InvalidArgument);
    }
    if send_account.amount < stake_amount {
        return Err(ProgramError::InsufficientFunds);
    }

    // if escrow account does not exist, create it
    if escrow.data_is_empty() {
        let rent_amount = Rent::get()?.minimum_balance(Account::LEN);
        let (authority, _) =
            Pubkey::find_program_address(&["authority".as_bytes().as_ref()], program_id);
        invoke_signed(
            &system_instruction::create_account(
                player.key,
                escrow.key,
                rent_amount,
                Account::LEN.try_into().unwrap(),
                &TOKEN_PROGRAM_ID,
            ),
            &[player.clone(), escrow.clone()],
            &[&["escrow".as_bytes().as_ref(), mint.key.as_ref(), &[bump]]],
        )?;
        invoke(
            &instruction::initialize_account3(&TOKEN_PROGRAM_ID, escrow.key, mint.key, &authority)?,
            &[escrow.clone(), mint.clone()],
        )?;
    }

    // transfer the stake tokens
    invoke(
        &instruction::transfer(
            &TOKEN_PROGRAM_ID,
            token_account.key,
            escrow.key,
            player.key,
            &[],
            stake_amount,
        )?,
        &[token_account.clone(), escrow.clone(), player.clone()],
    )?;

    // create and initialize the game account
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
    game.stake_mint = *mint.key;
    game.stake_amount = stake_amount;
    game.is_initialized = true;
    game.serialize(&mut &mut game_account.data.borrow_mut()[..])
        .unwrap();

    Ok(())
}

fn accept_game(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let player_two = next_account_info(account_info_iter)?;
    let game_account = next_account_info(account_info_iter)?;
    let escrow = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    // account validation
    if !player_two.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if game_account.owner != program_id || *token_account.owner != TOKEN_PROGRAM_ID {
        return Err(ProgramError::IllegalOwner);
    }
    if *token_program.key != TOKEN_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let send_account = Account::unpack(&token_account.data.borrow())?;
    let mut game = try_from_slice_unchecked::<Game>(&game_account.data.borrow()).unwrap();
    if !game.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    if game.players[1] != *player_two.key {
        return Err(Error::UnauthorizedToAccept.into());
    }
    if game.state != GameState::Unaccepted {
        return Err(Error::AlreadyAccepted.into());
    }
    if send_account.owner != *player_two.key {
        return Err(ProgramError::InvalidArgument);
    }
    if send_account.amount < game.stake_amount {
        return Err(ProgramError::InsufficientFunds);
    }
    let (escrow_key, _) = Pubkey::find_program_address(
        &["escrow".as_bytes().as_ref(), game.stake_mint.as_ref()],
        program_id,
    );
    if *escrow.key != escrow_key {
        return Err(ProgramError::InvalidArgument);
    }

    // transfer stake tokens
    invoke(
        &instruction::transfer(
            &TOKEN_PROGRAM_ID,
            token_account.key,
            &escrow_key,
            player_two.key,
            &[],
            game.stake_amount,
        )?,
        &[token_account.clone(), escrow.clone(), player_two.clone()],
    )?;

    // update and save the game account
    game.state = GameState::Ongoing;
    game.serialize(&mut &mut game_account.data.borrow_mut()[..])
        .unwrap();

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
