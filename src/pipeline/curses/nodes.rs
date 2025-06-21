use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind, Color, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Padding, Paragraph, Tabs, Widget},
    DefaultTerminal,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};


#[derive(Default)]
struct Nodes {
    ordering: Ordering,
    references: Vec<Arc<PipelineThread>>
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Ordering {
    #[default]
    Id,
    Time,
    State
}