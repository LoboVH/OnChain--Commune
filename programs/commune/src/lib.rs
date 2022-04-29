use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/*commune program is a online community
 join_commune function allows users to be the member(approver) of the commune by transfering joining fee to commune
 create_item let the members to add items on commune for sale
 create_item_sale let the member to buy a particular item
 sales tax is collected on the course of this trade 

 using add_proposal functon different proposals of goods and services can be added for the development of the community
 vote_for-proposal function let members to vote for a particular proposal
 approve_proposal function will transfer the amount from the commune's balance to the owner of the proposal which is voted yes('vote_yes').
*/
#[program]
pub mod commune {
    use super::*;
    use anchor_lang::solana_program::native_token::{ LAMPORTS_PER_SOL };
    pub fn initialize_market(ctx: Context<InitializeMarket>, commune_bump: u8) -> ProgramResult {
        let commune = &mut ctx.accounts.commune;
        commune.bump = commune_bump;
        commune.item_count = 0;
        commune.fee = (0.01 * LAMPORTS_PER_SOL as f64) as u64; //joining fee
        commune.tax = 3; //sales tax percentage
        commune.total_proposal_count = 0;
        Ok(())
    }

    pub fn join_commune(ctx: Context<JoinCommune>, approver_account_bump: u8) -> ProgramResult {
        let commune = &mut ctx.accounts.commune;
        let fee = commune.fee;

        let member = &ctx.accounts.member;

        //transfers joining fee from user to commune account
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            member.to_account_info().key,
            commune.to_account_info().key,
            fee,
            );
            anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                member.to_account_info(),
                commune.to_account_info(),
                ctx.accounts.system_program.to_account_info(),

            ],
        )?;
        

        let approver = &mut ctx.accounts.approver;
        approver.bump = approver_account_bump;
        approver.approval = true; //approved as a member

        Ok(())

    }

    //add item 
    pub fn create_item(ctx: Context<CreateItem>, item_account_bump: u8, item_id: u64, title: String, price: u64, description: String) -> ProgramResult {
        
        let commune = &mut ctx.accounts.commune;

        let item = &mut ctx.accounts.item;

        let address = &ctx.accounts.seller.key();

        let tax_percent = commune.tax;
        item.tax = (((price * tax_percent * LAMPORTS_PER_SOL)/100) as f64 ) as u64; //tax amount calculation

        let approver = &mut ctx.accounts.approver;
        
        
            if approver.approval == true {

                if title.chars().count() > 80 {
                return Err(ErrorCode::TitleIsTooLong.into());
                }  

                if description.chars().count() > 1024 {
                    return Err(ErrorCode::DescriptionIsTooLong.into());
                }

                //Transfer sales tax to commune
                let ix = anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.seller.to_account_info().key,
                &commune.to_account_info().key,
                item.tax,
                );
                anchor_lang::solana_program::program::invoke(
                &ix,
                &[
                    ctx.accounts.seller.to_account_info(),
                    commune.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),

                ],
                )?;

                //updated price with sales tax
                let market_price = price * LAMPORTS_PER_SOL + item.tax ;

                item.id = item_id;
                item.seller = *address;
                item.buyer = Pubkey::default();
                item.title = title;
                item.description = description;
                item.price = market_price;
                item.bump = item_account_bump;

                commune.item_count += 1;       


            }else {
                return Err(ErrorCode::InvalidAddress.into());
            }
        Ok(())
    }   

    //buy item
    pub fn create_market_sale(ctx: Context<CreateMarketSale>, item_id: u64) -> ProgramResult {
        let commune = &mut ctx.accounts.commune;

        let address = &ctx.accounts.buyer.key();
        
        let item = &mut ctx.accounts.item;
        let price = item.price;

        let approver = &mut ctx.accounts.approver;

            if approver.approval == true {
                if item.sold == false {
                    if &item.seller == &ctx.accounts.to.key() {
                    let ix = anchor_lang::solana_program::system_instruction::transfer(
                        address,
                        &item.seller,
                        price,
                    );

                    anchor_lang::solana_program::program::invoke(
                    &ix,
                    &[
                        ctx.accounts.buyer.to_account_info(),
                        ctx.accounts.to.to_account_info(),

                    ],
                    )?;
                    
                        item.id = item_id;
                        item.buyer = *address;
                        item.sold = true;
                        commune.item_count -= 1;


                    } else {
                        return Err(ErrorCode::WrongSeller.into());
                    }

                } else {
                    return Err(ErrorCode::ItemSold.into());
                }
                
            } else {
                return Err(ErrorCode::InvalidAddress.into());
            }

            Ok(())
    }

    pub fn add_proposal(
        ctx: Context<AddProposal>,
        proposal_account_bump: u8,
        proposal_id: u64,
        title: String,
        description: String,
        price: u64,
        end_time_stamp: u128,
    ) -> ProgramResult {
        let commune = &mut ctx.accounts.commune;

        let proposal = &mut ctx.accounts.proposal;

        let user = &mut ctx.accounts.user;

        let approver = &mut ctx.accounts.approver;

            if approver.approval == true {

                if title.chars().count() > 80 {
                return Err(ErrorCode::TitleIsTooLong.into());
                }  

                if description.chars().count() > 1024 {
                    return Err(ErrorCode::DescriptionIsTooLong.into());
                }

                proposal.id = proposal_id;
                proposal.owner = *user.to_account_info().key;
                proposal.title = title;
                proposal.description = description;
                proposal.price = price;
                proposal.vote_yes =  0;
                proposal.vote_no = 0;
                proposal.created_at = Clock::get()?.unix_timestamp;
                proposal.end_time_stamp = end_time_stamp;
                proposal.bump = proposal_account_bump;

                // increment total proposal count
                commune.total_proposal_count += 1;

            } else {
                return Err(ErrorCode::InvalidAddress.into());
            }

        
        Ok(())
    
}

