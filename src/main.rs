#![allow(warnings)]
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use oracle::meal_plan::{self, MConstraint, Recipe, RecipeIngredient, SolveError, SolvedRecipe};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};
use std::collections::HashMap;
use tui_big_text::{BigText, PixelSize};

pub fn main() {
    ratatui::run(|terminal| Tui::new().run(terminal));
}

#[derive(PartialEq, Clone, Copy)]
enum Screen {
    Menu,
    NewMeal,
    SavedMeals,
}

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Command,
    Insert,
}

fn mode_to_str(mode: &Mode) -> &'static str {
    match mode {
        Mode::Command => "Command",
        Mode::Insert => "Insert",
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Name,
    Target,
    RowName(usize),
    RowKind(usize),
    RowValue(usize),
}

struct Tui {
    screen: Screen,
    menu_index: usize,
    mode: Mode,
    exit: bool,
    ingredients_db: HashMap<String, f32>,
    ingredients_db_error: Option<String>,
    new_meal: NewMealForm,
    saved: SavedMealsScreen,
    status: Option<String>,
}

impl Tui {
    fn new() -> Self {
        let (ingredients_db, ingredients_db_error) =
            match meal_plan::load_ingredient_db(&meal_plan::default_ingredients_path()) {
                Ok(map) => (map, None),
                Err(e) => (HashMap::new(), Some(e)),
            };
        Tui {
            screen: Screen::Menu,
            menu_index: 0,
            mode: Mode::Command,
            exit: false,
            ingredients_db,
            ingredients_db_error,
            new_meal: NewMealForm::blank(),
            saved: SavedMealsScreen::new(),
            status: None,
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) {
        while !self.exit {
            let _ = terminal.draw(|frame| self.render(frame));
            self.handle_events();
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        match self.screen {
            Screen::Menu => self.render_menu(frame),
            Screen::NewMeal => self.render_new_meal(frame),
            Screen::SavedMeals => self.render_saved_meals(frame),
        }
    }

    fn instructions(&self, extra: Vec<(&str, &str)>) -> Line<'static> {
        let mut spans = vec![Span::from(format!(" {} ", mode_to_str(&self.mode)))];
        for (label, key) in extra {
            spans.push(format!(" {label} ").into());
            spans.push(format!(" <{key}> ").blue().bold());
        }
        spans.push(" Quit/Back ".into());
        spans.push(" <Q> ".red().bold());
        Line::from(spans)
    }


    fn render_menu(&self, frame: &mut Frame) {
        let instructions = self.instructions(vec![("Down", "J"), ("Up", "K"), ("Select", "Enter")]);
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

        let items = ["New Meal", "Saved Meals", "Exit"];
        for (i, item) in items.iter().enumerate() {
            let card_area = menu_items_area[i];
            let style = if i == self.menu_index {
                Style::default().fg(Color::Rgb(200, 0, 0))
            } else {
                Style::default()
            };
            let big = BigText::builder()
                .pixel_size(PixelSize::Full)
                .lines(vec![item.to_string().into()])
                .alignment(Alignment::Center)
                .style(style)
                .build();

            frame.render_widget(big, card_area);
        }

        if let Some(err) = &self.ingredients_db_error {
            let warn = Paragraph::new(format!("Warning: {err}"))
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            let warn_area = Rect {
                x: inner_area.x,
                y: inner_area.y + inner_area.height.saturating_sub(1),
                width: inner_area.width,
                height: 1,
            };
            frame.render_widget(warn, warn_area);
        }
    }


    fn render_new_meal(&mut self, frame: &mut Frame) {
        let title = match &self.new_meal.editing_saved {
            Some(name) => format!(" Editing: {name} "),
            None => " New Meal ".to_string(),
        };
        let mut extra = vec![
            ("Move", "HJKL"),
            ("Edit", "I"),
            ("Add row", "A"),
            ("Del row", "X"),
            ("Toggle/Solve", "Enter"),
            ("Save", "S"),
        ];
        if self.mode == Mode::Insert {
            extra = vec![("Stop editing", "Esc")];
        }
        let instructions = self.instructions(extra);
        let outer_block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .title_bottom(instructions)
            .borders(Borders::ALL);
        let inner_area = outer_block.inner(frame.area());
        frame.render_widget(outer_block, frame.area());

        let sections = Layout::vertical([
            Constraint::Length(3),// name / target row
            Constraint::Length(3 * self.new_meal.rows.len() as u16 + 2), // ingredient rows + header
            Constraint::Min(3),// results / status
        ])
        .split(inner_area);

        let top = Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(sections[0]);
        let focus = self.new_meal.focus_target();
        self.new_meal
            .name
            .render(frame, top[0], focus == Focus::Name, self.mode);
        self.new_meal
            .target_calories
            .render(frame, top[1], focus == Focus::Target, self.mode);

        let rows_block = Block::default().title(" Ingredients ").borders(Borders::ALL);
        let rows_inner = rows_block.inner(sections[1]);
        frame.render_widget(rows_block, sections[1]);

        if self.new_meal.rows.is_empty() {
            frame.render_widget(
                Paragraph::new("No ingredients yet — press A to add one.")
                    .style(Style::default().fg(Color::DarkGray)),
                rows_inner,
            );
        } else {
            let row_areas =
                Layout::vertical(vec![Constraint::Length(3); self.new_meal.rows.len()])
                    .split(rows_inner);
            for (i, row) in self.new_meal.rows.iter().enumerate() {
                let cols = Layout::horizontal([
                    Constraint::Percentage(45),
                    Constraint::Percentage(20),
                    Constraint::Percentage(35),
                ])
                .split(row_areas[i]);

                row.name
                    .render(frame, cols[0], focus == Focus::RowName(i), self.mode);

                let kind_focused = focus == Focus::RowKind(i);
                let kind_block = Block::default()
                    .title("Type")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(if kind_focused {
                        Style::default().fg(Color::Rgb(200, 0, 0))
                    } else {
                        Style::default()
                    });
                let kind_text = Paragraph::new(row.kind.label())
                    .alignment(Alignment::Center)
                    .block(kind_block);
                frame.render_widget(kind_text, cols[1]);

                let value_title = match row.kind {
                    RowKind::Fixed => "Grams",
                    RowKind::Ratio => "Ratio weight",
                };
                row.value.render_titled(
                    frame,
                    cols[2],
                    focus == Focus::RowValue(i),
                    self.mode,
                    value_title,
                );
            }
        }

        self.render_new_meal_status(frame, sections[2]);
    }

    fn render_new_meal_status(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title(" Result ").borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();
        if let Some(status) = &self.status {
            lines.push(Line::from(status.clone()).green());
        }
        match &self.new_meal.result {
            Some(Ok(solved)) => {
                let mut names: Vec<&String> = solved.keys().collect();
                names.sort();
                for name in names {
                    lines.push(Line::from(format!("{name}: {:.1} g", solved[name])));
                }
            }
            Some(Err(err)) => {
                lines.push(Line::from(err.clone()).red());
            }
            None => {
                if lines.is_empty() {
                    lines.push(
                        Line::from("Press Enter (with a text field or nothing focused) to solve.")
                            .style(Style::default().fg(Color::DarkGray)),
                    );
                }
            }
        }
        frame.render_widget(Paragraph::new(Text::from(lines)), inner);
    }

    // --------------------------------------------------------- Saved Meals

    fn render_saved_meals(&self, frame: &mut Frame) {
        let instructions = self.instructions(vec![
            ("Down", "J"),
            ("Up", "K"),
            ("Edit", "Enter"),
            ("Delete", "D"),
        ]);
        let outer_block = Block::default()
            .title(" Saved Meals ")
            .title_alignment(Alignment::Center)
            .title_bottom(instructions)
            .borders(Borders::ALL);
        let inner_area = outer_block.inner(frame.area());
        frame.render_widget(outer_block, frame.area());

        if self.saved.recipes.is_empty() {
            frame.render_widget(
                Paragraph::new("No saved meals yet. Create one from the New Meal screen with S.")
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center),
                inner_area,
            );
            return;
        }

        let row_areas =
            Layout::vertical(vec![Constraint::Length(3); self.saved.recipes.len()]).split(inner_area);
        for (i, recipe) in self.saved.recipes.iter().enumerate() {
            let selected = i == self.saved.index;
            let style = if selected {
                Style::default().fg(Color::Rgb(200, 0, 0))
            } else {
                Style::default()
            };
            let text = format!(
                "{}  —  {:.0} kcal target, {} ingredient(s)",
                recipe.name,
                recipe.target_calories,
                recipe.ingredients.len()
            );
            let block = Block::default().borders(Borders::ALL).border_style(style);
            frame.render_widget(Paragraph::new(text).style(style).block(block), row_areas[i]);
        }

        if let Some(status) = &self.saved.status {
            let status_area = Rect {
                x: inner_area.x,
                y: inner_area.y + inner_area.height.saturating_sub(1),
                width: inner_area.width,
                height: 1,
            };
            frame.render_widget(
                Paragraph::new(status.clone()).style(Style::default().fg(Color::Yellow)),
                status_area,
            );
        }
    }

