use map_vec::Map; 
use oasis_std::{Address, Context};

#[derive(oasis_std::Service)]
pub struct Ballot {
    description: String,
    candidates: Vec<String>,
    tally: Vec<u32>,
    accepting_votes: bool,
    admin: Address,
    voters: Map<Address, u32>,
}

type Result<T> = std::result::Result<T, String>; // define our own result type, for convenience

impl Ballot {

    pub fn new(ctx: &Context, description: String, candidates: Vec<String>) -> Result<Self> {
        Ok(Self {
            description,
            tally: vec![0; candidates.len()],
            candidates,
            accepting_votes: true,
            admin: ctx.sender(),
            voters: Map::new(),
        })
    }

    pub fn description(&self, _ctx: &Context) -> Result<&str> {
        Ok(&self.description)
    }

    pub fn candidates(&self, _ctx: &Context) -> Result<Vec<&str>> {
        Ok(self.candidates.iter().map(String::as_ref).collect())
    }
    
    pub fn vote(&mut self, ctx: &Context, candidate_num: u32) -> Result<()> {
        if !self.accepting_votes {
            return Err("Voting is closed.".to_string());
        }
        if candidate_num as usize >= self.candidates.len() {
            return Err(format!("Invalid candidate `{}`.", candidate_num));
        }
        if let Some(prev_vote) = self.voters.insert(ctx.sender(), candidate_num) {
            self.tally[prev_vote as usize] -= 1;
        }
        self.tally[candidate_num as usize] =
            self.tally[candidate_num as usize].checked_add(1).unwrap();
        Ok(())
    }
    
    pub fn close(&mut self, ctx: &Context) -> Result<()> {
        if self.admin != ctx.sender() {
            return Err("You cannot close the ballot.".to_string());
        }
        self.accepting_votes = false;
        Ok(())
    }
    
    pub fn winner(&self, _ctx: &Context) -> Result<u32> {
        if self.accepting_votes {
            return Err("Voting is not closed.".to_string());
        }
        Ok(self
            .tally
            .iter()
            .enumerate()
            .max_by_key(|(_i, v)| *v)
            .unwrap()
            .0 as u32)
    }
}

fn main() {
    oasis_std::service!(Ballot);
}

#[cfg(test)]
mod tests {

    extern crate oasis_test;

    use super::*;
    
    fn create_account() -> (Address, Context) {
        let addr = oasis_test::create_account(0 /* initial balance */);
        let ctx = Context::default().with_sender(addr).with_gas(100_000);
        (addr, ctx)
    }

    #[test]
    fn functionality() {
        let (_admin, admin_ctx) = create_account();
        let (_voter, voter_ctx) = create_account();

        let description = "What's for dinner?";
        let candidates = vec!["beef".to_string(), "yogurt".to_string()];
        let mut ballot =
            Ballot::new(&admin_ctx, description.to_string(), candidates.clone()).unwrap();

        assert_eq!(ballot.description(&admin_ctx).unwrap(), description);
        assert_eq!(ballot.candidates(&admin_ctx).unwrap(), candidates);

        assert!(ballot.winner(&voter_ctx).is_err());

        ballot.vote(&voter_ctx, 0).unwrap();
        ballot.vote(&voter_ctx, 1).unwrap();
        ballot.vote(&admin_ctx, 1).unwrap();

        ballot.close(&voter_ctx).unwrap_err();
        ballot.close(&admin_ctx).unwrap();
        
        ballot.vote(&admin_ctx, 0).unwrap_err();

        assert_eq!(ballot.winner(&voter_ctx).unwrap(), 1);
    }
}
