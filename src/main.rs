#![feature(trait_alias)]
#![feature(specialization)]
#![feature(mpmc_channel, portable_simd, test)]
#![feature(generic_const_exprs)]

mod pipeline;
mod byte_line;
mod dsp;
mod general;

use color_eyre::Result;
use ratatui::{
    style::Stylize

    ,
    widgets::Widget
    ,
};
use strum::IntoEnumIterator;

fn main() -> Result<()> {
    Ok(())
}
