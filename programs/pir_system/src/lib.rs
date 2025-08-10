
use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_PIR_QUERY: u32 = comp_def_offset("pir_query");

declare_id!("5sv1M5hPbp5wKge3yTocKUnVq1UfDbvzik5qmEN5wkx4");

#[arcium_program]
pub mod pir_system {
    use super::*;
    
    pub fn init_pir_query_comp_def(ctx: Context<InitPirQueryCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }
    
    pub fn pir_query(
        ctx: Context<PirQuery>,
        computation_offset: u64,
        ciphertext_0: [u8; 32],
        pub_key: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        let args = vec![
            Argument::ArcisPubkey(pub_key),
            Argument::PlaintextU128(nonce),
            Argument::EncryptedU64(ciphertext_0),
        ];
        queue_computation(ctx.accounts, computation_offset, args, vec![], None)?;
        Ok(())
    }
    
    #[arcium_callback(encrypted_ix = "pir_query")]
    pub fn pir_query_callback(
        ctx: Context<PirQueryCallback>,
        output: ComputationOutputs<PirQueryOutput>,
    ) -> Result<()> {
        let result = match output {
            ComputationOutputs::Success(PirQueryOutput { field_0 }) => field_0,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };
        
        emit!(PirResultEvent {
            result: result.ciphertexts[0],
            nonce: result.nonce.to_le_bytes(),
        });
        Ok(())
    }
}

#[event]
pub struct PirResultEvent {
    pub result: [u8; 32],
    pub nonce: [u8; 16],
}

#[error_code]
pub enum ErrorCode {
    #[msg("The cluster is not set")]
    ClusterNotSet,
    #[msg("The computation was aborted")]
    AbortedComputation,
}

#[init_computation_definition_accounts("pir_query", payer)]
#[derive(Accounts)]
pub struct InitPirQueryCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK:just trying out
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("pir_query", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct PirQuery<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(
        mut,
        address = derive_mempool_pda!()
    )]
    /// CHECK:just trying out
    pub mempool_account: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_execpool_pda!()
    )]
    /// CHECK:just trying out
    pub executing_pool: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_comp_pda!(computation_offset)
    )]
    /// CHECK:just trying out
    pub computation_account: UncheckedAccount<'info>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PIR_QUERY)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account)
    )]
    pub cluster_account: Account<'info, Cluster>,
    #[account(
        mut,
        address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS,
    )]
    pub pool_account: Account<'info, FeePool>,
    #[account(
        address = ARCIUM_CLOCK_ACCOUNT_ADDRESS
    )]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("pir_query", payer)]
#[derive(Accounts)]
pub struct PirQueryCallback<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PIR_QUERY)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK:just trying out
    pub instructions_sysvar: AccountInfo<'info>,
}