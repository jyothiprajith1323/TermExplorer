//! System monitor panel rendering.

use crate::monitor::MonitorSnapshot;
use crate::theme::Theme;
use bytesize::ByteSize;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table},
    Frame,
};

pub fn render_system_monitor(frame: &mut Frame, area: Rect, snap: &MonitorSnapshot, theme: &Theme) {
    let block = Block::default()
        .title(Span::styled(
            " 󰻠  System Monitor [F6 toggle] ",
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 3 {
        return;
    }

    // ── Horizontal layout: CPU / Mem / Disk / Processes ──────────────────
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(25),
            Constraint::Percentage(30),
        ])
        .split(inner);

    // ── CPU ───────────────────────────────────────────────────────────────
    render_cpu(frame, cols[0], snap, theme);
    // ── Memory ────────────────────────────────────────────────────────────
    render_memory(frame, cols[1], snap, theme);
    // ── Disk ──────────────────────────────────────────────────────────────
    render_disk(frame, cols[2], snap, theme);
    // ── Processes ─────────────────────────────────────────────────────────
    render_processes(frame, cols[3], snap, theme);
}

fn render_cpu(frame: &mut Frame, area: Rect, snap: &MonitorSnapshot, theme: &Theme) {
    let cpu = &snap.cpu;
    let mut lines = vec![
        Line::from(Span::styled("CPU", theme.accent_style())),
        Line::from(format!("  {}", cpu.cpu_name)),
        Line::from(format!("  Usage: {:.1}%", cpu.global_usage)),
        Line::from(format!(
            "  Load: {:.2} {:.2} {:.2}",
            cpu.load_avg.0, cpu.load_avg.1, cpu.load_avg.2
        )),
        Line::from(""),
        Line::from(Span::styled("Cores:", Style::default().fg(Color::DarkGray))),
    ];

    for (i, usage) in cpu
        .per_core
        .iter()
        .enumerate()
        .take(area.height as usize - 6)
    {
        lines.push(Line::from(format!("  C{}: {:.0}%", i, usage)));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_memory(frame: &mut Frame, area: Rect, snap: &MonitorSnapshot, theme: &Theme) {
    let mem = &snap.mem;
    let lines = vec![
        Line::from(Span::styled("Memory", theme.accent_style())),
        Line::from(format!("  Total:  {}", ByteSize(mem.total))),
        Line::from(format!("  Used:   {}", ByteSize(mem.used))),
        Line::from(format!("  Avail:  {}", ByteSize(mem.available))),
        Line::from(format!("  Usage:  {:.1}%", mem.used_pct())),
        Line::from(""),
        Line::from(Span::styled("Swap:", Style::default().fg(Color::DarkGray))),
        Line::from(format!("  Total:  {}", ByteSize(mem.swap_total))),
        Line::from(format!("  Used:   {}", ByteSize(mem.swap_used))),
        Line::from(format!("  Usage:  {:.1}%", mem.swap_pct())),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_disk(frame: &mut Frame, area: Rect, snap: &MonitorSnapshot, theme: &Theme) {
    let mut lines = vec![Line::from(Span::styled("Disks", theme.accent_style()))];
    for disk in &snap.disks {
        let short_mount = if disk.mount_point.len() > 12 {
            format!("…{}", &disk.mount_point[disk.mount_point.len() - 11..])
        } else {
            disk.mount_point.clone()
        };
        lines.push(Line::from(format!(
            "  {} {:.0}%",
            short_mount,
            disk.used_pct()
        )));
        lines.push(Line::from(format!(
            "    {}/{}",
            ByteSize(disk.used()),
            ByteSize(disk.total)
        )));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_processes(frame: &mut Frame, area: Rect, snap: &MonitorSnapshot, theme: &Theme) {
    let header = Row::new(vec!["PID", "Name", "CPU%", "Mem"]).style(theme.header_style());

    let rows: Vec<Row> = snap
        .processes
        .iter()
        .take(area.height.saturating_sub(2) as usize)
        .map(|p| {
            Row::new(vec![
                p.pid.to_string(),
                p.name.chars().take(12).collect::<String>(),
                format!("{:.1}", p.cpu_usage),
                ByteSize(p.memory_kb * 1024).to_string(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Min(10),
            Constraint::Length(5),
            Constraint::Length(8),
        ],
    )
    .header(header);

    frame.render_widget(table, area);
}
