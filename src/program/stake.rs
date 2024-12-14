use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    program::{invoke, invoke_signed},
    sysvar::{rent::Rent, Sysvar},
};

use borsh::{BorshDeserialize, BorshSerialize};


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct StakeAccount {
    pub owner: Pubkey,           
    pub amount: u64,             
    pub locked_until: i64,       
    pub is_active: bool,        
}


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum StakeInstruction {
    
    CreateStake {
        amount: u64,
        lock_period: i64,
    },
    
    Withdraw {
        amount: u64,
    },
}


entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = StakeInstruction::try_from_slice(instruction_data)?;
    
    match instruction {
        StakeInstruction::CreateStake { amount, lock_period } => {
            process_create_stake(program_id, accounts, amount, lock_period)
        }
        StakeInstruction::Withdraw { amount } => {
            process_withdraw(program_id, accounts, amount)
        }
    }
}


fn process_create_stake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    lock_period: i64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
   
    let staker_account = next_account_info(account_info_iter)?;    
    let stake_account = next_account_info(account_info_iter)?;     
    let system_program = next_account_info(account_info_iter)?;   
    
    
    if amount < 10_000_000_000 {
        return Err(ProgramError::InvalidArgument);
    }

    
    let rent = Rent::get()?;
    let stake_account_data = StakeAccount {
        owner: *staker_account.key,
        amount,
        locked_until: lock_period,
        is_active: true,
    };

   
    let space = stake_account_data.try_to_vec()?.len();
    let rent_lamports = rent.minimum_balance(space);

    
    invoke(
        &system_instruction::create_account(
            staker_account.key,
            stake_account.key,
            amount + rent_lamports,
            space as u64,
            program_id,
        ),
        &[
            staker_account.clone(),
            stake_account.clone(),
            system_program.clone(),
        ],
    )?;

    
    stake_account_data.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;

    msg!("Stake account created and SOL locked successfully");
    Ok(())
}


fn process_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let staker_account = next_account_info(account_info_iter)?;
    let stake_account = next_account_info(account_info_iter)?;
    
    
    if stake_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    
    let mut stake_data = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
    
    
    if stake_data.owner != *staker_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

   
    if stake_data.locked_until > 0 {
        return Err(ProgramError::InvalidArgument);
    }

    
    if amount > stake_data.amount {
        return Err(ProgramError::InsufficientFunds);
    }

    
    **stake_account.try_borrow_mut_lamports()? -= amount;
    **staker_account.try_borrow_mut_lamports()? += amount;

    
    stake_data.amount -= amount;
    stake_data.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;

    msg!("Withdrew {} lamports from stake account", amount);
    Ok(())
}
