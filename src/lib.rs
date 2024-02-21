use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{borrow::BorrowMut, collections::HashMap, sync::Mutex};

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
struct TokenType {
    symbol: String,
    // address: Pubkey,
}

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

// admin pubkey
const ADMIN_PUBKEY: &str = "D6gQXdUX7AwrGtdQaCuZ5p1MwyXHaidWvKypdKY9bmkA";
// todo, not familiar with Solana sig verification
const MOCK_SIG: [u8; 65] = [0u8; 65];

// todo, save balance of all users of all supportted token by global variable, need to know Solana contract's way of storing contract data
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
    let mut lock = CONTRACT_STATE.lock().unwrap();
    let all_token_balances = lock.all_token_balances.borrow_mut();

    match instruction {
        ContractInstruction::AdminAddSupportedToken { token } => {
            check_add_token(token, all_token_balances)?;
        }
        ContractInstruction::AdminDeleteSupportedToken { token } => {
            check_delete_token(token, all_token_balances)?;
        }
        ContractInstruction::UserDeposit {
            token,
            user,
            amount,
        } => {
            user_deposit_token(token, user, amount, all_token_balances)?;
        }
        ContractInstruction::UserWithdraw {
            token,
            user,
            amount,
        } => {
            user_withdraw_token(token, user, amount, all_token_balances)?;
        }
    }

    Ok(())
}

// use serde_json for simplicity
fn deserialize_instruction(data: &[u8]) -> Result<ContractInstruction, ProgramError> {
    let instruction: ContractInstruction = serde_json::from_slice(data).unwrap();
    println!("instruction: {:?}", instruction);
    Ok(instruction)
}

