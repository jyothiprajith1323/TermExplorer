//! Centralized Hit-testing Registry & Typed Action Dispatch.
//!
//! Guarantees that the coordinates used for mouse hit-testing come from the exact same
//! layout rectangles computed during rendering of the active frame.

use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetId {
    HeaderBrand,
    HeaderBreadcrumb,
    HeaderGit,
    HeaderTelemetry,
    HeaderSearch,
    HeaderPlugins,
    HeaderSettings,
    TerminalTitle,
    TerminalButtonNew,
    TerminalButtonClose,
    TerminalButtonMax,
    TerminalButtonMin,
    TerminalButtonResize,
    TerminalContent,
    ExplorerPanel,
    ExplorerRow(usize),
    PreviewPanel,
    PreviewTab(usize),
    PluginCenterTab(usize),
    PluginCenterCategory(usize),
    PluginCenterItem(usize),
    SettingsCategory(usize),
    SettingsItem(usize),
    DiagnosticsClose,
    HelpTab(usize),
    HelpClose,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    OpenSettings,
    OpenPluginCenter,
    OpenSearch,
    OpenHelp,
    OpenGitDashboard,
    ToggleTerminal,
    MaximizeTerminal,
    MinimizeTerminal,
    RestoreTerminal,
    ResizeTerminalCycle,
    ClearTerminal,
    FocusExplorer,
    FocusPreview,
    FocusTerminal,
    SelectExplorerIndex(usize),
    SelectPreviewTab(usize),
    SelectPluginCenterTab(usize),
    SelectPluginCenterCategory(usize),
    SelectPluginCenterItem(usize),
    SelectSettingsCategory(usize),
    SelectSettingsItem(usize),
    SelectHelpCategory(usize),
    ToggleDiagnostics,
    CloseActiveView,
}

#[derive(Debug, Clone)]
pub struct HitRegion {
    pub id: WidgetId,
    pub rect: Rect,
    pub action: AppAction,
    pub enabled: bool,
    pub z_index: u16,
}

#[derive(Default, Debug, Clone)]
pub struct HitRegistry {
    regions: Vec<HitRegion>,
}

impl HitRegistry {
    pub fn new() -> Self {
        Self {
            regions: Vec::with_capacity(64),
        }
    }

    pub fn clear(&mut self) {
        self.regions.clear();
    }

    pub fn register(&mut self, region: HitRegion) {
        if region.rect.width > 0 && region.rect.height > 0 {
            self.regions.push(region);
        }
    }

    pub fn register_button(&mut self, id: WidgetId, rect: Rect, action: AppAction, z_index: u16) {
        self.register(HitRegion {
            id,
            rect,
            action,
            enabled: true,
            z_index,
        });
    }

    pub fn hit_test(&self, col: u16, row: u16) -> Option<AppAction> {
        let mut hits: Vec<&HitRegion> = self
            .regions
            .iter()
            .filter(|r| {
                r.enabled
                    && col >= r.rect.x
                    && col < r.rect.x + r.rect.width
                    && row >= r.rect.y
                    && row < r.rect.y + r.rect.height
            })
            .collect();

        // Highest z-index wins. If tied, last registered wins.
        hits.sort_by_key(|r| std::cmp::Reverse(r.z_index));
        hits.first().map(|r| r.action.clone())
    }

    pub fn hit_test_region(&self, col: u16, row: u16) -> Option<&HitRegion> {
        let mut hits: Vec<&HitRegion> = self
            .regions
            .iter()
            .filter(|r| {
                r.enabled
                    && col >= r.rect.x
                    && col < r.rect.x + r.rect.width
                    && row >= r.rect.y
                    && row < r.rect.y + r.rect.height
            })
            .collect();

        hits.sort_by_key(|r| std::cmp::Reverse(r.z_index));
        hits.first().copied()
    }

    pub fn regions(&self) -> &[HitRegion] {
        &self.regions
    }
}
