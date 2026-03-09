use ratatui::layout::{Constraint, Direction};

/// Responsive layout mode based on terminal dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// ≥120 cols: 3 panels side-by-side
    Wide,
    /// 80-119 cols: 2 panels top + 1 bottom
    Compact,
    /// <80 cols, ≥40 rows: stacked vertically
    Tall,
    /// <80 cols, <40 rows: single panel at a time
    Minimal,
}

impl LayoutMode {
    /// Picks layout based on terminal dimensions.
    pub fn auto_select(cols: u16, rows: u16) -> Self {
        if cols >= 120 {
            Self::Wide
        } else if cols >= 80 {
            Self::Compact
        } else if rows >= 40 {
            Self::Tall
        } else {
            Self::Minimal
        }
    }

    /// Parses a layout mode from its lowercase name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "wide" => Some(Self::Wide),
            "compact" => Some(Self::Compact),
            "tall" => Some(Self::Tall),
            "minimal" => Some(Self::Minimal),
            _ => None,
        }
    }

    /// Returns the lowercase name of this layout mode.
    pub fn name(self) -> &'static str {
        match self {
            Self::Wide => "wide",
            Self::Compact => "compact",
            Self::Tall => "tall",
            Self::Minimal => "minimal",
        }
    }

    /// Cycles to the next layout mode: Wide→Compact→Tall→Minimal→Wide.
    pub fn next(self) -> Self {
        match self {
            Self::Wide => Self::Compact,
            Self::Compact => Self::Tall,
            Self::Tall => Self::Minimal,
            Self::Minimal => Self::Wide,
        }
    }

    /// Returns the primary panel direction for this layout mode.
    pub fn panel_direction(self) -> Direction {
        match self {
            Self::Wide => Direction::Horizontal,
            _ => Direction::Vertical,
        }
    }
}

/// Describes the outer layout structure (top bar, panels area, status bar).
pub struct LayoutSpec {
    pub mode: LayoutMode,
    /// Constraints for the outer vertical split: [top bar, panels, status bar].
    pub outer_constraints: Vec<Constraint>,
}

impl LayoutSpec {
    pub fn new(mode: LayoutMode) -> Self {
        Self {
            mode,
            outer_constraints: vec![
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(1),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_select_wide() {
        assert_eq!(LayoutMode::auto_select(120, 40), LayoutMode::Wide);
        assert_eq!(LayoutMode::auto_select(200, 50), LayoutMode::Wide);
    }

    #[test]
    fn test_auto_select_compact() {
        assert_eq!(LayoutMode::auto_select(80, 40), LayoutMode::Compact);
        assert_eq!(LayoutMode::auto_select(119, 30), LayoutMode::Compact);
    }

    #[test]
    fn test_auto_select_tall() {
        assert_eq!(LayoutMode::auto_select(79, 40), LayoutMode::Tall);
        assert_eq!(LayoutMode::auto_select(60, 60), LayoutMode::Tall);
    }

    #[test]
    fn test_auto_select_minimal() {
        assert_eq!(LayoutMode::auto_select(79, 39), LayoutMode::Minimal);
        assert_eq!(LayoutMode::auto_select(40, 20), LayoutMode::Minimal);
    }

    #[test]
    fn test_layout_cycle() {
        assert_eq!(LayoutMode::Wide.next(), LayoutMode::Compact);
        assert_eq!(LayoutMode::Compact.next(), LayoutMode::Tall);
        assert_eq!(LayoutMode::Tall.next(), LayoutMode::Minimal);
        assert_eq!(LayoutMode::Minimal.next(), LayoutMode::Wide);
    }

    #[test]
    fn test_layout_name_roundtrip() {
        for mode in [
            LayoutMode::Wide,
            LayoutMode::Compact,
            LayoutMode::Tall,
            LayoutMode::Minimal,
        ] {
            assert_eq!(LayoutMode::from_name(mode.name()), Some(mode));
        }
    }

    #[test]
    fn test_layout_spec_outer_constraints() {
        let spec = LayoutSpec::new(LayoutMode::Wide);
        assert_eq!(spec.outer_constraints.len(), 3);
    }
}
