//! System monitor panel rendering — workstation telemetry with visual spark gauges.

use crate::monitor::MonitorSnapshot;
use crate::theme::Theme;
use bytesize::ByteSize;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table},
    Frame,
};

pub fn render_system_monitor(frame: &mut Frame, area: Rect, snap: &MonitorSnapshot, theme: &Theme) {
    let block = Block::default()
        .title(Span::styled(
            " 󰻠 SYSTEM TELEMETRY  ─  [F6 Toggle] ",
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 3 || inner.width < 20 {
        return;
    }

    // ── Horizontal layout: CPU / Mem / Disk / Processes ──────────────────
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(26),
            Constraint::Percentage(24),
            Constraint::Percentage(22),
            Constraint::Percentage(28),
        ])
        .split(inner);

    render_cpu(frame, cols[0], snap, theme);
    render_memory(frame, cols[1], snap, theme);
    render_disk(frame, cols[2], snap, theme);
    render_processes(frame, cols[3], snap, theme);
}

fn render_gauge_bar(pct: f32, width: usize) -> (String, Color) {
    let w = width.clamp(5, 20);
    let filled = ((pct / 100.0) * w as f32).round() as usize;
    let filled = filled.min(w);
    let empty = w.saturating_sub(filled);
    let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));
    let color = if pct >= 85.0 {
        Color::Rgb(255, 107, 107) // Error Red
    } else if pct >= 65.0 {
        Color::Rgb(255, 190, 70) // Warning Amber
    } else {
        Color::Rgb(80, 220, 140) // Success Green / Cyan
    };
    (bar, color)
}

fn render_cpu(frame: &mut Frame, area: Rect, snap: &MonitorSnapshot, theme: &Theme) {
    let cpu = &snap.cpu;
    let (bar, bar_color) = render_gauge_bar(cpu.global_usage, 12);

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                "CPU: ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                bar,
                Style::default().fg(bar_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" {:.1}%", cpu.global_usage)),
        ]),
        Line::from(vec![
            Span::styled("Model: ", Style::default().fg(theme.hidden_color)),
            Span::raw(&cpu.cpu_name),
        ]),
        Line::from(vec![
            Span::styled("Load:  ", Style::default().fg(theme.hidden_color)),
            Span::raw(format!(
                "{:.2}  {:.2}  {:.2}",
                cpu.load_avg.0, cpu.load_avg.1, cpu.load_avg.2
            )),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "Per Core:",
            Style::default().fg(theme.size_color),
        )),
    ];

    let max_cores = area.height.saturating_sub(6) as usize;
    for (i, usage) in cpu.per_core.iter().enumerate().take(max_cores) {
        let (core_bar, c_color) = render_gauge_bar(*usage, 8);
        lines.push(Line::from(vec![
            Span::styled(
                format!("  C{:02} ", i),
                Style::default().fg(theme.hidden_color),
            ),
            Span::styled(core_bar, Style::default().fg(c_color)),
            Span::raw(format!(" {:.0}%", usage)),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_memory(frame: &mut Frame, area: Rect, snap: &MonitorSnapshot, theme: &Theme) {
    let mem = &snap.mem;
    let (mem_bar, mem_color) = render_gauge_bar(mem.used_pct() as f32, 12);
    let (swap_bar, swap_color) = render_gauge_bar(mem.swap_pct() as f32, 12);

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "RAM:  ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                mem_bar,
                Style::default().fg(mem_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" {:.1}%", mem.used_pct())),
        ]),
        Line::from(format!(
            "  Used:   {} / {}",
            ByteSize(mem.used),
            ByteSize(mem.total)
        )),
        Line::from(format!("  Avail:  {}", ByteSize(mem.available))),
        Line::raw(""),
        Line::from(vec![
            Span::styled("SWAP: ", Style::default().fg(theme.size_color)),
            Span::styled(swap_bar, Style::default().fg(swap_color)),
            Span::raw(format!(" {:.1}%", mem.swap_pct())),
        ]),
        Line::from(format!(
            "  Used:   {} / {}",
            ByteSize(mem.swap_used),
            ByteSize(mem.swap_total)
        )),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_disk(frame: &mut Frame, area: Rect, snap: &MonitorSnapshot, theme: &Theme) {
    let mut lines = vec![Line::from(Span::styled(
        "DISKS",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))];
    for disk in &snap.disks {
        let (disk_bar, d_color) = render_gauge_bar(disk.used_pct() as f32, 10);
        let short_mount = if disk.mount_point.len() > 10 {
            format!("…{}", &disk.mount_point[disk.mount_point.len() - 9..])
        } else {
            disk.mount_point.clone()
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<8} ", short_mount),
                Style::default().fg(theme.header_fg),
            ),
            Span::styled(disk_bar, Style::default().fg(d_color)),
        ]));
        lines.push(Line::from(format!(
            "   {}/{} ({:.0}%)",
            ByteSize(disk.used()),
            ByteSize(disk.total),
            disk.used_pct()
        )));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_processes(frame: &mut Frame, area: Rect, snap: &MonitorSnapshot, theme: &Theme) {
    let header = Row::new(vec![" PID", "Name", "CPU%", "Mem"]).style(theme.header_style());

    let rows: Vec<Row> = snap
        .processes
        .iter()
        .take(area.height.saturating_sub(2) as usize)
        .map(|p| {
            let cpu_color = if p.cpu_usage > 50.0 {
                theme.error
            } else if p.cpu_usage > 20.0 {
                theme.warning
            } else {
                theme.file_color
            };
            Row::new(vec![
                Span::styled(
                    format!(" {:<5}", p.pid),
                    Style::default().fg(theme.hidden_color),
                ),
                Span::styled(
                    p.name.chars().take(12).collect::<String>(),
                    Style::default().fg(theme.file_color),
                ),
                Span::styled(
                    format!("{:.1}%", p.cpu_usage),
                    Style::default().fg(cpu_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    ByteSize(p.memory_kb * 1024).to_string(),
                    Style::default().fg(theme.size_color),
                ),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Min(10),
            Constraint::Length(7),
            Constraint::Length(8),
        ],
    )
    .header(header);

    frame.render_widget(table, area);
}
