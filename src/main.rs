#![allow(warnings)]
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use oracle::meal_plan::MConstraint;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::collections::HashMap;
use tui_big_text::{BigText, PixelSize};

pub fn main() {
    ratatui::run(|terminal| Tui::new().run(terminal));
}

#[derive(PartialEq)]
enum Screen {
    Menu,
    NewMeal,
}

#[derive(PartialEq)]
enum Mode {
    Command,
    Insert,
}

fn mode_to_str(mode: &Mode) -> &str {
    match mode {
        Mode::Command => return "Command",
        Mode::Insert => return "Insert",
        _ => return "",
    }
}

struct Tui {
    x: usize,
    y: usize,
    screen: Screen,
    new_meal: NewMeal,
    mode: Mode,
    exit: bool,
}

impl Tui {
    fn new() -> Self {
        return Tui {
            x: 0,
            y: 0,
            screen: Screen::Menu,
            exit: false,
            mode: Mode::Command,
            new_meal: NewMeal::new(),
        };
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) {
        while !self.exit {
            match self.screen {
                Screen::Menu => {
                    terminal.draw(|frame| self.render_menu(frame));
                }
                Screen::NewMeal => {
                    terminal.draw(|frame| self.render_new_meal(frame));
                }
            }
            self.handle_events();
        }
    }

    fn render_menu(&self, frame: &mut Frame) {
        let outer_area = frame.area();
        let instructions = Line::from(vec![
            format!(" {} ", mode_to_str(&self.mode)).into(),
            " Down ".into(),
            " <J> ".blue().bold(),
            " Up ".into(),
            " <K> ".blue().bold(),
            " Quit ".into(),
            " <Q> ".red().bold(),
        ]);
        let outer_block = Block::default()
            .title(" Meal Plan ")
            .title_bottom(instructions.centered())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL);
        let inner_area = outer_block.inner(frame.area());
        frame.render_widget(outer_block, frame.area());

        let menu_area = center_rect(85, 85, inner_area);
        let menu_block = Block::default();
        let menu_inner = menu_block.inner(menu_area);
        frame.render_widget(menu_block, menu_area);

        let menu_items_area = Layout::vertical([
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ])
        .flex(Flex::SpaceEvenly)
        .split(menu_inner);

        let items = vec!["New Meal", "Saved Meals", "Exit"];
        for (i, item) in items.iter().enumerate() {
            let card_area = menu_items_area[i];
            let mut style = Style::default();
            if i == self.x{
                style = Style::default().fg(Color::Rgb(200, 0, 0));
            }
            let big = BigText::builder()
                .pixel_size(PixelSize::Full)
                .lines(vec![item.to_string().into()])
                .alignment(Alignment::Center)
                .style(style)
                .build();

            frame.render_widget(big, card_area);
        }
    }

    fn render_new_meal(&mut self, frame: &mut Frame) {
        let outer_area = frame.area();
        let instructions = Line::from(vec![
            format!(" {} ", mode_to_str(&self.mode)).into(),
            " Down ".into(),
            " <J> ".blue().bold(),
            " Up ".into(),
            " <K> ".blue().bold(),
            " Quit ".into(),
            " <Q> ".red().bold(),
        ]);
        let outer_block = Block::default()
            .title(" New Meal ")
            .title_alignment(Alignment::Center)
            .title_bottom(instructions)
            .borders(Borders::ALL);
        let inner_area = outer_block.inner(frame.area());
        frame.render_widget(outer_block, outer_area);
        let input_area =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(inner_area);
        let _h = Layout::horizontal([Constraint::Percentage(50),Constraint::Percentage(50)]).split(input_area[0]);
        self.new_meal.inputs[0].render(frame, _h[0]);
        self.new_meal.inputs[1].render(frame, _h[1]);

        let imap = HashMap::from([
            ("rice".to_string(), MConstraint::Ratio(1)),
            ("chicken".to_string(), MConstraint::Fixed(250.)),
            ("ghee".to_string(), MConstraint::Fixed(2.5)),
            ("tomato".to_string(), MConstraint::Fixed(70.)),
            ("onion".to_string(), MConstraint::Fixed(70.)),
            ("mint".to_string(), MConstraint::Fixed(50.)),
        ]);
        let x = Layout::vertical([Constraint::Length(3); 6]).split(input_area[1]);
        for (index, key) in imap.keys().enumerate() {
            let y = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(x[index]);
            let mut text = TextInput::new("".to_string());
            text.value = key.to_string();
            text.render(frame, y[0]);
            self.new_meal.inputs.push(text);
            let mut num = TextInput::new("".to_string());
            if let MConstraint::Fixed(val) = imap[key] {
                num.value = format!("F: {}", val);
            } else if let MConstraint::Ratio(val) = imap[key] {
                num.value = format!("R: {}", val);
            }
            num.render(frame, y[1]);
            self.new_meal.inputs.push(num);
        }
    }

    fn handle_events(&mut self) {
        if let Ok(Event::Key(key_event)) = event::read()
            && key_event.kind == KeyEventKind::Press
        {
            if self.mode == Mode::Insert {
                self.new_meal.handle_key_event(key_event)
            }
            self.handle_key_event(key_event.clone());
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.mode {
            Mode::Command => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Char('k') => self.go_up(),
                    KeyCode::Char('j') => self.go_down(),
                    KeyCode::Char('i') => self.mode = Mode::Insert,
                    KeyCode::Enter => self.enter(),
                    _ => {}
                };
            }
            Mode::Insert => {
                match key_event.code {
                    KeyCode::Esc => self.mode = Mode::Command,
                    _ => {}
                };
            }
            _ => {}
        };
    }

    fn exit(&mut self) {
        if self.screen != Screen::Menu {
            self.screen = Screen::Menu
        } else {
            self.exit = true;
        }
    }

    fn go_up(&mut self) {
        if self.x > 0 {
            self.x -= 1;
        }
    }

    fn go_down(&mut self) {
        self.x += 1;
        // if self.x < 2 {
        //     self.x += 1;
        // }
    }
    fn go_right(&mut self){
        self.y +=1
    }
    fn go_left(&mut self){
        self.y -= 1
    }
    fn enter(&mut self) {
        if self.screen == Screen::Menu {
            match self.x {
                0 => self.screen = Screen::NewMeal,
                2 => self.exit(),
                _ => {}
            }
        }
    }
}

#[derive(Default)]
struct NewMeal {
    inputs: Vec<TextInput>,
}

impl NewMeal {
    fn new() -> Self {
        return NewMeal {
            inputs: vec![TextInput::new("Ingredient".to_string()),TextInput::new("Quantity".to_string())],
        };
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        for mut i in &mut self.inputs {
            i.handle_key_event(key_event)
        }
    }
}

#[derive(Default)]
struct TextInput {
    value: String,
    is_focused: bool,
    title: String,
    cursor_pos: usize,
}

impl TextInput {
    fn new(title: String) -> Self {
        TextInput {
            value: String::new(),
            title,
            is_focused: false,
            cursor_pos: 0,
        }
    }
    fn render(&self, frame: &mut Frame, area: Rect) {
        let outer_block = Block::default()
            .title(self.title.clone())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL);
        // frame.render_widget(outer_block, area);
        let input_widget = Paragraph::new(self.value.as_str()).block(outer_block);
        frame.render_widget(input_widget, area);
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        // if !self.is_focused{
        //     return
        // }
        match key_event.code {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.value.pop();
                    self.cursor_pos -= 1;
                }
            }
            _ => {}
        };
    }
}

fn center_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split((vertical[1]))[1]
}
