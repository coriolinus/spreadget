use super::app::App;
use spreadget::Level;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

pub(crate) fn draw<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(15),
            ]
            .as_ref(),
        )
        .split(frame.size());

    let symbol_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
    let addr_style = Style::default().fg(Color::Gray);

    let title_text = Spans::from(vec![
        Span::styled(app.options.symbol.clone(), symbol_style),
        Span::raw(" <- "),
        Span::styled(format!("{}", app.options.address), addr_style),
    ]);

    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Black).bg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Black).bg(Color::White)),
        );
    frame.render_widget(title, chunks[0]);

    let spread_text = Spans::from(vec![
        Span::raw("Spread: "),
        Span::styled(
            format!("{:.10}", app.summary.spread),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]);
    let spread = Paragraph::new(spread_text)
        .style(Style::default().fg(Color::Black).bg(Color::White))
        .alignment(Alignment::Left);
    frame.render_widget(spread, chunks[1]);

    let table_halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(chunks[2]);
    let bids = levels_as_table("Bids", &app.summary.bids);
    frame.render_widget(bids, table_halves[0]);
    let asks = levels_as_table("Asks", &app.summary.asks);
    frame.render_widget(asks, table_halves[1]);
}

fn levels_as_table(which: &str, levels: &[Level]) -> Table<'static> {
    Table::new(levels.iter().map(|level| {
        Row::new([
            Cell::from(format!("{:.5}", level.amount)),
            Cell::from("@").style(Style::default().fg(Color::Red)),
            Cell::from(format!("{:.10}", level.price)),
            Cell::from(format!("({})", level.exchange))
                .style(Style::default().fg(Color::LightBlue)),
        ])
    }))
    .style(Style::default().bg(Color::White))
    .header(
        Row::new(["Amount", "", "Price", "Exchange"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .widths(&[
        Constraint::Min(9),
        Constraint::Length(1),
        Constraint::Min(14),
        Constraint::Min(10),
    ])
    .column_spacing(1)
    .block(
        Block::default()
            .title(which.to_string())
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::Black).bg(Color::White)),
    )
}
