use pinocchio::{
    account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::create_program_address, ProgramResult
};
use pinocchio_token::{instructions::{CloseAccount, Transfer}, state::TokenAccount};

use crate::{
    AccountCheck, AccountClose, AssociatedTokenAccount, AssociatedTokenAccountCheck, AssociatedTokenAccountInit, Escrow, MintInterface, ProgramAccount, SignerAccount
};

pub struct RefundAccounts<'a> {
    pub maker: &'a AccountInfo, //need to check this is signer              - done
    pub escrow: &'a AccountInfo, //validate the key during the process      - done
    pub mint_a: &'a AccountInfo, //validate mint                            - done
    pub vault: &'a AccountInfo, //check AssociatedToken account check       - done
    pub maker_ata_a: &'a AccountInfo, //init if needed                      - done
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

//account validation
impl<'a> TryFrom<&'a [AccountInfo]> for RefundAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, vault, maker_ata_a, system_program, token_program] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(maker)?;
        MintInterface::check(mint_a)?;
        AssociatedTokenAccount::check(vault, escrow, mint_a, token_program)?;

        Ok(Self {
            maker,
            escrow,
            mint_a,
            vault,
            maker_ata_a,
            system_program,
            token_program,
        })
    }
}

pub struct Refund<'a> {
    pub accounts: RefundAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Refund<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let accounts = RefundAccounts::try_from(accounts)?;

        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_a,
            accounts.mint_a,
            accounts.maker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self { accounts })
    }
}

impl<'a> Refund<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

    pub fn process(&mut self) -> ProgramResult {
        let data = self.accounts.escrow.try_borrow_data()?;
        let escrow = Escrow::load(&data)?;

        let escrow_key = create_program_address(
            &[
                b"escrow",
                self.accounts.maker.key(),
                &escrow.seed.to_le_bytes(),
                &escrow.bump,
            ],
            &crate::ID,
        )?;

        if &escrow_key != self.accounts.escrow.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        let amount= {
            let vault = TokenAccount::from_account_info(&self.accounts.vault)?;
            vault.amount()
        };

        let seed_binding = escrow.seed.to_le_bytes();
        let bump_binding = escrow.bump;

        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.key().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];

        let signer_seeds = Signer::from(&escrow_seeds);

        //transfer token from vault to maker ata a
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.maker_ata_a,
            authority: self.accounts.escrow,
            amount,
        }
        .invoke_signed(&[signer_seeds.clone()])?;


        //close the vault
        CloseAccount{
            account: self.accounts.vault,
            destination: self.accounts.maker,
            authority: self.accounts.escrow
        }.invoke_signed(&[signer_seeds.clone()])?;

        //close the escrow
        drop(data);
        ProgramAccount::close(self.accounts.escrow, self.accounts.maker)?;

        Ok(())
    }
}
