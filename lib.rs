#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod token_balance {
    use ink::storage::Mapping;

    /// Custom error types for the token contract
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Insufficient balance for the operation
        InsufficientBalance,
        /// Transfer to the same account is not allowed
        TransferToSelf,
        /// Only the owner can perform this operation
        NotOwner,
        /// Invalid amount (zero or overflow)
        InvalidAmount,
    }

    /// Result type for the contract operations
    pub type Result<T> = core::result::Result<T, Error>;

    /// Event emitted when tokens are minted
    #[ink(event)]
    pub struct TokensMinted {
        #[ink(topic)]
        pub to: AccountId,
        #[ink(topic)]
        pub amount: u128,
    }

    /// Event emitted when tokens are transferred
    #[ink(event)]
    pub struct TokensTransferred {
        #[ink(topic)]
        pub from: AccountId,
        #[ink(topic)]
        pub to: AccountId,
        #[ink(topic)]
        pub amount: u128,
    }

    /// The token balance contract
    #[ink(storage)]
    pub struct TokenBalance {
        /// Mapping from account to their token balance
        balances: Mapping<AccountId, u128>,
        /// Total supply of tokens
        total_supply: u128,
        /// Owner of the contract (can mint tokens)
        owner: AccountId,
        //--- ASSIGNMENT --- Added storage for assignment requirements ---//
        /// Allowances mapping (owner, spender) -> amount
        allowances: Mapping<(AccountId, AccountId), u128>,
        /// Pause state
        paused: bool,
        /// Blacklisted addresses
        blacklisted: Mapping<AccountId, bool>,
    }

    impl Default for TokenBalance {
        fn default() -> Self {
            Self::new()
        }
    }

    impl TokenBalance {
        /// Creates a new token contract
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            Self {
                balances: Mapping::new(),
                total_supply: 0,
                owner: caller,
                //--- ASSIGNMENT --- Initialize new fields ---//
                allowances: Mapping::new(),
                paused: false,
                blacklisted: Mapping::new(),
            }
        }

        /// Mint new tokens to an account (only owner can do this)
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, amount: u128) -> Result<()> {
            // Check if caller is the owner
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }

            // Check for valid amount
            if amount == 0 {
                return Err(Error::InvalidAmount);
            }

            // Check for overflow
            let current_balance = self.balances.get(to).unwrap_or(0);
            let new_balance = current_balance.checked_add(amount)
                .ok_or(Error::InvalidAmount)?;

            // Update balances and total supply
            self.balances.insert(to, &new_balance);
            self.total_supply = self.total_supply.checked_add(amount)
                .ok_or(Error::InvalidAmount)?;

            // Emit event
            self.env().emit_event(TokensMinted { to, amount });

            Ok(())
        }

        /// Get the balance of an account
        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u128 {
            self.balances.get(account).unwrap_or(0)
        }

        /// Get the total supply of tokens
        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }

        /// Get the owner of the contract
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Transfer tokens from caller to another account
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: u128) -> Result<()> {
            let caller = self.env().caller();

            //--- ASSIGNMENT --- Check pause state and blacklist ---//
            if self.paused {
                return Err(Error::InvalidAmount); // Using InvalidAmount as pause error
            }

            if self.blacklisted.get(caller).unwrap_or(false) || self.blacklisted.get(to).unwrap_or(false) {
                return Err(Error::InvalidAmount); // Using InvalidAmount as blacklist error
            }

            // Check if transferring to self
            if caller == to {
                return Err(Error::TransferToSelf);
            }

            // Check for valid amount
            if amount == 0 {
                return Err(Error::InvalidAmount);
            }

            // Get current balances
            let caller_balance = self.balances.get(caller).unwrap_or(0);
            let to_balance = self.balances.get(to).unwrap_or(0);

            // Check if caller has sufficient balance
            if caller_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            // Calculate new balances
            let new_caller_balance = caller_balance.saturating_sub(amount);
            let new_to_balance = to_balance.checked_add(amount)
                .ok_or(Error::InvalidAmount)?;

            // Update balances
            self.balances.insert(caller, &new_caller_balance);
            self.balances.insert(to, &new_to_balance);

            // Emit event
            self.env().emit_event(TokensTransferred {
                from: caller,
                to,
                amount,
            });

            Ok(())
        }

        /// Get the caller's own balance
        #[ink(message)]
        pub fn my_balance(&self) -> u128 {
            self.balance_of(self.env().caller())
        }

        //--- ASSIGNMENT --- Added functionalities for assignment requirements ---//

        /// Burn tokens from caller's account
        #[ink(message)]
        pub fn burn(&mut self, amount: u128) -> Result<()> {
            let caller = self.env().caller();
            let caller_balance = self.balances.get(caller).unwrap_or(0);

            if amount == 0 {
                return Err(Error::InvalidAmount);
            }

            if caller_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            let new_balance = caller_balance.saturating_sub(amount);
            self.balances.insert(caller, &new_balance);
            self.total_supply = self.total_supply.saturating_sub(amount);

            Ok(())
        }

        /// Check allowance for spender
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.allowances.get((owner, spender)).unwrap_or(0)
        }

        /// Approve spender to spend tokens
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, amount: u128) -> Result<()> {
            let caller = self.env().caller();
            self.allowances.insert((caller, spender), &amount);
            Ok(())
        }

        /// Transfer tokens using allowance
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, amount: u128) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowances.get((from, caller)).unwrap_or(0);

            if amount == 0 {
                return Err(Error::InvalidAmount);
            }

            if allowance < amount {
                return Err(Error::InsufficientBalance);
            }

            let from_balance = self.balances.get(from).unwrap_or(0);
            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            let new_from_balance = from_balance.saturating_sub(amount);
            let new_to_balance = self.balances.get(to).unwrap_or(0).checked_add(amount)
                .ok_or(Error::InvalidAmount)?;
            let new_allowance = allowance.saturating_sub(amount);

            self.balances.insert(from, &new_from_balance);
            self.balances.insert(to, &new_to_balance);
            self.allowances.insert((from, caller), &new_allowance);

            Ok(())
        }

        /// Pause all transfers (owner only)
        #[ink(message)]
        pub fn pause(&mut self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            self.paused = true;
            Ok(())
        }

        /// Unpause all transfers (owner only)
        #[ink(message)]
        pub fn unpause(&mut self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            self.paused = false;
            Ok(())
        }

        /// Check if contract is paused
        #[ink(message)]
        pub fn is_paused(&self) -> bool {
            self.paused
        }

        /// Blacklist an address (owner only)
        #[ink(message)]
        pub fn blacklist(&mut self, account: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            self.blacklisted.insert(account, &true);
            Ok(())
        }

        /// Remove from blacklist (owner only)
        #[ink(message)]
        pub fn unblacklist(&mut self, account: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            self.blacklisted.insert(account, &false);
            Ok(())
        }

        /// Check if address is blacklisted
        #[ink(message)]
        pub fn is_blacklisted(&self, account: AccountId) -> bool {
            self.blacklisted.get(account).unwrap_or(false)
        }

        /// Batch transfer to multiple addresses
        #[ink(message)]
        pub fn batch_transfer(&mut self, recipients: Vec<(AccountId, u128)>) -> Result<()> {
            let caller = self.env().caller();
            let caller_balance = self.balances.get(caller).unwrap_or(0);

            // Check if caller has enough balance for all transfers
            let total_amount: u128 = recipients.iter().map(|(_, amount)| amount).sum();
            if caller_balance < total_amount {
                return Err(Error::InsufficientBalance);
            }

            // Check for zero amounts
            for (_, amount) in &recipients {
                if *amount == 0 {
                    return Err(Error::InvalidAmount);
                }
            }

            // Perform all transfers
            for (to, amount) in recipients {
                if caller == to {
                    return Err(Error::TransferToSelf);
                }

                let to_balance = self.balances.get(to).unwrap_or(0);
                let new_to_balance = to_balance.checked_add(amount)
                    .ok_or(Error::InvalidAmount)?;

                self.balances.insert(to, &new_to_balance);
            }

            // Update caller's balance
            let new_caller_balance = caller_balance.saturating_sub(total_amount);
            self.balances.insert(caller, &new_caller_balance);

            Ok(())
        }
    }
}