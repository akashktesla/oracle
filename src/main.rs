#![allow(warnings)]
// use oracle::meal_plan;
// fn main() {
//     meal_plan::main();
// }
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tui_big_text::{BigText, PixelSize};

pub fn main() {
    ratatui::run(|terminal| Tui::new().run(terminal));
}

#[derive(PartialEq)]
enum Screen {
    Menu,
    NewMeal,
}

struct Tui {
    state: usize,
    screen: Screen,
    exit: bool,
}

impl Tui {
    fn new() -> Self {
        return Tui {
            state: 0,
            screen: Screen::Menu,
            exit: false,
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
            if i == self.state {
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

    fn render_new_meal(&self, frame: &mut Frame) {
        let outer_area = frame.area();
        let outer_block = Block::default()
            .title(" New Meal ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL);
        let inner_area = outer_block.inner(frame.area());
        frame.render_widget(outer_block, outer_area);
        let input_area = Layout::vertical([
            Constraint::Percentage(10),
        ]).split(inner_area);
        let input1 = TextInput::new("Meal Name".to_string());
        input1.render(frame,input_area[0]);
    }

    fn handle_events(&mut self) {
        match event::read() {
            Ok(Event::Key(key_event)) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('k') => self.go_up(),
            KeyCode::Char('j') => self.go_down(),
            KeyCode::Enter => self.enter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        if self.screen != Screen::Menu {
            self.screen = Screen::Menu
        } else {
            self.exit = true;
        }
    }

    fn go_up(&mut self) {
        if self.state > 0 {
            self.state -= 1;
        }
    }

    fn go_down(&mut self) {
        if self.state < 2 {
            self.state += 1;
        }
    }

    fn enter(&mut self) {
        if self.screen == Screen::Menu {
            match self.state {
                0 => self.screen = Screen::NewMeal,
                2 => self.exit(),
                _=>{}
            }
        }
    }
}

struct TextInput{
    value: String,
    is_focused: bool,
    title: String,
    cursor_pos:usize,
}

impl TextInput{
    fn new(title:String)->Self{
        TextInput{
            value: String::new(),
            title,
            is_focused: false,
            cursor_pos: 0,
        }
    }
    fn render(&self,frame: &mut Frame,area: Rect){
        let outer_block = Block::default()
            .title(self.title.clone())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL);
        // frame.render_widget(outer_block, area);
        let input_widget = Paragraph::new(self.value.as_str()).block(outer_block);
        frame.render_widget(input_widget,area);
    }
    fn handle_key_event(&mut self,key_event: KeyEvent){
        // if !self.is_focused{
        //     return 
        // }
        match key_event.code{
            KeyCode::Char(c) => {
                self.value.insert(self.cursor_pos,c);
                self.cursor_pos +=1;
            }
            _=>{}
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