    fn handle_events(&mut self) {
        if let Ok(Event::Key(key_event)) = event::read() {
            if key_event.kind == KeyEventKind::Press {
                self.handle_key_event(key_event);
            }
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.mode {
            Mode::Command => self.handle_command_key(key_event),
            Mode::Insert => self.handle_insert_key(key_event),
        }
    }

    fn handle_command_key(&mut self, key_event: KeyEvent) {
        // 'q' always means "back one screen, or quit if already on the menu".
        if key_event.code == KeyCode::Char('q') {
            self.back_or_exit();
            return;
        }
        match self.screen {
            Screen::Menu => match key_event.code {
                KeyCode::Char('k') => {
                    if self.menu_index > 0 {
                        self.menu_index -= 1;
                    }
                }
                KeyCode::Char('j') => {
                    if self.menu_index < 2 {
                        self.menu_index += 1;
                    }
                }
                KeyCode::Enter => self.select_menu_item(),
                _ => {}
            },
            Screen::NewMeal => self.handle_new_meal_command_key(key_event),
            Screen::SavedMeals => self.handle_saved_meals_command_key(key_event),
        }
    }

    fn handle_insert_key(&mut self, key_event: KeyEvent) {
        if key_event.code == KeyCode::Esc {
            self.mode = Mode::Command;
            return;
        }
        if self.screen == Screen::NewMeal {
            let focus = self.new_meal.focus_target();
            self.new_meal.route_key_to_focused(focus, key_event);
        }
    }

    fn select_menu_item(&mut self) {
        match self.menu_index {
            0 => {
                self.new_meal = NewMealForm::blank();
                self.status = None;
                self.screen = Screen::NewMeal;
            }
            1 => {
                self.saved.refresh();
                self.screen = Screen::SavedMeals;
            }
            2 => self.exit = true,
            _ => {}
        }
    }

    fn back_or_exit(&mut self) {
        match self.screen {
            Screen::Menu => self.exit = true,
            _ => {
                self.screen = Screen::Menu;
                self.mode = Mode::Command;
            }
        }
    }

    fn handle_new_meal_command_key(&mut self, key_event: KeyEvent) {
        let focus = self.new_meal.focus_target();
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => self.new_meal.move_focus_vertical(1),
            KeyCode::Char('k') | KeyCode::Up => self.new_meal.move_focus_vertical(-1),
            KeyCode::Char('h') | KeyCode::Left => self.new_meal.move_focus_horizontal(-1),
            KeyCode::Char('l') | KeyCode::Right => self.new_meal.move_focus_horizontal(1),
            KeyCode::Char('i') => {
                if !matches!(focus, Focus::RowKind(_)) {
                    self.mode = Mode::Insert;
                }
            }
            KeyCode::Char('a') => self.new_meal.add_row(),
            KeyCode::Char('x') => self.new_meal.remove_focused_row(),
            KeyCode::Char('s') => {
                self.status = Some(self.new_meal.save());
            }
            KeyCode::Enter => {
                if let Focus::RowKind(i) = focus {
                    self.new_meal.toggle_row_kind(i);
                } else {
                    self.new_meal.solve(&self.ingredients_db);
                }
            }
            _ => {}
        }
    }

