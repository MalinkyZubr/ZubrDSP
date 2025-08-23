use color_eyre::Result;
use ratatui::{
    buffer::Buffer, 
    crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseEventKind, MouseEvent}, 
    layout::{Constraint, Layout, }, 
    style::{palette::tailwind, Color, Stylize}, 
    symbols, text::Line, widgets::{Block, Padding, Paragraph, Tabs, Widget}, 
    DefaultTerminal, Frame,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use std::collections::{HashMap, HashSet};
use ratatui::layout::{Direction, Rect};
use super::components::component;
use super::components::component::{Action, Component, ComponentWrapper};


pub struct AppLayout {
    pub top_layout: Layout,
    pub bottom_layout: Layout,
}
impl AppLayout {
    pub fn new() -> Self {
        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    Constraint::Percentage(10),
                    Constraint::Percentage(90),
                ]
            );
        let bottom_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                vec![
                    Constraint::Percentage(10),
                    Constraint::Percentage(40),
                    Constraint::Percentage(40),
                ]
            );
        Self { top_layout, bottom_layout }
    }
}


pub struct AppBuilder {
    components: HashMap<&'static str, ComponentWrapper>,
    expected_components: HashSet<&'static str>
}
impl AppBuilder {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            expected_components: HashSet::new()
        }
    }
    pub fn append_component(mut self, component: Box<dyn Component>,
                            key: &'static str,
                            neighbor_above: Option<&'static str>,
                            neighbor_below: Option<&'static str>,
                            neighbor_left: Option<&'static str>,
                            neighbor_right: Option<&'static str>
    ) {
        self.expected_components.remove(key);

        self.conditional_add_expected(neighbor_above);
        self.conditional_add_expected(neighbor_below);
        self.conditional_add_expected(neighbor_left);
        self.conditional_add_expected(neighbor_right);

        let component_wrapper = ComponentWrapper::new(component, neighbor_above, neighbor_below, neighbor_left, neighbor_right);
        self.components.insert(key.clone(), component_wrapper);
    }
    fn conditional_add_expected(&mut self, key: Option<&'static str>) {
        match key {
            Some(key) => {
                if !self.expected_components.contains(key) {
                    self.expected_components.insert(key);
                }
            }
            None => {}
        }
    }
    pub fn complete(mut self) -> App {
        assert!(self.expected_components.is_empty());
        
        App {
            state: AppState::Running,
            components: self.components,
            selected_component: "",
            focusing_component: false,
            layout: AppLayout::new(),
        }
    }
}


pub struct App {
    state: AppState,
    components: HashMap<&'static str, ComponentWrapper>,
    selected_component: &'static str,
    layout: AppLayout,
    focusing_component: bool,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Quitting,
}

impl App {
    fn shift(&mut self, direction: KeyCode) -> Result<(), ()> {
        let shift_return = match direction {
            KeyCode::Left => self.components.get_mut(self.selected_component).unwrap().shift_left(), // verifiers should guarantee no trouble with unwrap
            KeyCode::Right => self.components.get_mut(self.selected_component).unwrap().shift_right(),
            KeyCode::Up => self.components.get_mut(self.selected_component).unwrap().shift_up(),
            KeyCode::Down => self.components.get_mut(self.selected_component).unwrap().shift_down(),
            _ => return Err(())
        };
        match shift_return {
            Some(key) => self.selected_component = key,
            None => ()
        };
        Ok(())
    }
    fn handle_events(&mut self, event: Option<Event>) -> Action {
        match event {
            Some(Event::Key(key_event)) => match key_event.clone().code {
                KeyCode::Esc => Action::Quit,
                KeyCode::Left => Action::MoveLeft,
                KeyCode::Right => Action::MoveRight,
                KeyCode::Up => Action::MoveUp,
                KeyCode::Down => Action::MoveDown,
                KeyCode::Enter => Action::Select,
                KeyCode::Tab => Action::Inspect,
                KeyCode::Backspace => Action::Quit,
                _ => Action::Keystroke(key_event),
            },
            Some(Event::Mouse(mouse_event)) => match mouse_event.kind {
                MouseEventKind::Down(_) => Action::MouseDown,
                MouseEventKind::Up(_) => Action::MouseUp,
                _ => Action::Noop
            },
            _ => Action::Noop
        }
    }
    fn get_component_wrapper(&mut self) -> &mut ComponentWrapper {
        self.components.get_mut(self.selected_component).unwrap()
    }
    fn get_component(&mut self) -> &mut Box<dyn Component> {
        &mut self.get_component_wrapper().component
    }
    fn update(&mut self, action: Action) {
        if self.focusing_component {
            match action.clone() {
                Action::Escape => {
                    self.focusing_component = false;
                    self.get_component().unfocus()
                },
                Action::Quit => (),
                _ => self.get_component().update(action.clone())
            }
        }
        else {
            match action.clone() {
                Action::Inspect => {
                    self.focusing_component = true;
                    self.get_component().focus();
                },
                Action::Quit => (),
                Action::MoveUp => self.shift(KeyCode::Up).unwrap(),
                Action::MoveDown => self.shift(KeyCode::Down).unwrap(),
                Action::MoveLeft => self.shift(KeyCode::Left).unwrap(),
                Action::MoveRight => self.shift(KeyCode::Right).unwrap(),
                _ => self.get_component().update(action.clone())
            }
        }
        match action {
            Action::Quit => self.state = AppState::Quitting,
            _ => ()
        }
    }
}


pub fn construct_app_skeleton() -> AppBuilder {
    AppBuilder::new()
}