// vote on a proposal
    pub fn vote_for_proposal(
        ctx: Context<VoteForProposal>,
        vote_account_bump: u8,
        proposal_id: u64,
        vote: bool,
    ) -> ProgramResult {
        let proposal = &mut ctx.accounts.proposal;
        let vote_account = &mut ctx.accounts.vote;
        let user = &mut ctx.accounts.user;

        let approver = &mut ctx.accounts.approver;

        if approver.approval == true {
            vote_account.proposal_id = proposal_id;
            vote_account.voter = *user.to_account_info().key;
            vote_account.vote = vote;
            vote_account.created_at =  Clock::get()?.unix_timestamp;
            vote_account.bump =  vote_account_bump;

            if (Clock::get()?.unix_timestamp as u128) > proposal.end_time_stamp {
                // return error if proposal has ended
                return Err(ErrorCode::ProposalHasEnded.into());
            }

            // corespoing vote count base on `vote`
            if vote {
                proposal.vote_yes += 1
            } else {
                proposal.vote_no += 1
            }

        } else {
                return Err(ErrorCode::InvalidAddress.into());
        }

        Ok(())
    }

    pub fn approve_proposal(
        ctx: Context<ApproveProposal>,
        proposal_id: u64,
    ) -> ProgramResult {
        let proposal = &mut ctx.accounts.proposal;
        let commune = &mut ctx.accounts.commune;
        if proposal.owner == ctx.accounts.to.key() {
            if (Clock::get()?.unix_timestamp as u128) > proposal.end_time_stamp {
            if proposal.vote_yes > proposal.vote_no {

                if proposal.approved == true {
                    return Err(ErrorCode::Approved.into());
                }
                let ix = anchor_lang::solana_program::system_instruction::transfer(
                    &commune.to_account_info().key,
                    &proposal.owner,
                    proposal.price,
                    );
                    anchor_lang::solana_program::program::invoke(
                    &ix,
                    &[
                        commune.to_account_info(),
                        ctx.accounts.to.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),

                    ],
                    )?;

                    proposal.id = proposal_id;
                    proposal.approved = true;
            } else {
            return Err(ErrorCode::Rejected.into());
            }

        } else {
            return Err(ErrorCode::Voting.into());
        }

        } else {
                return Err(ErrorCode::InvalidAddress.into());
        }

        
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(commune_bump: u8)]
pub struct InitializeMarket<'info> {
    #[account(init, payer = user, space=9000, seeds = [b"commune".as_ref()], bump = commune_bump)]
    pub commune: Account<'info, Commune>,  
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(approver_account_bump: u8)]
pub struct JoinCommune<'info> {
    #[account(mut)]
    pub commune:Account<'info, Commune>,
    #[account(init, payer = member, space  = Approver::LEN, seeds = [b"approver_account".as_ref(), member.key().as_ref()], bump = approver_account_bump)]
    pub approver: Account<'info, Approver>,
    #[account(mut)]
    pub member: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(item_account_bump: u8, item_id: u64)]