// add newly supported token
fn check_add_token(
    token: TokenType,
    all_token_balances: &mut HashMap<TokenType, HashMap<Pubkey, u64>>,
) -> Result<(), ProgramError> {
    // only admin can add token
    let admin_pubkey = Pubkey::from_str(ADMIN_PUBKEY).unwrap();
    if !verify_signature(admin_pubkey, MOCK_SIG.as_slice()) {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if all_token_balances.contains_key(&token) {
        // Add already added token
        return Err(ProgramError::Custom(0));
    }
    let user = HashMap::new();
    all_token_balances.insert(token, user);
    Ok(())
}

// delete supported token
fn check_delete_token(
    token: TokenType,
    all_token_balances: &mut HashMap<TokenType, HashMap<Pubkey, u64>>,
) -> Result<(), ProgramError> {
    // only admin can delete token
    let admin_pubkey = Pubkey::from_str(ADMIN_PUBKEY).unwrap();
    if !verify_signature(admin_pubkey, &MOCK_SIG) {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // delete non-exist Token
    if !all_token_balances.contains_key(&token) {
        return Err(ProgramError::Custom(1));
    }
    all_token_balances.remove(&token);
    Ok(())
}

// user deposit token
fn user_deposit_token(
    token: TokenType,
    user: Pubkey,
    amount: u64,
    all_token_balances: &mut HashMap<TokenType, HashMap<Pubkey, u64>>,
) -> Result<(), ProgramError> {
    if !verify_signature(user, MOCK_SIG.as_slice()) {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Token not added
    if !all_token_balances.contains_key(&token) {
        return Err(ProgramError::Custom(2));
    }

    // todo, check user has enough token to transfer and substract user's account
    // not familiar with solana's mechanism, may do this by check and modify _accounts in process_instruction's parameter list
    let current_token_balances = all_token_balances.get_mut(&token).unwrap();
    *current_token_balances.entry(user).or_insert(0) += amount;
    Ok(())
}

fn user_withdraw_token(
    token: TokenType,
    user: Pubkey,
    amount: u64,
    all_token_balances: &mut HashMap<TokenType, HashMap<Pubkey, u64>>,
) -> Result<(), ProgramError> {
    if !verify_signature(user, MOCK_SIG.as_slice()) {
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

    // todo, add amount to user's account
    // not familiar with solana's mechanism, may do this by check and modify _accounts in process_instruction's parameter list
    
    Ok(())
}

// todo, do not verify signature by far
fn verify_signature(_pubkey: Pubkey, _sig: &[u8]) -> bool {
    return true;
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::ContractInstruction::{
        AdminAddSupportedToken, AdminDeleteSupportedToken, UserDeposit, UserWithdraw,
    };
    use crate::{process_instruction, TokenType, CONTRACT_STATE};
    use solana_program::program_error::ProgramError;
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_add_delete_deposit_withdraw() {
        let program_id = Pubkey::default();
        let accounts = vec![];

        {
            // legal add
            println!("legal add");
            let instruction_data = AdminAddSupportedToken {
                token: TokenType {
                    symbol: "sol".to_string(),
                },
            };
            let instruction_data: Vec<u8> = serde_json::to_vec(&instruction_data).unwrap();

            let result = process_instruction(&program_id, &accounts, &instruction_data);
            assert_eq!(result, Ok(()));

            let lock = CONTRACT_STATE.lock().unwrap(); // Acquire the lock with mutability
            let all_token_balances = &lock.all_token_balances;
            let sol = all_token_balances.get(&TokenType {
                symbol: "sol".to_string(),
            });
            assert_eq!(sol, Some(&HashMap::new()));
        }

        {
            // duplicate add
            println!("duplicate add");
            let instruction_data = AdminAddSupportedToken {
                token: TokenType {
                    symbol: "sol".to_string(),
                },
            };
            let instruction_data: Vec<u8> = serde_json::to_vec(&instruction_data).unwrap();

            // legal add
            let result = process_instruction(&program_id, &accounts, &instruction_data);
            assert_eq!(result, Err(ProgramError::Custom(0)));
        }

        {
            // deposit sol token
            println!("user sol token deposit");
            let instruction_data = UserDeposit {
                token: TokenType {
                    symbol: "sol".to_string(),
                },
                user: Pubkey::default(),
                amount: 100,
            };
            let instruction_data: Vec<u8> = serde_json::to_vec(&instruction_data).unwrap();

            let result = process_instruction(&program_id, &accounts, &instruction_data);
            assert_eq!(result, Ok(()));

            let lock = CONTRACT_STATE.lock().unwrap(); // Acquire the lock with mutability
            let all_token_balances = &lock.all_token_balances;
            let sol = all_token_balances.get(&TokenType {
                symbol: "sol".to_string(),
            });
            assert_eq!(sol, Some(&HashMap::from([(Pubkey::default(), 100)])));
        }

        {
            // withdraw sol token
            println!("user sol token withdraw");
            let instruction_data = UserWithdraw {
                token: TokenType {
                    symbol: "sol".to_string(),
                },
                user: Pubkey::default(),
                amount: 10,
            };
            let instruction_data: Vec<u8> = serde_json::to_vec(&instruction_data).unwrap();

            let result = process_instruction(&program_id, &accounts, &instruction_data);
            assert_eq!(result, Ok(()));

            let lock = CONTRACT_STATE.lock().unwrap();
            let all_token_balances = &lock.all_token_balances;
            let sol = all_token_balances.get(&TokenType {
                symbol: "sol".to_string(),
            });
            assert_eq!(sol, Some(&HashMap::from([(Pubkey::default(), 90)])));
        }

        {
            // illegal withdraw sol token
            println!("illegal user sol token withdraw");
            let instruction_data = UserWithdraw {
                token: TokenType {
                    symbol: "sol".to_string(),
                },
                user: Pubkey::default(),
                amount: 100,
            };
            let instruction_data: Vec<u8> = serde_json::to_vec(&instruction_data).unwrap();

            let result = process_instruction(&program_id, &accounts, &instruction_data);
            assert_eq!(result, Err(ProgramError::InsufficientFunds));
        }

        {
            println!("legal delete");
            let instruction_del_data = AdminDeleteSupportedToken {
                token: TokenType {
                    symbol: "sol".to_string(),
                },
            };
            let instruction_del_data: Vec<u8> = serde_json::to_vec(&instruction_del_data).unwrap();
            let result = process_instruction(&program_id, &accounts, &instruction_del_data);
            assert_eq!(result, Ok(()));

            let lock = CONTRACT_STATE.lock().unwrap(); // Acquire the lock with mutability
            let all_token_balances = &lock.all_token_balances;
            let sol = all_token_balances.get(&TokenType {
                symbol: "sol".to_string(),
            });
            assert_eq!(sol, None);
        }

        {
            // illegal delete
            println!("illegal delete");
            let instruction_del_data = AdminDeleteSupportedToken {
                token: TokenType {
                    symbol: "sool".to_string(),
                },
            };
            let instruction_del_data: Vec<u8> = serde_json::to_vec(&instruction_del_data).unwrap();
            let result = process_instruction(&program_id, &accounts, &instruction_del_data);
            assert_eq!(result, Err(ProgramError::Custom(1)));
        }
    }
}
