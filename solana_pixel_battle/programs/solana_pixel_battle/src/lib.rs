use anchor_lang::prelude::*;
use anchor_spl::token;

declare_id!("GnuasBR5263pzfmCkP8GzmxDmDDc1JrhDW7Wg828DxPC");

#[program]
pub mod solana_pixel_battle {
    use super::*;

    // Initialize the Pixel Battle
    pub fn initialize(ctx: Context<Initialize>, canvas_size: u16, charity_count: u8) -> Result<()> {
        let battle = &mut ctx.accounts.battle;
        battle.canvas_size = canvas_size;
        battle.charity_count = charity_count;
        battle.total_pixels = 0;
        battle.total_funds = 0;
        Ok(())
    }

    // Purchase a pixel and vote for a charity
    pub fn purchase_pixel(ctx: Context<PurchasePixel>, x: u16, y: u16, charity_id: u8) -> Result<()> {
        let battle = &mut ctx.accounts.battle;
        let user = &mut ctx.accounts.user;

        // Ensure the pixel is within the canvas
        require!(x < battle.canvas_size && y < battle.canvas_size, ErrorCode::InvalidPixel);
        require!(charity_id < battle.charity_count, ErrorCode::InvalidCharity);

        let amount: u64 = 100; // 100 lamports

        // Use temporary variables to avoid borrowing conflicts
        let battle_info = battle.to_account_info();
        let user_info = user.to_account_info();

        **user_info.try_borrow_mut_lamports()? -= amount;
        **battle_info.try_borrow_mut_lamports()? += amount;

        // Update struct fields after lamport transfers
        battle.total_pixels += 1;
        battle.total_funds += amount;
        user.votes += 1;
        user.charity_votes[charity_id as usize] += 1;

        // Emit an event for the pixel purchase
        emit!(PixelPurchased {
            user: user.key(),
            x,
            y,
            charity_id,
        });

        Ok(())
    }

    // Distribute funds to the winning charity
    pub fn distribute_funds(ctx: Context<DistributeFunds>, charity_id: u8) -> Result<()> {
        let battle = &mut ctx.accounts.battle;
        let charity_wallet = &ctx.accounts.charity_wallet;

        require!(charity_id < battle.charity_count, ErrorCode::InvalidCharity);

        let total_funds = battle.total_funds;

        let battle_info = battle.to_account_info();
        let charity_info = charity_wallet.to_account_info();

        **battle_info.try_borrow_mut_lamports()? -= total_funds;
        **charity_info.try_borrow_mut_lamports()? += total_funds;

        // Reset funds after transfer
        battle.total_funds = 0;

        // Emit an event for fund distribution
        emit!(FundsDistributed {
            charity_id,
            amount: total_funds,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + std::mem::size_of::<PixelBattle>())]
    pub battle: Account<'info, PixelBattle>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PurchasePixel<'info> {
    #[account(mut)]
    pub battle: Account<'info, PixelBattle>,
    #[account(mut)]
    pub user: Account<'info, User>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DistributeFunds<'info> {
    #[account(mut)]
    pub battle: Account<'info, PixelBattle>,
    /// CHECK: This is a valid recipient account for fund transfers.
    #[account(mut)]
    pub charity_wallet: AccountInfo<'info>, 
    pub system_program: Program<'info, System>,
}

#[account]
pub struct PixelBattle {
    pub canvas_size: u16,
    pub charity_count: u8,
    pub total_pixels: u64,
    pub total_funds: u64,
}

#[account]
pub struct User {
    pub votes: u64,
    pub charity_votes: Vec<u64>, // Tracks votes per charity
}

#[event]
pub struct PixelPurchased {
    pub user: Pubkey,
    pub x: u16,
    pub y: u16,
    pub charity_id: u8,
}

#[event]
pub struct FundsDistributed {
    pub charity_id: u8,
    pub amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid pixel coordinates")]
    InvalidPixel,
    #[msg("Invalid charity ID")]
    InvalidCharity,
}