pub struct CreateItem<'info> {
    #[account(mut)]
    pub commune: Account<'info, Commune>,

    #[account(init, seeds = [b"item_account".as_ref(), item_id.to_le_bytes().as_ref()], bump = item_account_bump, payer =seller, space = Item::LEN)]
    pub item: Account<'info, Item>,

    #[account(mut, seeds = [b"approver_account".as_ref(), seller.key().as_ref()], bump = approver.bump)]
    pub approver: Account<'info, Approver>,

    #[account(mut)]
    pub seller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(item_id: u64)]
pub struct CreateMarketSale<'info> {
    #[account(mut)]
    pub commune: Account<'info, Commune>,
    #[account(mut, seeds = [b"item_account".as_ref(), item_id.to_le_bytes().as_ref()], bump = item.bump)]
    pub item: Account<'info, Item>,

    #[account(mut, seeds = [b"approver_account".as_ref(), buyer.key().as_ref()], bump = approver.bump)]
    pub approver: Account<'info, Approver>,

    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub to: AccountInfo<'info>, 
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_account_bump: u8, proposal_id: u64)]  
pub struct AddProposal<'info> {
    #[account(mut)]
    pub commune: Account<'info, Commune>,

    #[account(init, seeds = [b"proposal_account".as_ref(), proposal_id.to_le_bytes().as_ref()], bump =  proposal_account_bump, payer = user, space = Proposal::LEN)]
    pub proposal: Account<'info, Proposal>,

    #[account(mut, seeds = [b"approver_account".as_ref(), user.key().as_ref()], bump = approver.bump)]
    pub approver: Account<'info, Approver>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(vote_account_bump: u8, proposal_id: u64)]
pub struct VoteForProposal<'info> {
    #[account(mut, seeds = [b"proposal_account".as_ref(), proposal_id.to_le_bytes().as_ref()], bump = proposal.bump)]
    pub proposal: Account<'info, Proposal>,

    #[account(init, seeds = [b"vote_account".as_ref(), proposal_id.to_le_bytes().as_ref(), user.key.as_ref()] , bump = vote_account_bump, payer = user, space = Vote::LEN)]
    pub vote: Account<'info, Vote>,

    #[account(mut, seeds = [b"approver_account".as_ref(), user.key().as_ref()], bump = approver.bump)]
    pub approver: Account<'info, Approver>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction( proposal_id: u64)]
pub struct ApproveProposal<'info> {
    #[account(mut, seeds = [b"proposal_account".as_ref(), proposal_id.to_le_bytes().as_ref()], bump = proposal.bump)]
    pub proposal: Account<'info, Proposal>,

    #[account(mut, seeds = [b"commune".as_ref()], bump = commune.bump)]
    pub commune: Account<'info, Commune>,

    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub system_program: Program<'info, System>,

}