    fn handle_saved_meals_command_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('j') => {
                if !self.saved.recipes.is_empty()
                    && self.saved.index + 1 < self.saved.recipes.len()
                {
                    self.saved.index += 1;
                }
            }
            KeyCode::Char('k') => {
                if self.saved.index > 0 {
                    self.saved.index -= 1;
                }
            }
            KeyCode::Enter => {
                if let Some(recipe) = self.saved.recipes.get(self.saved.index).cloned() {
                    self.new_meal = NewMealForm::from_recipe(recipe);
                    self.status = None;
                    self.screen = Screen::NewMeal;
                }
            }
            KeyCode::Char('d') => self.saved.delete_selected(),
            _ => {}
        }
    }
}


struct NewMealForm {
    name: TextInput,
    target_calories: TextInput,
    rows: Vec<IngredientRow>,
    focus_row: usize,
    focus_col: usize,
    result: Option<Result<SolvedRecipe, String>>,
    editing_saved: Option<String>,
}

impl NewMealForm {
    fn blank() -> Self {
        NewMealForm {
            name: TextInput::new("Recipe Name"),
            target_calories: TextInput::new("Target Calories"),
            rows: Vec::new(),
            focus_row: 0,
            focus_col: 0,
            result: None,
            editing_saved: None,
        }
    }

