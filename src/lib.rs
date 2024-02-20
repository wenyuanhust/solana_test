use solana_program::entrypoint;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{borrow::BorrowMut, collections::HashMap, sync::Mutex};

// struct UserBalance {
//     addr: Pubkey,
//     balance: u64,
// }

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
struct TokenType {
    symbol: String,
    // address: Pubkey,
}

// #[derive(DerefMut)]
struct ContractState {
    all_token_balances: HashMap<TokenType, HashMap<Pubkey, u64>>,
}

// Define the instructions that the contract can accept
#[derive(Serialize, Deserialize, Debug)]
enum ContractInstruction {
    AdminAddSupportedToken {
        token: TokenType,
    },
    AdminDeleteSupportedToken {
        token: TokenType,
    },
    UserDeposit {
        token: TokenType,
        user: Pubkey,
        amount: u64,
    },
    UserWithdraw {
        token: TokenType,
        user: Pubkey,
        amount: u64,
    },
}

const ADMIN_PUBKEY: &str = "D6gQXdUX7AwrGtdQaCuZ5p1MwyXHaidWvKypdKY9bmkA";
lazy_static! {
    static ref CONTRACT_STATE: Mutex<ContractState> = Mutex::new(ContractState {
        all_token_balances: HashMap::new(),
    });
}

// declare and export the program's entrypoint
entrypoint!(process_instruction);

// program entrypoint's implementation
pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = deserialize_instruction(instruction_data)?;
    let mut lock = CONTRACT_STATE.lock().unwrap(); // Acquire the lock with mutability
    let all_token_balances = lock.all_token_balances.borrow_mut();

    match instruction {
        ContractInstruction::AdminAddSupportedToken { token } => {
            // let user = HashMap::from([(Pubkey::default(), 100 as u64)]);
            // all_token_balances.insert(TokenType{ symbol: "sol".to_string(), address: Pubkey::default() }, user );
            let admin_pubkey = Pubkey::from_str(ADMIN_PUBKEY).unwrap();
            let mock_sig: [u8; 65] = [0u8; 65];
            if !verify_signature(admin_pubkey, mock_sig.as_slice()) {
                return Err(ProgramError::MissingRequiredSignature);
            }
            if all_token_balances.contains_key(&token) {
                // Add already added token
                return Err(ProgramError::Custom(0));
            }
            let user = HashMap::new();
            all_token_balances.insert(token, user);
        }
        ContractInstruction::AdminDeleteSupportedToken { token } => {
            let admin_pubkey = Pubkey::from_str(ADMIN_PUBKEY).unwrap();
            if !verify_signature(admin_pubkey, instruction_data) {
                return Err(ProgramError::MissingRequiredSignature);
            }
            // delete non-exist Token
            if !all_token_balances.contains_key(&token) {
                return Err(ProgramError::Custom(1));
            }
            all_token_balances.remove(&token);
        }
        ContractInstruction::UserDeposit {
            token,
            user,
            amount,
        } => {
            let mock_sig: [u8; 65] = [0u8; 65];
            if !verify_signature(user, mock_sig.as_slice()) {
                return Err(ProgramError::MissingRequiredSignature);
            }

            // Token not added
            if !all_token_balances.contains_key(&token) {
                return Err(ProgramError::Custom(2));
            }

            let current_token_balances = all_token_balances.get_mut(&token).unwrap();
            *current_token_balances.entry(user).or_insert(0) += amount;
        }
        ContractInstruction::UserWithdraw {
            token,
            user,
            amount,
        } => {
            let mock_sig: [u8; 65] = [0u8; 65];
            if !verify_signature(user, mock_sig.as_slice()) {
                return Err(ProgramError::MissingRequiredSignature);
            }

            // Token not added
            if !all_token_balances.contains_key(&token) {
                return Err(ProgramError::Custom(3));
            }

            let current_token_balances = all_token_balances.get_mut(&token).unwrap();
            let balance = current_token_balances.entry(user).or_insert(0);
            if *balance < amount {
                return Err(ProgramError::InsufficientFunds);
            }
            *balance -= amount;
        }
    }

    // gracefully exit the program
    Ok(())
}

fn deserialize_instruction(data: &[u8]) -> Result<ContractInstruction, ProgramError> {
    // TODO: Implement the logic for deserializing an instruction
    // unimplemented!()
    let instruction: ContractInstruction = serde_json::from_slice(data).unwrap();
    println!("instruction: {:?}", instruction);
    Ok(instruction)
}

fn verify_signature(_pubkey: Pubkey, _sig: &[u8]) -> bool {
    return true;
}

#[cfg(test)]
mod test {
    use crate::ContractInstruction::AdminAddSupportedToken;
    use crate::{process_instruction, TokenType, CONTRACT_STATE};
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_add() {
        let program_id = Pubkey::default();
        let instruction_data = AdminAddSupportedToken {
            token: TokenType {
                symbol: "sol".to_string(),
            },
        };
        let instruction_data: Vec<u8> = serde_json::to_vec(&instruction_data).unwrap();
        let accounts = vec![];

        let result = process_instruction(&program_id, &accounts, &instruction_data);
        assert_eq!(result, Ok(()));

        let mut lock = CONTRACT_STATE.lock().unwrap(); // Acquire the lock with mutability
        let all_token_balances = &lock.all_token_balances;

    }
}
