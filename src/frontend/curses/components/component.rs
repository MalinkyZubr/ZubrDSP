use std::collections::HashMap;
use crossterm::event::{Event, KeyEvent};


#[derive(Clone)]
pub enum Action {
    MoveUp, // cursor move up
    MoveDown, // cursor move down
    MoveLeft, // cursor move left
    MoveRight, // cursor move right
    Select, // select the current option the cursor is on top of
    Inspect, // expand menus for selected cursor space
    Escape,
    Quit, // exit submenu, or quit application
    Keystroke(KeyEvent), // do nothing
    MouseUp,
    MouseDown,
    Noop
}

pub struct ComponentWrapper {
    pub component: Box<dyn Component>,
    neighbor_above: Option<&'static str>,
    neighbor_below: Option<&'static str>,
    neighbor_left: Option<&'static str>,
    neighbor_right: Option<&'static str>,
}
impl ComponentWrapper {
    pub fn new(component: Box<dyn Component>, 
               neighbor_above: Option<&'static str>, 
               neighbor_below: Option<&'static str>, 
               neighbor_left: Option<&'static str>, 
               neighbor_right: Option<&'static str>,
    ) -> ComponentWrapper {
        Self {
            component,
            neighbor_above,
            neighbor_below,
            neighbor_left,
            neighbor_right,
        }
    }
    pub fn shift_up(&self) -> Option<&'static str> {
        self.neighbor_above
    }
    pub fn shift_down(&self) -> Option<&'static str> {
        self.neighbor_below
    }
    pub fn shift_left(&self) -> Option<&'static str> {
        self.neighbor_left
    }
    pub fn shift_right(&self) -> Option<&'static str> {
        self.neighbor_right
    }
}

pub trait Component {
    fn update(&mut self, action: Action);
    fn focus(&mut self);
    fn unfocus(&mut self);
}