    fn from_recipe(recipe: Recipe) -> Self {
        let mut name = TextInput::new("Recipe Name");
        name.set_value(recipe.name.clone());
        let mut target_calories = TextInput::new("Target Calories");
        target_calories.set_value(format!("{}", recipe.target_calories));
        let rows = recipe
            .ingredients
            .iter()
            .map(IngredientRow::from_recipe_ingredient)
            .collect();
        NewMealForm {
            name,
            target_calories,
            rows,
            focus_row: 0,
            focus_col: 0,
            result: None,
            editing_saved: Some(recipe.name),
        }
    }

    fn col_count(&self, row: usize) -> usize {
        if row == 0 { 2 } else { 3 }
    }

    fn focus_target(&self) -> Focus {
        if self.focus_row == 0 {
            if self.focus_col == 0 { Focus::Name } else { Focus::Target }
        } else {
            let i = self.focus_row - 1;
            match self.focus_col {
                0 => Focus::RowName(i),
                1 => Focus::RowKind(i),
                _ => Focus::RowValue(i),
            }
        }
    }

    fn move_focus_vertical(&mut self, delta: i32) {
        let max_row = self.rows.len() as i32; // row 0 = name/target, rows 1..=len = ingredients
        let new_row = (self.focus_row as i32 + delta).clamp(0, max_row) as usize;
        self.focus_row = new_row;
        let max_col = self.col_count(self.focus_row) as i32 - 1;
        self.focus_col = (self.focus_col as i32).clamp(0, max_col) as usize;
    }

    fn move_focus_horizontal(&mut self, delta: i32) {
        let max_col = self.col_count(self.focus_row) as i32 - 1;
        self.focus_col = (self.focus_col as i32 + delta).clamp(0, max_col) as usize;
    }

