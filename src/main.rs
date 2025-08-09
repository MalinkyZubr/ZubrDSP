#![feature(trait_alias)]
#![feature(mpmc_channel, portable_simd, test)]
#![feature(let_chains)]


use pipeline::logging::initialize_logger;


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
    initialize_logger();
    Ok(())
}
