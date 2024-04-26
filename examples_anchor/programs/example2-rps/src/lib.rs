use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;
use anchor_lang::solana_program::hash::Hash;

use self::Hand::*;
use self::HandResult::*;

declare_id!("5521eQoUE17iVGZXCwwGApaF6qExq2hkYVviETpfaXQE");

#[program]
mod example2 {
    use super::*;

    // Creates an account for the game
    pub fn new_game(ctx: Context<NewGame>, player_two: Pubkey, wait_for: u64) -> Result<()> {
        ctx.accounts
            .game
            .new([ctx.accounts.player1.key(), player_two], wait_for)
    }

    // Load the hash to the game account for upcoming hand
    pub fn place_hash(ctx: Context<PlaceHash>, hashed_hand: [u8; 32]) -> Result<()> {
        let game: &mut Account<Game> = &mut ctx.accounts.game;

        // Get player index, but also act as a check
        let indx: usize = game.get_player_index(ctx.accounts.player.key()).unwrap();

        game.place_hash(hashed_hand, indx)
    }

    // Load hand to the the game account
    pub fn place_hand(ctx: Context<PlaceHash>, hand_string: String) -> Result<()> {
        let game: &mut Account<Game> = &mut ctx.accounts.game;

        // Get player index, but also act as a check
        let indx: usize = game.get_player_index(ctx.accounts.player.key()).unwrap();

        game.place_hand(hand_string, indx)
    }

    pub fn forfeit(ctx: Context<PlaceHash>) -> Result<()> {
        let game: &mut Account<Game> = &mut ctx.accounts.game;

        game.forfeit(ctx.accounts.player.key())
    }
}

#[derive(Accounts)]
pub struct PlaceHash<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct NewGame<'info> {
    #[account(init, payer = player1, space = 64 + Game::MAXIMUM_SIZE)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub player1: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Rock,
    Paper,
    Scissors,
    Lizard,
    Spock,
}

impl Hand {
    pub fn new(hand: char) -> Result<Self> {
        match hand {
            '0' => Ok(Hand::Rock),
            '1' => Ok(Hand::Paper),
            '2' => Ok(Hand::Scissors),
            '3' => Ok(Hand::Lizard),
            '4' => Ok(Hand::Spock),
            _ => Err(SErrors::WrongHandChar.into()),
        }
    }
}

impl Default for Hand {
    fn default() -> Self {
        Hand::Scissors
    }
}

pub trait Beats {
    fn beats(&self) -> [Hand; 2];
}

impl Beats for Hand {
    fn beats(&self) -> [Hand; 2] {
        match *self {
            Rock => [Scissors, Lizard],
            Paper => [Rock, Spock],
            Scissors => [Paper, Lizard],
            Lizard => [Paper, Spock],
            Spock => [Rock, Scissors]
        }
    }
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum HandResult {
    Win,
    Lose,
    Draw,
}

// Account
////////////////////////////////////////////////////////////////

#[account]
pub struct Game {
    players: [Pubkey; 2],
    hashed_hand: [[u8; 32]; 2],
    hash_submitted: [bool; 2],
    hand: [Hand; 2],
    hand_submitted: [bool; 2],
    winner: String,
    deadline: u64,
    submission_wait_time: u64,
}

impl Game {
    // Based on account varfiable sizes
    pub const MAXIMUM_SIZE: usize = (32 * 2) + (32 * 2) + 3 * 2 + 32 + 8 + 8;

    // Player that pays for account set up calls this with both pubkeys
    fn new(&mut self, players: [Pubkey; 2], wait_for: u64) -> Result<()> {
        self.hash_submitted = [false, false];
        self.hand_submitted = [false, false];
        self.players = players;
        self.hand = [Hand::Rock, Hand::Rock];
        self.hashed_hand = [[0; 32], [0; 32]];
        self.submission_wait_time = wait_for;

        Ok(())
    }

    pub fn get_player_index(&mut self, player: Pubkey) -> Result<usize> {
        // This original code will panic instead of return the MissingPlayer error when the input player is not found
        // let index_player: usize = self.players.iter().position(|&x| x == player).unwrap();

        // match index_player {
        //     0 => Ok(index_player),
        //     1 => Ok(index_player),
        //     _ => Err(SErrors::MissingPlayer.into()),
        // }

        // This modified code handles an invalid player to return the MissingPlayer error
        let index_player: Result<usize> = self.players.iter().position(|&x| x == player).ok_or(Error::from(SErrors::MissingPlayer));

        match index_player {
            Ok(index) => match index {
                0 => Ok(index),
                1 => Ok(index),
                _ => Err(SErrors::MissingPlayer.into()),
            },
            Err(_e) => Err(SErrors::MissingPlayer.into()),
        }
    }

    pub fn pick_winner(&mut self) -> HandResult {
        let (player1, player2) = (self.hand[0].beats(), self.hand[1].beats());

        msg!("player1 hand: {:?}", self.hand[0]);
        msg!("player2 hand: {:?}", self.hand[1]);

        match (player1, player2) {
            _ if player1.contains(&self.hand[1]) => Win,
            _ if player2.contains(&self.hand[0]) => Lose,
            _ => Draw,
        }
    }

    pub fn place_hash(&mut self, hashed_hand: [u8; 32], indx: usize) -> Result<()> {
        // Set hash
        self.hashed_hand[indx] = hashed_hand;

        // Mark submission
        self.hash_submitted[indx] = true;

        Ok(())
    }

    pub fn place_hand(&mut self, hand_string: String, indx: usize) -> Result<()> {
        // Extract the first word
        let words: Vec<&str> = hand_string.split(' ').collect();

        // Hash first word
        let new_hash = hash(hand_string.as_bytes());

        // Check if the same
        if new_hash == Hash::new_from_array(self.hashed_hand[indx]) {
            // Extract hand from the first char of the frist word
            let hand: Hand = Hand::new(words[0].chars().next().unwrap()).unwrap();

            // Place hand
            self.hand[indx] = hand;

            // Mark as loaded
            self.hand_submitted[indx] = true;

            // Check if the end ie final hand
            if self.hand_submitted[0] == true && self.hand_submitted[1] == true {
                let result_p1 = self.pick_winner();

                match result_p1 {
                    Win => self.winner = self.players[0].to_string(),
                    Lose => self.winner = self.players[1].to_string(),
                    Draw => self.winner = "DRAW".to_string(),
                }
            } else {
                let clock = Clock::get()?;
                self.deadline = clock.unix_timestamp as u64 + self.submission_wait_time;
            }

            return Ok(());
        }

        return Err(SErrors::WrongHash.into());
    }

    pub fn forfeit(&mut self, forfeitor: Pubkey) -> Result<()> {
        let clock = Clock::get()?;
        if self.deadline == 0 || self.deadline > clock.unix_timestamp as u64 {
            return Err(SErrors::ForfeitDeadlineNotReached.into());
        }

        let indx: usize = self.get_player_index(forfeitor)?;
        if self.hand_submitted[indx] == true && self.hand_submitted[1 - indx] == false {
            self.winner = self.players[indx].to_string();
        } else {
            return Err(SErrors::InvalidForfeiture.into());
        }

        Ok(())
    }
}

// Errors
////////////////////////////////////////////////////////////////

#[error_code]
pub enum SErrors {
    MissingPlayer,
    WrongHandChar,
    WrongHash,
    ForfeitDeadlineNotReached,
    InvalidForfeiture,
}