    fn add_row(&mut self) {
        self.rows.push(IngredientRow::blank());
        // Jump focus straight to the new row's name field so you can type right away.
        self.focus_row = self.rows.len();
        self.focus_col = 0;
    }

    fn remove_focused_row(&mut self) {
        if let Some(i) = self.focused_row_index() {
            self.rows.remove(i);
            self.focus_row = self.focus_row.min(self.rows.len());
            let max_col = self.col_count(self.focus_row) as i32 - 1;
            self.focus_col = (self.focus_col as i32).clamp(0, max_col) as usize;
        }
    }

    fn focused_row_index(&self) -> Option<usize> {
        if self.focus_row == 0 {
            None
        } else {
            Some(self.focus_row - 1)
        }
    }

    fn toggle_row_kind(&mut self, i: usize) {
        if let Some(row) = self.rows.get_mut(i) {
            row.kind = match row.kind {
                RowKind::Fixed => RowKind::Ratio,
                RowKind::Ratio => RowKind::Fixed,
            };
        }
    }

    fn route_key_to_focused(&mut self, focus: Focus, key_event: KeyEvent) {
        match focus {
            Focus::Name => self.name.handle_key_event(key_event),
            Focus::Target => self.target_calories.handle_key_event(key_event),
            Focus::RowName(i) => {
                if let Some(row) = self.rows.get_mut(i) {
                    row.name.handle_key_event(key_event);
                }
            }
            Focus::RowValue(i) => {
                if let Some(row) = self.rows.get_mut(i) {
                    row.value.handle_key_event(key_event);
                }
            }
            Focus::RowKind(_) => {}
        }
    }

    fn to_recipe(&self) -> Result<Recipe, String> {
        let name = self.name.value.trim();
        if name.is_empty() {
            return Err("Recipe name is required.".to_string());
        }
        let target_calories: f32 = self
            .target_calories
            .value
            .trim()
            .parse()
            .map_err(|_| "Target calories must be a number.".to_string())?;

        let mut ingredients = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            let ingredient_name = row.name.value.trim();
            if ingredient_name.is_empty() {
                return Err("Every ingredient row needs a name.".to_string());
            }
            let constraint = match row.kind {
                RowKind::Fixed => {
                    let grams: f32 = row.value.value.trim().parse().map_err(|_| {
                        format!("'{ingredient_name}' needs a numeric gram amount.")
                    })?;
                    MConstraint::Fixed(grams)
                }
                RowKind::Ratio => {
                    let weight: usize = row.value.value.trim().parse().map_err(|_| {
                        format!("'{ingredient_name}' needs a whole-number ratio weight.")
                    })?;
                    MConstraint::Ratio(weight)
                }
            };
            ingredients.push(RecipeIngredient {
                name: ingredient_name.to_string(),
                constraint,
            });
        }

