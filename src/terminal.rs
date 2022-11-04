use crossterm::{
  event::{self, Event, KeyCode},
};
use std::{io};
use tui::{
  backend::{Backend},
  layout::{Constraint, Layout},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, Cell, Row, Table, TableState, Paragraph},
  text::{Span, Spans},
  Frame, Terminal,
};

use crate::model_ec2::{EC2Instances, ViewMode};

pub struct TopWindowOptions<'a> {
  content: Vec<Spans<'a>>,
}
impl<'a> TopWindowOptions<'a> {
  pub fn new(content: Vec<Spans<'a>>) -> TopWindowOptions<'a> {
      TopWindowOptions {
        content: content,
      }
  }
}
pub struct TableOptions {
  ec2_instances: EC2Instances
}
impl TableOptions {
  pub fn new(ec2_instances: EC2Instances) -> TableOptions {
      TableOptions {
        ec2_instances: ec2_instances,
      }
  }
}
pub struct App<'a> {
  state: TableState,
  top_window_options: TopWindowOptions<'a>,
  table_options: TableOptions
}

impl<'a> App<'a> {
  pub fn new(top_window_options: TopWindowOptions<'a>, table_options: TableOptions) -> App<'a> {
      App {
          state: TableState::default(),
          top_window_options: top_window_options,
          table_options: table_options,
      }
  }
  pub fn next(&mut self) {
      let i = match self.state.selected() {
          Some(i) => {
              if i >= self.table_options.ec2_instances.to_vec().len() - 1 {
                  0
              } else {
                  i + 1
              }
          }
          None => 0,
      };
      self.state.select(Some(i));
  }

  pub fn previous(&mut self) {
      let i = match self.state.selected() {
          Some(i) => {
              if i == 0 {
                  self.table_options.ec2_instances.to_vec().len() - 1
              } else {
                  i - 1
              }
          }
          None => 0,
      };
      self.state.select(Some(i));
  }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('d') => app.table_options.ec2_instances.set_view_mode(ViewMode::Instance),
                KeyCode::Char('n') => app.table_options.ec2_instances.set_view_mode(ViewMode::NormalizationFactor),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                _ => {}
            }
        }
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .margin(0)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let binding = app.table_options.ec2_instances.header();
    let header_cells = binding
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1);
    let binding = app.table_options.ec2_instances.to_vec();
    let rows = binding.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.as_str())
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.as_str()));
        Row::new(cells).height(height as u16)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(app.table_options.ec2_instances.title()))
        .highlight_style(selected_style)
        .widths(app.table_options.ec2_instances.widths());
    f.render_stateful_widget(t, rects[1], &mut app.state);


    // Block
    let block = Block::default()
      .borders(Borders::ALL)
      .title("Summary");
    let paragraph = Paragraph::new(app.top_window_options.content.clone())
      .block(block);
    f.render_widget(paragraph, rects[0]);
}