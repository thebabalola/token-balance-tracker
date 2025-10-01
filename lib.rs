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
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let contract = TokenBalance::new();
            assert_eq!(contract.total_supply(), 0);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 0);
        }

        #[ink::test]
        fn mint_works() {
            let mut contract = TokenBalance::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let alice = accounts.alice;

            // Mint tokens to alice
            assert_eq!(contract.mint(alice, 100), Ok(()));
            assert_eq!(contract.balance_of(alice), 100);
            assert_eq!(contract.total_supply(), 100);
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = TokenBalance::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let alice = accounts.alice;
            let bob = accounts.bob;

            // Mint tokens to alice
            assert_eq!(contract.mint(alice, 100), Ok(()));

            // Set alice as caller and transfer to bob
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice);
            assert_eq!(contract.transfer(bob, 50), Ok(()));
            assert_eq!(contract.balance_of(alice), 50);
            assert_eq!(contract.balance_of(bob), 50);
        }

        #[ink::test]
        fn transfer_insufficient_balance() {
            let mut contract = TokenBalance::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let alice = accounts.alice;
            let bob = accounts.bob;

            // Set alice as caller and try to transfer without balance
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice);
            assert_eq!(contract.transfer(bob, 50), Err(Error::InsufficientBalance));
        }

        #[ink::test]
        fn only_owner_can_mint() {
            let mut contract = TokenBalance::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let alice = accounts.alice;
            let bob = accounts.bob;

            // Set bob as caller and try to mint (should fail)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(bob);
            assert_eq!(contract.mint(alice, 100), Err(Error::NotOwner));
        }
    }
}