        Ok(Recipe {
            name: name.to_string(),
            target_calories,
            ingredients,
        })
    }

    fn solve(&mut self, db: &HashMap<String, f32>) {
        self.result = Some(match self.to_recipe() {
            Ok(recipe) => meal_plan::solve_recipe(&recipe, db).map_err(|e: SolveError| e.to_string()),
            Err(e) => Err(e),
        });
    }

    fn save(&mut self) -> String {
        let recipe = match self.to_recipe() {
            Ok(r) => r,
            Err(e) => return e,
        };
        let new_name = recipe.name.clone();
        // If we're editing a saved recipe under a *different* new name, drop the old entry.
        if let Some(old_name) = &self.editing_saved {
            if old_name != &new_name {
                let _ = meal_plan::delete_recipe(old_name);
            }
        }
        match meal_plan::upsert_recipe(recipe) {
            Ok(()) => {
                self.editing_saved = Some(new_name.clone());
                format!("Saved '{new_name}'.")
            }
            Err(e) => format!("Couldn't save recipe: {e}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum RowKind {
    Fixed,
    Ratio,
}

impl RowKind {
    fn label(&self) -> &'static str {
        match self {
            RowKind::Fixed => "Fixed",
            RowKind::Ratio => "Ratio",
        }
    }
}

struct IngredientRow {
    name: TextInput,
    kind: RowKind,
    value: TextInput,
}

impl IngredientRow {
    fn blank() -> Self {
        IngredientRow {
            name: TextInput::new("Ingredient"),
            kind: RowKind::Fixed,
            value: TextInput::new("Value"),
        }
    }

    fn from_recipe_ingredient(item: &RecipeIngredient) -> Self {
        let mut name = TextInput::new("Ingredient");
        name.set_value(item.name.clone());
        let mut value = TextInput::new("Value");
        let kind = match item.constraint {
            MConstraint::Fixed(grams) => {
                value.set_value(format!("{grams}"));
                RowKind::Fixed
            }
            MConstraint::Ratio(weight) => {
                value.set_value(format!("{weight}"));
                RowKind::Ratio
            }
        };
        IngredientRow { name, kind, value }
    }
}

struct SavedMealsScreen {
    recipes: Vec<Recipe>,
    index: usize,
    status: Option<String>,
}

impl SavedMealsScreen {
    fn new() -> Self {
        SavedMealsScreen {
            recipes: Vec::new(),
            index: 0,
            status: None,
        }
    }

    fn refresh(&mut self) {
        match meal_plan::load_recipes() {
            Ok(recipes) => {
                self.recipes = recipes;
                self.status = None;
            }
            Err(e) => {
                self.recipes = Vec::new();
                self.status = Some(format!("Couldn't load saved meals: {e}"));
            }
        }
        if self.index >= self.recipes.len() {
            self.index = self.recipes.len().saturating_sub(1);
        }
    }

    fn delete_selected(&mut self) {
        if let Some(recipe) = self.recipes.get(self.index) {
            let name = recipe.name.clone();
            match meal_plan::delete_recipe(&name) {
                Ok(_) => {
                    self.status = Some(format!("Deleted '{name}'."));
                    self.refresh();
                }
                Err(e) => self.status = Some(format!("Couldn't delete '{name}': {e}")),
            }
        }
    }
}

//Widgets -

#[derive(Default)]
struct TextInput {
    value: String,
    title: String,
    cursor: usize,
}

impl TextInput {
    fn new(title: impl Into<String>) -> Self {
        TextInput {
            value: String::new(),
            title: title.into(),
            cursor: 0,
        }
    }

    fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.chars().count();
    }

    fn byte_index(&self) -> usize {
        self.value
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.value.len())
    }

    fn render(&self, frame: &mut Frame, area: Rect, focused: bool, mode: Mode) {
        self.render_titled(frame, area, focused, mode, &self.title.clone());
    }

    fn render_titled(&self, frame: &mut Frame, area: Rect, focused: bool, mode: Mode, title: &str) {
        let border_style = if focused {
            Style::default().fg(Color::Rgb(200, 0, 0))
        } else {
            Style::default()
        };
        let block = Block::default()
            .title(format!(" {title} "))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(border_style);

        // Show a text cursor only while actively typing into this field.
        let display = if focused && mode == Mode::Insert {
            let mut v = self.value.clone();
            let idx = self.byte_index();
            v.insert(idx, '│');
            v
        } else {
            self.value.clone()
        };
        frame.render_widget(Paragraph::new(display).block(block), area);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(c) => {
                let idx = self.byte_index();
                self.value.insert(idx, c);
                self.cursor += 1;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    let idx = self.byte_index();
                    self.value.remove(idx);
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.value.chars().count() {
                    let idx = self.byte_index();
                    self.value.remove(idx);
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor < self.value.chars().count() {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.value.chars().count(),
            _ => {}
        }
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
    .split(vertical[1])[1]
}