#[account]
pub struct Commune {
    pub fee: u64, 
    pub bump: u8,
    pub tax: u64,
    pub item_count: u64,
    pub total_proposal_count: u64,

}

#[account]
pub struct Approver {
    pub approval: bool,
    pub bump: u8,
}

#[account]
pub struct Item {
    pub id: u64,
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub title: String,
    pub description: String,
    pub price: u64,
    pub tax: u64,
    pub sold: bool,
    pub bump: u8,
}

#[account]
pub struct Proposal {
    pub id: u64, // unique id for each proposal
    pub owner: Pubkey,
    pub created_at: i64,
    pub title: String,
    pub description: String,
    pub price: u64,
    pub vote_yes: u64,
    pub vote_no: u64,
    pub bump: u8,
    pub end_time_stamp: u128, 
    pub approved: bool,
}

#[account]
pub struct Vote {
    pub proposal_id: u64,
    pub vote: bool,
    pub voter: Pubkey,
    pub created_at: i64,
    pub bump: u8,
}

#[error]
pub enum ErrorCode {
    #[msg("Tried sending to the wrong seller!")] WrongSeller,
    #[msg("Price transfer failed...!!!")] SomethingWrong,
    #[msg("Not a commune member")] InvalidAddress,
    #[msg("Title is too long. maximum: 80 character")]
    TitleIsTooLong,
    #[msg("Description is too long. maximum: 1024 character")]
    DescriptionIsTooLong,
    #[msg("Item is already sold")] ItemSold,
    #[msg("This proposal is rejected")] Rejected,
    #[msg("Voting time remaining")] Voting,
    #[msg("Time is up")] ProposalHasEnded,
    #[msg("proposal is already approved")] Approved,
}

const U64_LEN: usize = 8;
const DISCRIMINATOR_LENGTH: usize = 8;
const BUMP_LENGTH: usize = 1;
const BOOL_LENGTH: usize = 1;
const PUBKEY_LENGTH: usize = 32;
const STRING_LENGTH_PREFIX: usize = 4;
const TIMESTAMP_LENGTH: usize = 8;
const MAX_TITLE_LENGTH: usize = 80 * STRING_LENGTH_PREFIX;
const MAX_DESCRIPTION_LENGTH: usize = 1024 * STRING_LENGTH_PREFIX;
const END_TIME_STAMP_LENGTH: usize = 16;

impl Approver {
    const LEN: usize = DISCRIMINATOR_LENGTH
    + BOOL_LENGTH //approval
    + BUMP_LENGTH;
}

impl Item {
    const LEN: usize = DISCRIMINATOR_LENGTH
    + U64_LEN //id
    + PUBKEY_LENGTH //seller
    + PUBKEY_LENGTH //buyer
    + MAX_TITLE_LENGTH //title
    + MAX_DESCRIPTION_LENGTH //description
    + U64_LEN //price
    + U64_LEN //tax
    + BOOL_LENGTH //sold
    + BUMP_LENGTH;
}

impl Proposal {
    const LEN: usize = DISCRIMINATOR_LENGTH
    + U64_LEN //id
    + PUBKEY_LENGTH //owner
    + TIMESTAMP_LENGTH //timestamp
    + MAX_TITLE_LENGTH //title
    + MAX_DESCRIPTION_LENGTH //description
    + U64_LEN //price
    + U64_LEN //vote yes
    + U64_LEN //vote no
    + BUMP_LENGTH
    + END_TIME_STAMP_LENGTH //end time stamp
    + BOOL_LENGTH;
}

impl Vote {
    const LEN: usize = DISCRIMINATOR_LENGTH
    + U64_LEN //proposal id
    + BOOL_LENGTH // vote
    + PUBKEY_LENGTH // voter
    + TIMESTAMP_LENGTH // created at
    + BUMP_LENGTH;
}