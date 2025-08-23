use color_eyre::Result;
use crossterm::event::KeyModifiers;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Margin, Rect},
    style::{self, Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
    DefaultTerminal, Frame,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use style::palette::tailwind;
use super::component::{Action, Component};
use crate::pipeline::api::ThreadDiagnostic;


struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    selected_column_style_fg: Color,
    selected_cell_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_row_style_fg: color.c400,
            selected_column_style_fg: color.c400,
            selected_cell_style_fg: color.c600,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}


struct Nodes {
    ordering: OrderingParam,
    focused: bool,
    ascending: bool,
    diagnostics: Vec<ThreadDiagnostic>,
    state: TableState,
    longest_item_lens: (u16, u16, u16, u16),
    scroll_state: ScrollbarState,
    colors: TableColors,
    color_index: usize,
    //references: Vec<Arc<PipelineThread>>
}


#[derive(Clone, Copy, PartialEq, Eq)]
enum OrderingParam {
    Id,
    ExecTime,
    State,
}

const ITEM_HEIGHT: usize = 2;

impl Nodes {
    fn new(num_nodes: usize) -> Self {
        Self {
            state: TableState::default().with_selected(0),
            longest_item_lens: (20, 20, 20, 20),
            scroll_state: ScrollbarState::new((num_nodes - 1) * ITEM_HEIGHT),
            colors: TableColors::new(&tailwind::INDIGO),
            color_index: 0,
            diagnostics: Vec::new(),
            ordering: OrderingParam::Id,
            ascending: false,
            focused: false,
        }
    }
    
    fn sort_diagnostic(&mut self) {
        match (self.ordering, self.ascending) {
            (OrderingParam::Id, false) => {
                self.diagnostics.sort_by(
                    |a, b| b.id.cmp(&a.id));
            },
            (OrderingParam::Id, true) => {
                self.diagnostics.sort_by(
                    |a, b| a.id.cmp(&b.id));
            },
            (OrderingParam::State, false) => {
                self.diagnostics.sort_by(
                    |a, b| 
                        b.thread_state.to_u8_ref().cmp(&a.thread_state.to_u8_ref()));
            },
            (OrderingParam::State, true) => {
                self.diagnostics.sort_by(
                    |a, b| 
                        a.thread_state.to_u8_ref().cmp(&b.thread_state.to_u8_ref()));
            },
            (OrderingParam::ExecTime, false) => {
                self.diagnostics.sort_by(
                    |a, b| 
                        b.execution_time.cmp(&a.execution_time));
            }
            (OrderingParam::ExecTime, true) => {
                self.diagnostics.sort_by(
                    |a, b| 
                        a.execution_time.cmp(&b.execution_time));
            },
        }
    }
    pub fn push_diagnostics(&mut self, diagnostics: Vec<ThreadDiagnostic>) {
        self.diagnostics = diagnostics;
        self.sort_diagnostic();
    } // should render here
    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.diagnostics.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }
    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.diagnostics.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }
    fn draw(&mut self, frame: &mut Frame) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
        let rects = vertical.split(frame.area());

        self.set_colors();

        self.render_table(frame, rects[0]);
        self.render_scrollbar(frame, rects[0]);
        self.render_footer(frame, rects[1]);
    }
    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);
        let selected_col_style = Style::default().fg(self.colors.selected_column_style_fg);
        let selected_cell_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_cell_style_fg);

        let header = ["Name", "Address", "Email"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self.diagnostics.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => self.colors.normal_row_color,
                _ => self.colors.alt_row_color,
            };
            let item = data.ref_array();
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                .collect::<Row>()
                .style(Style::new().fg(self.colors.row_fg).bg(color))
                .height(4)
        });
        let bar = " █ ";
        let t = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Length(self.longest_item_lens.0 + 1),
                Constraint::Min(self.longest_item_lens.1 + 1),
                Constraint::Min(self.longest_item_lens.2),
            ],
        )
            .header(header)
            .row_highlight_style(selected_row_style)
            .column_highlight_style(selected_col_style)
            .cell_highlight_style(selected_cell_style)
            .highlight_symbol(Text::from(vec![
                "".into(),
                bar.into(),
                bar.into(),
                "".into(),
            ]))
            .bg(self.colors.buffer_bg)
            .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Text::from_iter("Text here"))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
        frame.render_widget(info_footer, area);
    }
}
impl Component for Nodes {
    fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::MouseDown => self.next_row(),
            Action::MouseUp => self.previous_row(),
            Action::Inspect => (), // to open up special menus and charts for a node
            Action::Escape => (), // to get out of special menus
            Action::Keystroke(event) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    match event.code {
                        KeyCode::Char('i') => self.ordering = OrderingParam::Id,
                        KeyCode::Char('s') => self.ordering = OrderingParam::State,
                        KeyCode::Char('e') => self.ordering = OrderingParam::ExecTime,
                        KeyCode::Char('r') => self.ascending = !self.ascending,
                        _ => (),
                    }
                }
            }
            _ => ()
        }
        Ok(())
    }
    fn focus(&mut self) {
        self.focused = true;
    }
    fn unfocus(&mut self) {
        self.focused = false;
    }
}