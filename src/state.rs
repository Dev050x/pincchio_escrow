use std::sync::Mutex;

use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

#[repr(C)] //structured bytes
pub struct Escrow {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
    pub bump: [u8; 1],
}


//helper function 
impl Escrow {
    pub const LEN: usize = size_of::<u64>()
        + size_of::<Pubkey>()
        + size_of::<Pubkey>()
        + size_of::<Pubkey>()
        + size_of::<u64>()
        + size_of::<[u8; 1]>();
    
    #[inline(always)]
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self , ProgramError>{
        if bytes.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        //re-interpretion (not exactly conversion) using pointer
        //bytes.as_mut_ptr() => *mut u8
        //convert *mut u8 to *mut Self
        //this is pointer we need actual value so derefarance it *
        //return &mut 
        Ok(unsafe {
            &mut *core::mem::transmute::<*mut u8 , *mut Self>(bytes.as_mut_ptr())
        })
    }

    #[inline(always)]
    pub fn load(bytes: & [u8]) -> Result<& Self , ProgramError>{
        if bytes.len() != Self::LEN{
            return Err(ProgramError::InvalidAccountData);
        }

        //bytes.as_ptr() => *const u8
        //.........
        Ok(unsafe {
            & *core::mem::transmute::<*const u8 , *const Self>(bytes.as_ptr())
        })
    }

    #[inline(always)]
    pub fn set_seed(&mut self , seed:u64){
        self.seed = seed;
    }

    #[inline(always)]
    pub fn set_maker(&mut self , maker:Pubkey){
        self.maker = maker;
    }

    #[inline(always)]
    pub fn set_mint_a(&mut self , mint_a:Pubkey){
        self.mint_a = mint_a;
    }

    pub fn set_mint_b(&mut self , mint_b:Pubkey){
        self.mint_b = mint_b;
    }

    pub fn set_receive(&mut self , receive: u64){
        self.receive = receive;
    }

    pub fn set_bump(&mut self , bump: [u8;1]){
        self.bump = bump;
    }

    #[inline(always)]
    pub fn set_inner(&mut self, seed: u64, maker: Pubkey, mint_a: Pubkey, mint_b: Pubkey, receive: u64, bump: [u8;1]) {
        self.seed = seed;
        self.maker = maker;
        self.mint_a = mint_a;
        self.mint_b = mint_b;
        self.receive = receive;
        self.bump = bump;
    }


}
